use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Spawn a child process, stream its stdout/stderr line-by-line to the Rust
/// logger (which tauri-plugin-log forwards to the terminal), and return the
/// collected output + exit status.
pub async fn run_and_stream(
    cmd: &mut tokio::process::Command,
    label: &str,
) -> Result<(bool, String), String> {
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", label, e))?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let label_out = label.to_string();
    let label_err = label.to_string();

    let stdout_handle = tokio::spawn(async move {
        let mut lines = Vec::new();
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            log::info!("[{}] {}", label_out, line);
            lines.push(line);
        }
        lines
    });

    let stderr_handle = tokio::spawn(async move {
        let mut lines = Vec::new();
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            log::info!("[{}:err] {}", label_err, line);
            lines.push(line);
        }
        lines
    });

    let status = child
        .wait()
        .await
        .map_err(|e| format!("{} wait failed: {}", label, e))?;

    let out_lines = stdout_handle.await.unwrap_or_default();
    let err_lines = stderr_handle.await.unwrap_or_default();

    let mut combined = out_lines;
    combined.extend(err_lines);
    let output = combined.join("\n").trim().to_string();

    Ok((status.success(), output))
}
