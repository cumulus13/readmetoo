use anyhow::Result;

/// Display content using the best available pager strategy.
///
/// Strategy order:
///  1. If `$PAGER` env var is set → use it as an external command
///  2. Try `less -R` (Unix) or `more` (Windows) as external command
///  3. Fall back to the built-in minus static pager
///  4. Last resort: just print to stdout
pub fn page_content(content: &str, title: Option<&str>) -> Result<()> {
    // 1. Honour $PAGER env var (but not if it's set to the binary itself)
    if let Ok(pager_cmd) = std::env::var("PAGER") {
        if !pager_cmd.trim().is_empty() && external_pager(content, &pager_cmd).is_ok() {
            return Ok(());
        }
    }

    // 2. Try platform-native pager
    #[cfg(windows)]
    let native = &["more"];
    #[cfg(not(windows))]
    let native = &["less -R", "less", "more"];

    for cmd in native {
        if external_pager(content, cmd).is_ok() {
            return Ok(());
        }
    }

    // 3. Built-in minus pager
    if try_minus_pager(content, title).is_ok() {
        return Ok(());
    }

    // 4. Plain stdout fallback
    use std::io::Write;
    std::io::stdout().lock().write_all(content.as_bytes())?;
    Ok(())
}

/// Try the minus built-in static pager. Returns Err if minus isn't available or fails.
fn try_minus_pager(content: &str, title: Option<&str>) -> Result<()> {
    let pager = minus::Pager::new();
    if let Some(t) = title {
        pager.set_prompt(t).ok();
    }
    pager.push_str(content).ok();
    minus::page_all(pager).map_err(|e| anyhow::anyhow!("minus pager error: {}", e))?;
    Ok(())
}

/// Pipe content to an external pager command (e.g. "less -R", "more").
pub fn external_pager(content: &str, cmd: &str) -> Result<()> {
    use std::io::Write as IoWrite;
    use std::process::{Command, Stdio};

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty pager command"));
    }

    let mut child = Command::new(parts[0])
        .args(&parts[1..])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Cannot launch '{}': {}", parts[0], e))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| anyhow::anyhow!("Pager stdin write error: {}", e))?;
    }
    // drop stdin so the pager sees EOF
    drop(child.stdin.take());

    let status = child
        .wait()
        .map_err(|e| anyhow::anyhow!("Pager wait error: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Pager '{}' exited with status {}",
            parts[0],
            status
        ))
    }
}
