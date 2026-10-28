use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Mutex, OnceLock},
    time::SystemTime,
};

use sha2::{Digest, Sha256};
use vision_core::{Result, VisionError};

const DEFAULT_CATALOG: &str = "models/registry.json";

#[derive(Debug, Clone)]
pub(crate) struct ResolvedModel {
    pub path: String,
    pub input_width: Option<i64>,
    pub input_height: Option<i64>,
    pub labels: Vec<String>,
}

pub(crate) fn resolve(model_id: &str, model_path: &str, task: &str) -> Result<ResolvedModel> {
    if !model_path.trim().is_empty() {
        return Ok(ResolvedModel {
            path: require_model_file(model_path)?,
            input_width: None,
            input_height: None,
            labels: Vec::new(),
        });
    }
    if model_id.trim().is_empty() {
        return Err(error("configure either model_id or model_path"));
    }

    let catalog_path = std::env::var("OPENVISIONFORGE_MODEL_CATALOG")
        .unwrap_or_else(|_| DEFAULT_CATALOG.to_string());
    let content = fs::read_to_string(&catalog_path).map_err(|err| {
        error(format!(
            "failed to read model catalog {catalog_path}: {err}"
        ))
    })?;
    let catalog: serde_json::Value = serde_json::from_str(&content)
        .map_err(|err| error(format!("invalid model catalog {catalog_path}: {err}")))?;
    let entry = catalog
        .get("models")
        .and_then(|models| models.get(model_id))
        .ok_or_else(|| error(format!("model_id '{model_id}' is not in {catalog_path}")))?;

    validate_entry(model_id, entry, task)?;
    let path = entry
        .get("path")
        .and_then(|value| value.as_str())
        .ok_or_else(|| error(format!("model_id '{model_id}' has no path")))?;
    let path = require_model_file(path)?;
    validate_integrity(model_id, &path, entry)?;

    Ok(ResolvedModel {
        path,
        input_width: entry.get("input_width").and_then(|value| value.as_i64()),
        input_height: entry.get("input_height").and_then(|value| value.as_i64()),
        labels: load_labels(entry)?,
    })
}

fn validate_entry(model_id: &str, entry: &serde_json::Value, task: &str) -> Result<()> {
    let status = entry
        .get("status")
        .and_then(|value| value.as_str())
        .unwrap_or("unverified");
    if status != "ready" {
        return Err(error(format!(
            "model_id '{model_id}' is not ready (status={status})"
        )));
    }
    let model_task = entry
        .get("task")
        .and_then(|value| value.as_str())
        .ok_or_else(|| error(format!("model_id '{model_id}' has no task")))?;
    if !task.is_empty() && model_task != task {
        return Err(error(format!(
            "model_id '{model_id}' task is '{model_task}', but node requires '{task}'"
        )));
    }
    Ok(())
}

fn validate_integrity(model_id: &str, path: &str, entry: &serde_json::Value) -> Result<()> {
    let metadata = fs::metadata(path)
        .map_err(|err| error(format!("failed to inspect model file {path}: {err}")))?;
    if let Some(expected) = entry.get("size_bytes").and_then(|value| value.as_u64()) {
        if metadata.len() != expected {
            return Err(error(format!(
                "model_id '{model_id}' size mismatch: expected {expected}, got {}",
                metadata.len()
            )));
        }
    }
    if let Some(expected) = entry.get("sha256").and_then(|value| value.as_str()) {
        let actual = cached_sha256(path, metadata.len(), metadata.modified().ok())?;
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(error(format!(
                "model_id '{model_id}' SHA256 mismatch: expected {expected}, got {actual}"
            )));
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct HashCacheEntry {
    size: u64,
    modified: Option<SystemTime>,
    sha256: String,
}

static HASH_CACHE: OnceLock<Mutex<HashMap<String, HashCacheEntry>>> = OnceLock::new();

fn cached_sha256(path: &str, size: u64, modified: Option<SystemTime>) -> Result<String> {
    let canonical = Path::new(path)
        .canonicalize()
        .map_err(|err| error(format!("failed to canonicalize model file {path}: {err}")))?;
    let key = canonical.to_string_lossy().into_owned();
    let cache = HASH_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(entry) = cache
        .lock()
        .map_err(|_| error("model hash cache lock is poisoned"))?
        .get(&key)
        .filter(|entry| entry.size == size && entry.modified == modified)
        .cloned()
    {
        return Ok(entry.sha256);
    }

    let hash = sha256(path)?;
    cache
        .lock()
        .map_err(|_| error("model hash cache lock is poisoned"))?
        .insert(
            key,
            HashCacheEntry {
                size,
                modified,
                sha256: hash.clone(),
            },
        );
    Ok(hash)
}

fn load_labels(entry: &serde_json::Value) -> Result<Vec<String>> {
    if let Some(labels) = entry.get("labels").and_then(|value| value.as_array()) {
        return Ok(labels
            .iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect());
    }
    let Some(path) = entry.get("labels_path").and_then(|value| value.as_str()) else {
        return Ok(Vec::new());
    };
    let content = fs::read_to_string(path)
        .map_err(|err| error(format!("failed to read labels file {path}: {err}")))?;
    serde_json::from_str::<Vec<String>>(&content)
        .map_err(|err| error(format!("invalid labels file {path}: {err}")))
}

fn sha256(path: &str) -> Result<String> {
    let bytes =
        fs::read(path).map_err(|err| error(format!("failed to hash model file {path}: {err}")))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn require_model_file(path: &str) -> Result<String> {
    let path = Path::new(path);
    if !path.is_file() {
        return Err(error(format!(
            "model file does not exist: {}",
            path.display()
        )));
    }
    Ok(path.to_string_lossy().into_owned())
}

fn error(message: impl Into<String>) -> VisionError {
    VisionError::Operator {
        node: "infer.model_assets".to_string(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use super::resolve;

    #[test]
    fn explicit_model_path_must_exist() {
        let error = resolve("", "missing-model.onnx", "detect").unwrap_err();
        assert!(error.to_string().contains("model file does not exist"));
    }

    #[test]
    fn requires_id_or_path() {
        let error = resolve("", "", "detect").unwrap_err();
        assert!(error.to_string().contains("model_id or model_path"));
    }

    #[test]
    fn validates_catalog_status_task_size_and_hash() {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("openvisionforge-model-{suffix}"));
        fs::create_dir_all(&root).unwrap();
        let model = root.join("model.onnx");
        let labels = root.join("labels.json");
        let catalog = root.join("registry.json");
        fs::write(&model, b"abc").unwrap();
        fs::write(&labels, "[\"ok\",\"defect\"]").unwrap();
        fs::write(
            &catalog,
            serde_json::json!({
                "models": {"test": {
                    "status": "ready",
                    "task": "detect",
                    "path": model,
                    "size_bytes": 3,
                    "sha256": "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
                    "input_width": 320,
                    "input_height": 192,
                    "labels_path": labels
                }}
            })
            .to_string(),
        )
        .unwrap();
        std::env::set_var("OPENVISIONFORGE_MODEL_CATALOG", &catalog);
        let resolved = resolve("test", "", "detect").unwrap();
        std::env::remove_var("OPENVISIONFORGE_MODEL_CATALOG");
        assert_eq!(resolved.input_width, Some(320));
        assert_eq!(resolved.input_height, Some(192));
        assert_eq!(resolved.labels, ["ok", "defect"]);
        std::thread::sleep(std::time::Duration::from_millis(20));
        fs::write(&model, b"abd").unwrap();
        std::env::set_var("OPENVISIONFORGE_MODEL_CATALOG", &catalog);
        let error = resolve("test", "", "detect").unwrap_err();
        std::env::remove_var("OPENVISIONFORGE_MODEL_CATALOG");
        assert!(error.to_string().contains("SHA256 mismatch"));
        fs::remove_dir_all(root).unwrap();
    }
}
