use tokio::process::Command;

pub struct AudioMixer;

impl AudioMixer {
    /// Mix voice audio with background music using ffmpeg
    /// Voice at full volume, music at 15% volume
    pub async fn mix(
        voice_path: &str,
        music_path: &str,
        output_path: &str,
    ) -> Result<String, String> {
        let args = Self::build_ffmpeg_args(voice_path, music_path, output_path);

        let output = Command::new("ffmpeg")
            .args(&args)
            .output()
            .await
            .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ffmpeg mixing failed: {}", stderr));
        }

        Ok(output_path.to_string())
    }

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

    /// Build ffmpeg arguments for mixing voice and music
    pub fn build_ffmpeg_args(voice_path: &str, music_path: &str, output_path: &str) -> Vec<String> {
        vec![
            "-y".to_string(),
            "-i".to_string(),
            voice_path.to_string(),
            "-i".to_string(),
            music_path.to_string(),
            "-filter_complex".to_string(),
            "[1:a]volume=0.15[music];[0:a][music]amix=inputs=2:duration=first:dropout_transition=2[out]"
                .to_string(),
            "-map".to_string(),
            "[out]".to_string(),
            "-codec:a".to_string(),
            "libmp3lame".to_string(),
            "-q:a".to_string(),
            "2".to_string(),
            output_path.to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffmpeg_args_contain_filter_complex() {
        let args = AudioMixer::build_ffmpeg_args("/voice.wav", "/music.wav", "/output.mp3");
        assert!(args.contains(&"-filter_complex".to_string()));
        let filter_idx = args.iter().position(|a| a == "-filter_complex").unwrap();
        let filter = &args[filter_idx + 1];
        assert!(filter.contains("volume=0.15"));
        assert!(filter.contains("amix"));
    }

    #[test]
    fn ffmpeg_args_output_is_mp3() {
        let args = AudioMixer::build_ffmpeg_args("/voice.wav", "/music.wav", "/out.mp3");
        assert!(args.contains(&"libmp3lame".to_string()));
        assert_eq!(args.last().unwrap(), "/out.mp3");
    }
}
