//! Global shortcuts (Task 15): quick-hide, push-to-ask, start/stop listen and read-screen.
//!
//! The accelerators come from `Settings::shortcuts` and are re-registered whenever
//! the settings are saved; nothing here hard-codes a key. A blank accelerator turns
//! that action off.
//!
//! None of the actions uses Tauri's `show()`/`set_focus()`: both activate the app,
//! and activating it is what yanks the user out of another app's full-screen Space.
//! Showing goes through `windows_ext::make_overlay` (idempotent, ends in
//! `order_front_regardless` and never makes the window key, non-activating), always on the main thread.

use crate::commands;
use crate::error::{AppError, AppResult};
use crate::events::{PUSH_TO_ASK_EVENT, READ_SCREEN_EVENT};
use crate::state::AppState;
use crate::windows_ext;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// The accelerators, in `tauri-plugin-global-shortcut` syntax (case-insensitive,
/// e.g. `cmdOrControl+shift+h`). `serde(default)` so a `settings.json` written
/// before this field existed, or one missing a key, gets the defaults.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Shortcuts {
    /// Hides the whole overlay if the hub is visible, brings the hub back otherwise.
    pub quick_hide: String,
    /// Opens the Ask window (through the hub, the same path as its Ask button).
    pub push_to_ask: String,
    /// Starts listening, or stops if a session is running.
    pub toggle_listen: String,
    /// Reads the screen and answers what is on it (through the hub, the same path as
    /// its Screen button).
    pub read_screen: String,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            quick_hide: "cmdOrControl+shift+h".into(),
            push_to_ask: "cmdOrControl+shift+space".into(),
            toggle_listen: "cmdOrControl+shift+l".into(),
            read_screen: "cmdOrControl+shift+s".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    QuickHide,
    PushToAsk,
    ToggleListen,
    ReadScreen,
}

impl Shortcuts {
    /// Every action with its accelerator, in a fixed order.
    pub fn bindings(&self) -> [(Action, &str); 4] {
        [
            (Action::QuickHide, self.quick_hide.as_str()),
            (Action::PushToAsk, self.push_to_ask.as_str()),
            (Action::ToggleListen, self.toggle_listen.as_str()),
            (Action::ReadScreen, self.read_screen.as_str()),
        ]
    }
}

/// A binding that could not be registered: invalid syntax, a duplicate, or a key
/// another app already holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failed {
    pub action: Action,
    pub accelerator: String,
    pub reason: String,
}

/// Replaces every registered shortcut with `shortcuts`. Never fails as a whole: a
/// binding that cannot be registered is logged and skipped, the rest still work, and
/// the app starts regardless. Returns the bindings that failed.
pub fn register_all(app: &AppHandle, shortcuts: &Shortcuts) -> Vec<Failed> {
    let gs = app.global_shortcut();
    if let Err(e) = gs.unregister_all() {
        eprintln!("[hotkeys] could not unregister the previous shortcuts: {e}");
    }
    let mut failed = Vec::new();
    for (action, accelerator) in shortcuts.bindings() {
        let accelerator = accelerator.trim();
        if accelerator.is_empty() {
            continue; // turned off
        }
        let registered = gs.on_shortcut(accelerator, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                dispatch(app.clone(), action);
            }
        });
        if let Err(e) = registered {
            eprintln!("[hotkeys] {action:?} = {accelerator:?} not registered: {e}");
            failed.push(Failed { action, accelerator: accelerator.into(), reason: e.to_string() });
        }
    }
    failed
}

/// Runs `action` off the shortcut handler's thread, logging a failure (a hotkey has
/// no UI of its own to report it in).
fn dispatch(app: AppHandle, action: Action) {
    tauri::async_runtime::spawn(async move {
        let result = match action {
            Action::QuickHide => windows_ext::on_main_thread(&app, toggle_overlay).await,
            // The hub opens it exactly like its Ask button: created hidden below the hub
            // if needed, then `promote_overlay`. Rust does not duplicate those options.
            Action::PushToAsk => app
                .emit_to("hub", PUSH_TO_ASK_EVENT, ())
                .map_err(|e| AppError::Audio(e.to_string())),
            Action::ToggleListen => toggle_listen(&app).await,
            Action::ReadScreen => app
                .emit_to("hub", READ_SCREEN_EVENT, ())
                .map_err(|e| AppError::Audio(e.to_string())),
        };
        if let Err(e) = result {
            eprintln!("[hotkeys] {action:?} failed: {e}");
        }
    });
}

/// Quick-hide: hub visible → hide every overlay window (the feature windows first,
/// the hub last, like the hub's Close button); hub hidden → `make_overlay`.
fn toggle_overlay(app: &AppHandle) -> AppResult<()> {
    let hub = app
        .get_webview_window("hub")
        .ok_or_else(|| AppError::Audio("hub window missing".into()))?;
    let visible = hub.is_visible().map_err(|e| AppError::Audio(e.to_string()))?;
    if !visible {
        return windows_ext::make_overlay(&hub);
    }
    for (label, window) in app.webview_windows() {
        if label != "hub" {
            // One window failing must not keep the rest on screen.
            if let Err(e) = window.hide() {
                eprintln!("[hotkeys] could not hide {label:?}: {e}");
            }
        }
    }
    hub.hide().map_err(|e| AppError::Audio(e.to_string()))
}

async fn toggle_listen(app: &AppHandle) -> AppResult<()> {
    let state = app.state::<AppState>();
    let listening = state.listen_session.lock().await.is_some();
    let result = if listening {
        commands::stop_listen(state).await
    } else {
        commands::start_listen(app.clone(), state).await
    };
    // `IpcError` is only `Serialize`; its JSON (`{"kind":"MissingApiKey"}`) is the log line.
    result.map_err(|e| AppError::Audio(serde_json::to_string(&e).unwrap_or_default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use tauri_plugin_global_shortcut::Shortcut;

    #[test]
    fn an_empty_object_yields_the_default_shortcuts() {
        let s: Shortcuts = serde_json::from_str("{}").unwrap();
        assert_eq!(s, Shortcuts::default());
        assert_eq!(s.quick_hide, "cmdOrControl+shift+h");
        assert_eq!(s.push_to_ask, "cmdOrControl+shift+space");
        assert_eq!(s.toggle_listen, "cmdOrControl+shift+l");
        assert_eq!(s.read_screen, "cmdOrControl+shift+s");
    }

    #[test]
    fn a_partial_object_keeps_the_defaults_for_the_missing_keys() {
        let s: Shortcuts = serde_json::from_str(r#"{"quickHide":"alt+h"}"#).unwrap();
        assert_eq!(s.quick_hide, "alt+h");
        assert_eq!(s.push_to_ask, Shortcuts::default().push_to_ask);
    }

    #[test]
    fn serializes_with_camel_case_keys_and_roundtrips() {
        let s = Shortcuts::default();
        let json = serde_json::to_value(&s).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "quickHide": "cmdOrControl+shift+h",
                "pushToAsk": "cmdOrControl+shift+space",
                "toggleListen": "cmdOrControl+shift+l",
                "readScreen": "cmdOrControl+shift+s"
            })
        );
        assert_eq!(serde_json::from_value::<Shortcuts>(json).unwrap(), s);
    }

    /// The defaults must be accelerators the plugin can parse, or a clean install
    /// would start with every shortcut silently missing.
    #[test]
    fn the_defaults_parse_as_accelerators_and_are_distinct() {
        let defaults = Shortcuts::default();
        let parsed: Vec<Shortcut> = defaults
            .bindings()
            .iter()
            .map(|(action, acc)| {
                Shortcut::from_str(acc).unwrap_or_else(|e| panic!("{action:?} = {acc:?}: {e}"))
            })
            .collect();
        let ids: std::collections::BTreeSet<_> = parsed.iter().map(|s| s.id()).collect();
        assert_eq!(ids.len(), parsed.len(), "two actions share an accelerator");
    }

    #[test]
    fn bindings_cover_every_action_once() {
        let actions: Vec<Action> = Shortcuts::default().bindings().iter().map(|(a, _)| *a).collect();
        assert_eq!(
            actions,
            vec![Action::QuickHide, Action::PushToAsk, Action::ToggleListen, Action::ReadScreen]
        );
    }
}
