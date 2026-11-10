//! OS-level window behaviour that Tauri's window config cannot express.
//!
//! On macOS the hub is turned into a non-activating `NSPanel` (via `tauri-nspanel`)
//! at `NSStatusWindowLevel` (25): above ordinary and floating windows and, as a
//! panel, allowed into other apps' full-screen Spaces.

use crate::error::AppResult;

/// Turns `window` into a true overlay: invisible to screenshots, recordings and screen
/// sharing, and visible over other apps' full-screen Spaces.
///
/// - `sharingType = .none` is the product's whole point: the overlay is visible to the
///   user and to nobody they share their screen with. `CLOAKVIEW_SHARING` (experiment,
///   any value) turns it off.
/// - The window's class is swapped to an `NSPanel` subclass with the
///   `NonactivatingPanel` style: it can float over another app's full-screen Space
///   without activating cloakview and pulling the user out of that Space.
/// - `CanJoinAllSpaces | FullScreenAuxiliary` lets it follow the user into every Space,
///   full-screen ones included.
///
/// Must run on the main thread (`setup` does) and after `tauri_nspanel::init()` is
/// registered (`to_panel` reads that plugin's state). If it fails, setup fails and the
/// app does not start — better no overlay than one that leaks into a screen share.
#[cfg(target_os = "macos")]
pub fn make_overlay(window: &tauri::WebviewWindow) -> AppResult<()> {
    use crate::error::AppError;
    use objc2_app_kit::{
        NSStatusWindowLevel, NSWindow, NSWindowCollectionBehavior, NSWindowSharingType,
        NSWindowStyleMask,
    };
    use objc2_foundation::MainThreadMarker;
    use tauri_nspanel::WebviewWindowExt;

    let ns_win = window.ns_window().map_err(|e| AppError::Audio(e.to_string()))?;
    if ns_win.is_null() {
        return Err(AppError::Audio("window has no NSWindow".into()));
    }
    // `setup` runs on the main thread; this is only the check.
    MainThreadMarker::new().ok_or_else(|| AppError::Audio("not on main thread".into()))?;

    // Swaps the class of the same object in place (`object_setClass`), so `ns_win`
    // still points at it afterwards.
    let panel = window.to_panel().map_err(|e| AppError::Audio(e.to_string()))?;

    // SAFETY: `ns_win` is the non-null window object Tauri owns for `window` (now an
    // NSPanel subclass, still an NSWindow), which outlives this call; we are on the main
    // thread, where AppKit windows may be touched.
    //
    // The typed setters go through objc2 rather than the panel: tauri-nspanel's
    // `set_style_mask`/`set_level` take `i32` (AppKit expects 64-bit NSUInteger /
    // NSInteger) and `set_collection_behaviour` takes the `cocoa` crate's type.
    unsafe {
        let win = &*ns_win.cast::<NSWindow>();
        if std::env::var("CLOAKVIEW_SHARING").is_err() {
            win.setSharingType(NSWindowSharingType::None);
        }
        win.setStyleMask(NSWindowStyleMask::NonactivatingPanel);
        win.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary,
        );
    }
    panel.set_hides_on_deactivate(false);
    panel.set_floating_panel(true);
    panel.set_becomes_key_only_if_needed(true);
    // After `set_floating_panel`, never before: `setFloatingPanel:YES` resets the level
    // to NSFloatingWindowLevel (3), which the probe caught.
    // SAFETY: same window, main thread, immutable AppKit constant (see above).
    unsafe { (*ns_win.cast::<NSWindow>()).setLevel(NSStatusWindowLevel) };
    panel.show();
    Ok(())
}

/// Every other platform has no equivalent here: succeeds without doing anything,
/// so callers need no `cfg` of their own.
#[cfg(not(target_os = "macos"))]
pub fn make_overlay(_window: &tauri::WebviewWindow) -> AppResult<()> {
    Ok(())
}

/// EXPERIMENT (debug builds only): every second, logs to stderr the hub's real AppKit
/// state whenever it changes, so a full-screen test shows *why* the overlay leaves:
/// whether something reset its level or collection behavior, or whether macOS just
/// refuses to place it on the active (full-screen) Space.
#[cfg(all(debug_assertions, target_os = "macos"))]
pub fn spawn_overlay_probe(window: tauri::WebviewWindow) {
    use objc2_app_kit::{NSApplication, NSWindow, NSWindowOcclusionState};
    use objc2_foundation::MainThreadMarker;
    use std::sync::{Arc, Mutex};

    let last = Arc::new(Mutex::new(String::new()));
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let w = window.clone();
        let last = last.clone();
        let _ = window.run_on_main_thread(move || {
            let Ok(ptr) = w.ns_window() else { return };
            let Some(mtm) = MainThreadMarker::new() else { return };
            if ptr.is_null() {
                return;
            }
            // SAFETY: non-null NSWindow* owned by Tauri, read on the main thread.
            let line = unsafe {
                let win = &*ptr.cast::<NSWindow>();
                format!(
                    "level={} behavior=0x{:x} onActiveSpace={} visible={} occlusionVisible={} activationPolicy={}",
                    win.level(),
                    win.collectionBehavior().0,
                    win.isOnActiveSpace(),
                    win.isVisible(),
                    win.occlusionState().contains(NSWindowOcclusionState::Visible),
                    NSApplication::sharedApplication(mtm).activationPolicy().0,
                )
            };
            let mut prev = last.lock().unwrap();
            if *prev != line {
                eprintln!("[overlay-probe] {line}");
                *prev = line;
            }
        });
    });
}
