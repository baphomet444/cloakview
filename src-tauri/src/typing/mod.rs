//! Type at cursor (⌃⌥T): read the screen, look at the focused field, ask the
//! model for text that fits it, and insert it. See
//! `docs/superpowers/specs/2026-11-19-type-at-cursor-design.md`.

pub mod logic;
pub mod mac;

use crate::error::{AppError, AppResult};
use crate::events::ERROR_EVENT;
use crate::state::AppState;
use logic::{Decision, Focus, Outcome, TYPE_DELAY_MS};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// The abort flag of the run in progress, if any: one typist at a time.
static RUNNING: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

const CANCEL_KEY: &str = "Escape";

const ACCESSIBILITY_HELP: &str = "CloakView needs the Accessibility permission to type for you. Allow it in \
System Settings → Privacy & Security → Accessibility, then press the shortcut again.";

/// What the user is told: the plain message, without the error-kind prefix.
fn user_message(e: &AppError) -> String {
    match e {
        AppError::Audio(m) => m.clone(),
        other => other.to_string(),
    }
}

/// Starts a run, or cancels the one in progress (pressing the shortcut again).
/// Failures reach the user through `ERROR_EVENT` and the diagnostics log.
pub async fn run(app: AppHandle) {
    let abort = {
        let mut running = RUNNING.lock().expect("typing lock");
        if let Some(flag) = running.as_ref() {
            flag.store(true, Ordering::SeqCst);
            crate::diag::log("type at cursor: cancelled by a second press");
            return;
        }
        let flag = Arc::new(AtomicBool::new(false));
        *running = Some(flag.clone());
        flag
    };
    let result = run_inner(&app, &abort).await;
    *RUNNING.lock().expect("typing lock") = None;
    if let Err(e) = result {
        let message = user_message(&e);
        crate::diag::log(&format!("type at cursor failed: {message}"));
        let _ = app.emit(ERROR_EVENT, message);
    }
}

async fn run_inner(app: &AppHandle, abort: &Arc<AtomicBool>) -> AppResult<()> {
    let state = app.state::<AppState>();
    // Same gates as the screen read: Pro, and a provider with a key.
    state.license.read().expect("license lock").require_pro()?;
    let (provider, about_user) = {
        let settings = state.settings.read().expect("settings lock");
        let provider = crate::providers::resolve_active(&settings)?;
        let about = Some(settings.personal_context.clone()).filter(|c| !c.trim().is_empty());
        (provider, about)
    };
    if !mac::accessibility_trusted(true) {
        return Err(AppError::Audio(ACCESSIBILITY_HELP.into()));
    }
    let focus = join(tokio::task::spawn_blocking(mac::probe_focus).await)?;
    if let Decision::Refuse(reason) = logic::decide(&focus) {
        return Err(AppError::Audio(reason.into()));
    }
    let expected_pid = match &focus {
        Focus::Editable { pid, .. } => Some(*pid),
        Focus::Inconclusive { pid, .. } => *pid,
        Focus::Secure | Focus::NotEditable => None,
    };
    let image = join(tokio::task::spawn_blocking(crate::screen::capture).await)??;
    let raw = crate::answers::client::type_ready(
        &provider,
        &image,
        &logic::describe(&focus),
        state.is_pro(),
        about_user.as_deref(),
    )
    .await?;
    if abort.load(Ordering::SeqCst) {
        return Ok(()); // cancelled while the model was answering
    }
    let ready = logic::parse_type_ready(&raw);
    let chars = ready.text.chars().count();

    // Esc cancels, but only while we are typing: it is never held otherwise.
    let gs = app.global_shortcut();
    let flag = abort.clone();
    let esc = gs.on_shortcut(CANCEL_KEY, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            flag.store(true, Ordering::SeqCst);
        }
    });
    if let Err(e) = &esc {
        crate::diag::log(&format!("type at cursor: Esc not registered: {e}"));
    }
    let a = abort.clone();
    let outcome = tokio::task::spawn_blocking(move || {
        logic::insert(&mac::MacKeyboard, &ready, expected_pid, &a, Duration::from_millis(TYPE_DELAY_MS))
    })
    .await;
    if esc.is_ok() {
        let _ = gs.unregister(CANCEL_KEY);
    }
    match join(outcome)?? {
        Outcome::Done => crate::diag::log(&format!("type at cursor: typed {chars} chars")),
        Outcome::Aborted => crate::diag::log("type at cursor: stopped before finishing"),
    }
    Ok(())
}

fn join<T>(r: Result<T, tokio::task::JoinError>) -> AppResult<T> {
    r.map_err(|e| AppError::Audio(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_message_drops_the_error_prefix_for_audio_errors_only() {
        assert_eq!(user_message(&AppError::Audio("nothing to type".into())), "nothing to type");
        assert_eq!(user_message(&AppError::ProRequired), "this feature requires CloakView Pro");
    }
}
