use std::path::Path;

use ort::session::Session;
use ort::value::Value;

use crate::KokoroError;

pub struct KokoroModel {
    session: Session,
}

impl KokoroModel {
    /// Load the ONNX model from a file path.
    pub fn new(model_path: &Path) -> Result<Self, KokoroError> {
        let session = Session::builder()
            .map_err(|e| KokoroError::Model(format!("Failed to create session builder: {}", e)))?
            .with_intra_threads(4)
            .map_err(|e| KokoroError::Model(format!("Failed to set thread count: {}", e)))?
            .commit_from_file(model_path)
            .map_err(|e| {
                KokoroError::Model(format!(
                    "Failed to load ONNX model from {}: {}",
                    model_path.display(),
                    e
                ))
            })?;

        log::info!("Loaded ONNX model from {}", model_path.display());
        Ok(Self { session })
    }

    /// Run inference with the model.
    ///
    /// - `input_ids`: padded token IDs [0, ...tokens, 0]
    /// - `style`: voice embedding, 256 f32 values
    /// - `speed`: speech rate (1.0 = normal)
    ///
    /// Returns audio samples as f32 at 24kHz.
    pub fn infer(
        &mut self,
        input_ids: &[i64],
        style: &[f32],
        speed: f32,
    ) -> Result<Vec<f32>, KokoroError> {
        let seq_len = input_ids.len();

        // Use (shape, data) tuple API to avoid ndarray version mismatches
        let ids_value = Value::from_array(([1, seq_len], input_ids.to_vec()))
            .map_err(|e| KokoroError::Model(format!("Failed to create input_ids tensor: {}", e)))?;

        let style_value = Value::from_array(([1usize, 256], style.to_vec()))
            .map_err(|e| KokoroError::Model(format!("Failed to create style tensor: {}", e)))?;

        let speed_value = Value::from_array(([1usize], vec![speed]))
            .map_err(|e| KokoroError::Model(format!("Failed to create speed tensor: {}", e)))?;

        let outputs = self
            .session
            .run(ort::inputs![ids_value, style_value, speed_value])
            .map_err(|e| KokoroError::Model(format!("ONNX inference failed: {}", e)))?;

        // try_extract_tensor returns (&Shape, &[T])
        let (_shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| KokoroError::Model(format!("Failed to extract output tensor: {}", e)))?;

        Ok(data.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_model() {
        let model_path = Path::new("/tmp/kokoro-inspect/model_quantized.onnx");
        if !model_path.exists() {
            return; // Skip if model not available
        }

        let model = KokoroModel::new(model_path);
        assert!(model.is_ok());
    }
}
