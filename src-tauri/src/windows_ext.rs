//! OS-level window behaviour that Tauri's window config cannot express.
//!
//! Warning: on macOS the hub sits at `CGShieldingWindowLevel()` (2147483628). That
//! puts the pill above the menu bar and above system dialogs, not only above
//! ordinary windows.

use crate::error::AppResult;

/// Turns `window` into a true overlay: invisible to screenshots, recordings and screen
/// sharing, and visible over other apps' full-screen Spaces.
///
/// - `sharingType = .none` is the product's whole point: the overlay is visible to the
///   user and to nobody they share their screen with.
/// - `CanJoinAllSpaces | FullScreenAuxiliary | Stationary | IgnoresCycle` lets it
///   follow the user into every Space, stay put during Space transitions and stay out
///   of Cmd-` window cycling.
/// - The shielding window level keeps it above other apps' full-screen windows;
///   `alwaysOnTop` alone (the floating level) does not.
///
/// Must run on the main thread (`setup` does). Called from `lib.rs` right after the
/// app state is managed; if it fails, setup fails and the app does not start —
/// better no overlay than one that leaks into a screen share.
#[cfg(target_os = "macos")]
pub fn make_overlay(window: &tauri::WebviewWindow) -> AppResult<()> {
    use crate::error::AppError;
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior, NSWindowSharingType};
    use objc2_core_graphics::CGShieldingWindowLevel;
    use objc2_foundation::MainThreadMarker;

    let ns_win = window.ns_window().map_err(|e| AppError::Audio(e.to_string()))?;
    if ns_win.is_null() {
        return Err(AppError::Audio("hub window has no NSWindow".into()));
    }
    // `setup` runs on the main thread; this is only the check.
    MainThreadMarker::new().ok_or_else(|| AppError::Audio("not on main thread".into()))?;
    // SAFETY: `ns_win` is the non-null `NSWindow*` Tauri owns for `window`, which outlives
    // this call, and we are on the main thread, where AppKit windows may be touched.
    unsafe {
        let win = &*ns_win.cast::<NSWindow>();
        let behavior = win.collectionBehavior()
            | NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle;
        win.setCollectionBehavior(behavior);
        win.setLevel(CGShieldingWindowLevel() as isize);
        if std::env::var("CLOAKVIEW_SHARING").is_err() {
            win.setSharingType(NSWindowSharingType::None);
        }
    }
    Ok(())
}

/// Every other platform has no equivalent here: succeeds without doing anything,
/// so callers need no `cfg` of their own.
#[cfg(not(target_os = "macos"))]
pub fn make_overlay(_window: &tauri::WebviewWindow) -> AppResult<()> {
    Ok(())
}
