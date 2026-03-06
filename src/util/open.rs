use std::process::Command;

pub fn open_url(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("Empty URL".to_string());
    }

    // Validate URL scheme to prevent command injection via malicious URLs
    match url::Url::parse(trimmed) {
        Ok(parsed) if matches!(parsed.scheme(), "http" | "https") => {}
        Ok(parsed) => return Err(format!("Unsupported URL scheme: {}", parsed.scheme())),
        Err(e) => return Err(format!("Invalid URL: {e}")),
    }

    let status = if cfg!(target_os = "macos") {
        Command::new("open").arg(trimmed).status()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", "", trimmed])
            .status()
    } else {
        Command::new("xdg-open").arg(trimmed).status()
    };

    match status {
        Ok(result) if result.success() => Ok(()),
        Ok(result) => Err(format!("Open failed: {result}")),
        Err(error) => Err(format!("Open failed: {error}")),
    }
}
