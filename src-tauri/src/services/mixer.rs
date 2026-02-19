use tokio::process::Command;

pub struct AudioMixer;

impl AudioMixer {
    /// Concatenate multiple WAV files into one
    pub async fn concat_wav(input_paths: &[&str], output_path: &str) -> Result<String, String> {
        // Create a temporary file list for ffmpeg concat
        let file_list: String = input_paths
            .iter()
            .map(|p| format!("file '{}'", p))
            .collect::<Vec<_>>()
            .join("\n");

        let list_path = format!("{}.txt", output_path);
        tokio::fs::write(&list_path, &file_list)
            .await
            .map_err(|e| format!("Failed to write concat list: {}", e))?;

        let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                &list_path,
                "-c",
                "copy",
                output_path,
            ])
            .output()
            .await
            .map_err(|e| format!("Failed to run ffmpeg concat: {}", e))?;

        // Clean up temp file
        let _ = tokio::fs::remove_file(&list_path).await;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ffmpeg concat failed: {}", stderr));
        }

        Ok(output_path.to_string())
    }
}
