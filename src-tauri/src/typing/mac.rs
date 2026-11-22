//! The macOS side of "type at cursor": the Accessibility probe of the focused
//! field, keyboard events, and the pasteboard. None of this can run in CI (it
//! needs a logged-in session and the Accessibility permission), so the rules
//! live in `logic.rs` and this file stays thin; it is covered by the manual
//! checklist in `docs/release-checklist.md`.

use super::logic::{Focus, Keyboard};
use crate::error::{AppError, AppResult};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardTypeString, NSWorkspace};
use objc2_core_graphics::{
    CGEvent, CGEventFlags, CGEventSource, CGEventSourceStateID, CGEventTapLocation,
};
use objc2_foundation::{NSArray, NSData, NSDictionary, NSNumber, NSString};
use std::ffi::c_void;
use std::time::Duration;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    fn AXUIElementCreateSystemWide() -> *mut c_void;
    fn AXUIElementCopyAttributeValue(element: *const c_void, attribute: *const c_void, value: *mut *mut c_void) -> i32;
    fn AXUIElementIsAttributeSettable(element: *const c_void, attribute: *const c_void, settable: *mut bool) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
}

/// Virtual key codes (ANSI layout; only used for Return and the V of ⌘V).
const KEY_RETURN: u16 = 36;
const KEY_V: u16 = 9;
/// How long the target app gets to read the pasteboard before it is restored.
const PASTE_SETTLE: Duration = Duration::from_millis(150);
const MAX_VALUE_CHARS: usize = 500;

/// Whether CloakView may read the focused field and post keyboard events.
/// `prompt = true` shows the system's "allow in Privacy & Security" dialog once.
pub fn accessibility_trusted(prompt: bool) -> bool {
    let key = NSString::from_str("AXTrustedCheckOptionPrompt");
    let value = NSNumber::new_bool(prompt);
    let options = NSDictionary::from_slices(&[&*key], &[&*value as &AnyObject]);
    // SAFETY: an NSDictionary is a toll-free bridged CFDictionary; it outlives the call.
    unsafe { AXIsProcessTrustedWithOptions(Retained::as_ptr(&options) as *const c_void) }
}

fn frontmost() -> (Option<String>, Option<i32>) {
    let app = NSWorkspace::sharedWorkspace().frontmostApplication();
    match app {
        Some(app) => (app.localizedName().map(|n| n.to_string()), Some(app.processIdentifier())),
        None => (None, None),
    }
}

/// A copied (+1) AX attribute value, released on drop.
struct AxValue(*mut c_void);

impl Drop for AxValue {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: the pointer came from a Copy… call (+1) and is released once.
            unsafe { CFRelease(self.0) };
        }
    }
}

impl AxValue {
    fn copy(element: *const c_void, attribute: &str) -> Option<AxValue> {
        let name = NSString::from_str(attribute);
        let mut out: *mut c_void = std::ptr::null_mut();
        // SAFETY: `element` is a live AXUIElementRef, `name` a bridged CFString.
        let err = unsafe {
            AXUIElementCopyAttributeValue(element, Retained::as_ptr(&name) as *const c_void, &mut out)
        };
        (err == 0 && !out.is_null()).then_some(AxValue(out))
    }

    /// The value as a string, when it is one (AXValue can also be a number).
    fn as_string(&self) -> Option<String> {
        // SAFETY: AX string values are CFStrings, toll-free bridged to NSString; the
        // class check keeps a number or element from being read as a string.
        let obj: &AnyObject = unsafe { &*(self.0 as *const AnyObject) };
        obj.downcast_ref::<NSString>().map(|s| s.to_string())
    }
}

fn is_settable(element: *const c_void, attribute: &str) -> bool {
    let name = NSString::from_str(attribute);
    let mut settable = false;
    // SAFETY: as in `AxValue::copy`.
    let err = unsafe {
        AXUIElementIsAttributeSettable(element, Retained::as_ptr(&name) as *const c_void, &mut settable)
    };
    err == 0 && settable
}

/// What the cursor is in right now. Anything unreadable (browser page content,
/// Electron apps, any AX error) is `Inconclusive`, not a refusal.
pub fn probe_focus() -> Focus {
    let (app, pid) = frontmost();
    let inconclusive = || Focus::Inconclusive { app: app.clone(), pid };
    // SAFETY: returns a +1 AXUIElementRef (released by `AxValue`).
    let system = AxValue(unsafe { AXUIElementCreateSystemWide() });
    if system.0.is_null() {
        return inconclusive();
    }
    let Some(focused) = AxValue::copy(system.0, "AXFocusedUIElement") else {
        return inconclusive();
    };
    let role = AxValue::copy(focused.0, "AXRole").and_then(|v| v.as_string()).unwrap_or_default();
    let subrole = AxValue::copy(focused.0, "AXSubrole").and_then(|v| v.as_string()).unwrap_or_default();
    if subrole == "AXSecureTextField" {
        return Focus::Secure;
    }
    let editable_role = matches!(role.as_str(), "AXTextField" | "AXTextArea" | "AXComboBox");
    // Any other element counts only if it holds settable *text*: a slider or a
    // checkbox also has a settable AXValue, but a number, not a string.
    let text = AxValue::copy(focused.0, "AXValue").and_then(|v| v.as_string());
    if editable_role || (text.is_some() && is_settable(focused.0, "AXValue")) {
        let value: String = text.unwrap_or_default().chars().take(MAX_VALUE_CHARS).collect();
        return Focus::Editable {
            app: app.clone().unwrap_or_else(|| "unknown app".into()),
            pid: pid.unwrap_or_default(),
            role,
            value,
        };
    }
    // Containers say nothing about what is inside them (Chrome reports the page
    // as a web area); only a concrete non-text element is a refusal.
    match role.as_str() {
        "" | "AXWebArea" | "AXGroup" | "AXScrollArea" | "AXUnknown" => inconclusive(),
        _ => Focus::NotEditable,
    }
}

pub struct MacKeyboard;

/// Posts one key press (down then up) with exactly `flags` held, so the
/// modifiers still physically down from the shortcut never leak into it.
fn press(key: u16, unicode: Option<&[u16]>, flags: CGEventFlags) -> AppResult<()> {
    let src = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
    for down in [true, false] {
        let event = CGEvent::new_keyboard_event(src.as_deref(), key, down)
            .ok_or_else(|| AppError::Audio("could not create a keyboard event".into()))?;
        if let Some(units) = unicode {
            // SAFETY: `units` is valid for `units.len()` UTF-16 code units for the call.
            unsafe { CGEvent::keyboard_set_unicode_string(Some(&event), units.len() as u64, units.as_ptr()) };
        }
        CGEvent::set_flags(Some(&event), flags);
        CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
    }
    Ok(())
}

/// Every item on the pasteboard as (type, bytes) pairs, so it can be put back.
fn snapshot(pb: &NSPasteboard) -> Vec<Vec<(Retained<NSString>, Retained<NSData>)>> {
    let Some(items) = pb.pasteboardItems() else { return Vec::new() };
    items
        .iter()
        .map(|item| {
            item.types()
                .iter()
                .filter_map(|t| item.dataForType(&t).map(|d| (t, d)))
                .collect()
        })
        .collect()
}

fn restore(pb: &NSPasteboard, saved: Vec<Vec<(Retained<NSString>, Retained<NSData>)>>) {
    pb.clearContents();
    if saved.is_empty() {
        return;
    }
    let items: Vec<Retained<NSPasteboardItem>> = saved
        .into_iter()
        .map(|entries| {
            let item = NSPasteboardItem::new();
            for (t, d) in entries {
                item.setData_forType(&d, &t);
            }
            item
        })
        .collect();
    let writers: Vec<_> = items.into_iter().map(objc2::runtime::ProtocolObject::from_retained).collect();
    pb.writeObjects(&NSArray::from_retained_slice(&writers));
}

impl Keyboard for MacKeyboard {
    fn type_char(&self, c: char) -> AppResult<()> {
        let mut buf = [0u16; 2];
        let units = c.encode_utf16(&mut buf);
        press(0, Some(units), CGEventFlags(0))
    }

    fn shift_return(&self) -> AppResult<()> {
        press(KEY_RETURN, None, CGEventFlags::MaskShift)
    }

    fn paste(&self, text: &str) -> AppResult<()> {
        let pb = NSPasteboard::generalPasteboard();
        let saved = snapshot(&pb);
        pb.clearContents();
        // SAFETY: NSPasteboardTypeString is a valid, immutable AppKit constant.
        let wrote = pb.setString_forType(&NSString::from_str(text), unsafe { NSPasteboardTypeString });
        let result = if wrote {
            let r = press(KEY_V, None, CGEventFlags::MaskCommand);
            std::thread::sleep(PASTE_SETTLE);
            r
        } else {
            Err(AppError::Audio("could not put the text on the pasteboard".into()))
        };
        // Always put the user's clipboard back, also when the paste failed.
        restore(&pb, saved);
        result
    }

    fn frontmost_pid(&self) -> Option<i32> {
        frontmost().1
    }
}
