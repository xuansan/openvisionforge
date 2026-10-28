use vision_core::{Classification, Result, VisionError};

use crate::backend::onnx::OnnxRunOutput;

pub(crate) fn decode(
    output: &OnnxRunOutput,
    labels: &[String],
    score_threshold: f64,
    top_k: usize,
) -> Result<Vec<Classification>> {
    let tensor = output
        .tensors
        .first()
        .ok_or_else(|| error("model produced no output tensors"))?;
    let class_count = match tensor.shape.as_slice() {
        [count] => *count as usize,
        [1, count] => *count as usize,
        shape => {
            return Err(error(format!(
            "unsupported classification output shape {shape:?}; expected [classes] or [1, classes]"
        )))
        }
    };
    if class_count == 0 || tensor.data.len() < class_count {
        return Err(error("classification output tensor is empty or truncated"));
    }

    let scores = normalize_scores(&tensor.data[..class_count]);
    let mut ranked: Vec<(usize, f64)> = scores
        .into_iter()
        .enumerate()
        .filter(|(_, score)| *score >= score_threshold)
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked.truncate(top_k.max(1));

    Ok(ranked
        .into_iter()
        .enumerate()
        .map(|(rank, (index, score))| Classification {
            label: labels
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("class_{index}")),
            score,
            rank: rank as u32 + 1,
        })
        .collect())
}

fn normalize_scores(values: &[f32]) -> Vec<f64> {
    if values.iter().all(|value| (0.0..=1.0).contains(value)) {
        return values.iter().map(|value| *value as f64).collect();
    }
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exp: Vec<f64> = values
        .iter()
        .map(|value| (*value - max).exp() as f64)
        .collect();
    let sum: f64 = exp.iter().sum();
    exp.into_iter().map(|value| value / sum).collect()
}

fn error(message: impl Into<String>) -> VisionError {
    VisionError::Operator {
        node: "infer.classification.decode".to_string(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::onnx::{OnnxRunOutput, TensorOutput};

    #[test]
    fn ranks_real_classification_scores() {
        let output = OnnxRunOutput {
            tensors: vec![TensorOutput {
                name: "output0".to_string(),
                shape: vec![1, 3],
                data: vec![0.1, 0.8, 0.4],
            }],
            metadata: serde_json::Value::Null,
        };
        let classes = decode(
            &output,
            &["ok".into(), "scratch".into(), "dent".into()],
            0.2,
            2,
        )
        .unwrap();
        assert_eq!(classes[0].label, "scratch");
        assert_eq!(classes[1].label, "dent");
    }
}
