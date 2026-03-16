use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

use super::AppState;
use crate::error::AppError;

/// Parse a user-friendly shortcut string like "Alt+H" into a `Shortcut`.
pub(crate) fn parse_shortcut(s: &str) -> Result<Shortcut, AppError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(AppError::InvalidInput("Empty shortcut".into()));
    }

    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    let mut mods = Modifiers::empty();
    let mut code = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" => mods |= Modifiers::ALT,
            "shift" => mods |= Modifiers::SHIFT,
            "meta" | "super" | "win" | "cmd" => mods |= Modifiers::META,
            key => {
                code = Some(str_to_code(key)?);
            }
        }
    }

    let code = code.ok_or_else(|| AppError::InvalidInput("No key in shortcut".into()))?;
    let mods_opt = if mods.is_empty() { None } else { Some(mods) };
    Ok(Shortcut::new(mods_opt, code))
}

fn str_to_code(s: &str) -> Result<Code, AppError> {
    match s.to_uppercase().as_str() {
        "A" => Ok(Code::KeyA),
        "B" => Ok(Code::KeyB),
        "C" => Ok(Code::KeyC),
        "D" => Ok(Code::KeyD),
        "E" => Ok(Code::KeyE),
        "F" => Ok(Code::KeyF),
        "G" => Ok(Code::KeyG),
        "H" => Ok(Code::KeyH),
        "I" => Ok(Code::KeyI),
        "J" => Ok(Code::KeyJ),
        "K" => Ok(Code::KeyK),
        "L" => Ok(Code::KeyL),
        "M" => Ok(Code::KeyM),
        "N" => Ok(Code::KeyN),
        "O" => Ok(Code::KeyO),
        "P" => Ok(Code::KeyP),
        "Q" => Ok(Code::KeyQ),
        "R" => Ok(Code::KeyR),
        "S" => Ok(Code::KeyS),
        "T" => Ok(Code::KeyT),
        "U" => Ok(Code::KeyU),
        "V" => Ok(Code::KeyV),
        "W" => Ok(Code::KeyW),
        "X" => Ok(Code::KeyX),
        "Y" => Ok(Code::KeyY),
        "Z" => Ok(Code::KeyZ),
        "0" => Ok(Code::Digit0),
        "1" => Ok(Code::Digit1),
        "2" => Ok(Code::Digit2),
        "3" => Ok(Code::Digit3),
        "4" => Ok(Code::Digit4),
        "5" => Ok(Code::Digit5),
        "6" => Ok(Code::Digit6),
        "7" => Ok(Code::Digit7),
        "8" => Ok(Code::Digit8),
        "9" => Ok(Code::Digit9),
        "F1" => Ok(Code::F1),
        "F2" => Ok(Code::F2),
        "F3" => Ok(Code::F3),
        "F4" => Ok(Code::F4),
        "F5" => Ok(Code::F5),
        "F6" => Ok(Code::F6),
        "F7" => Ok(Code::F7),
        "F8" => Ok(Code::F8),
        "F9" => Ok(Code::F9),
        "F10" => Ok(Code::F10),
        "F11" => Ok(Code::F11),
        "F12" => Ok(Code::F12),
        other => Err(AppError::InvalidInput(format!("Unknown key: {}", other))),
    }
}

/// Register the global toggle-window shortcut from config.
/// Silently ignores empty or invalid shortcuts.
pub(crate) fn register_toggle_shortcut(app: &AppHandle) {
    let shortcut_str = {
        let state = app.state::<AppState>();
        let config = state.config.lock().unwrap();
        config.shortcut_toggle_window.clone()
    };

    if shortcut_str.is_empty() {
        return;
    }

    if let Ok(shortcut) = parse_shortcut(&shortcut_str) {
        let _ = app.global_shortcut().register(shortcut);
    }
}

#[tauri::command]
pub fn update_global_shortcut(
    app: AppHandle,
    old_shortcut: String,
    new_shortcut: String,
) -> Result<(), AppError> {
    // Unregister old shortcut
    if !old_shortcut.is_empty() {
        if let Ok(old) = parse_shortcut(&old_shortcut) {
            let _ = app.global_shortcut().unregister(old);
        }
    }

    // Register new shortcut
    if !new_shortcut.is_empty() {
        let new = parse_shortcut(&new_shortcut).map_err(|_| {
            AppError::InvalidInput(format!("Invalid shortcut: {}", new_shortcut))
        })?;
        app.global_shortcut()
            .register(new)
            .map_err(|e| AppError::InvalidInput(format!("Failed to register shortcut: {}", e)))?;
    }

    Ok(())
}
