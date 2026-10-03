//! Browser launch for OAuth login URLs: the platform opener (the TS
//! login dialog's command table — darwin `open`, Windows
//! `rundll32 url.dll,FileProtocolHandler`, otherwise `xdg-open`).

use std::process::{Command, Stdio};

/// The opener program and its argument list for one URL.
fn opener(url: &str) -> (String, Vec<String>) {
    #[cfg(target_os = "macos")]
    {
        ("open".to_string(), vec![url.to_string()])
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        ("xdg-open".to_string(), vec![url.to_string()])
    }
    #[cfg(windows)]
    {
        // Absolute System32 path (the TS dialog resolves it from
        // `SystemRoot`, defaulting to `C:\Windows`). rundll32 takes the
        // first argument as `<dll>,<entrypoint>`: the program itself is
        // the full path, so the argument list must not repeat it
        // (roundll32 would otherwise load itself as a DLL and fail with
        // `Error in ...rundll32.exe / Missing entry:
        // url.dll,FileProtocolHandler`).
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
        let rundll32 = std::path::Path::new(&system_root)
            .join("System32")
            .join("rundll32.exe");
        (
            rundll32.to_string_lossy().into_owned(),
            vec!["url.dll,FileProtocolHandler".to_string(), url.to_string()],
        )
    }
}

/// Open `url` in the user's browser. Fire-and-forget like the TS dialog
/// (`execFileHidden` with a swallowed callback): the caller also shows the
/// URL itself, so a failed launch (no desktop session, no opener) never
/// fails the login. The spawn result is deliberately not an error surface.
pub fn open_in_browser(url: &str) {
    let (program, args) = opener(url);
    let _ = Command::new(program)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opener_selection() {
        // The command table per platform; the program matches the TS dialog.
        // Regression: the program is the absolute rundll32 path and the
        // argument list carries only `<dll>,<entrypoint>` + the URL —
        // repeating the exe path as an argument makes rundll32 load itself
        // as a DLL and show a `Missing entry: url.dll,FileProtocolHandler`
        // dialog instead of the browser.
        let (program, args) = opener("https://example.com/login");
        assert!(!program.is_empty());
        assert!(args.len() == 2);
        assert!(args[0] == "url.dll,FileProtocolHandler");
        assert!(args[1] == "https://example.com/login");
        assert!(!args.iter().any(|arg| arg.contains("rundll32")));
    }
}
