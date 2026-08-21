use log::warn;
use std::process::Command;

// Whether the current session is a Wayland session.
#[cfg(target_os = "linux")]
pub fn is_wayland_session() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some()
}

#[cfg(not(target_os = "linux"))]
pub fn is_wayland_session() -> bool {
    false
}

// Get the text currently selected by the mouse (or copied to the clipboard).
//
// On Wayland the X11 based `selection` crate cannot read the selection of
// native Wayland apps, so we use `wl-paste` to read the PRIMARY selection
// (the one populated by a mouse drag), falling back to the CLIPBOARD.
pub fn get_selected_text() -> String {
    if is_wayland_session() {
        let primary = read_wayland_selection(true);
        if !primary.trim().is_empty() {
            return primary;
        }
        // Fallback: clipboard, e.g. text copied with Ctrl+C
        return read_wayland_selection(false);
    }
    selection::get_text()
}

// Read the Wayland selection with `wl-paste`.
//
// `wl-paste` grabs keyboard focus with an invisible surface to receive the
// selection, which can hang (e.g. while an always-on-top window is open), so it
// is wrapped in `timeout` to bound the wait.
fn read_wayland_selection(primary: bool) -> String {
    let mut command = Command::new("timeout");
    command
        .arg("-k")
        .arg("1")
        .arg("3")
        .arg("wl-paste")
        .arg("--no-newline");
    if primary {
        command.arg("--primary");
    }
    match command.output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(e) => {
            warn!("Failed to run wl-paste: {}", e);
            String::new()
        }
    }
}
