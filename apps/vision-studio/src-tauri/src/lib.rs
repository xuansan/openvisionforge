use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use vision_core::{
    validate_workflow as validate_workflow_core, Executor, RunEvent, RunTrace, Workflow,
};

#[derive(Debug, Serialize)]
struct FileEntry {
    name: String,
    path: String,
    size: u64,
}

#[derive(Debug, Serialize)]
struct RecordEntry {
    run_id: String,
    workflow_id: String,
    result: String,
    status: String,
    total_duration_ms: f64,
    fps: f64,
    path: String,
}

#[derive(Debug, Serialize)]
struct NodeSmokeStatus {
    total: usize,
    pass: usize,
    fail: usize,
    source: Option<String>,
    operators: Vec<NodeSmokeOperatorStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeSmokeOperatorStatus {
    id: String,
    category: String,
    name: String,
    status: String,
    exit_code: i32,
    ms: f64,
    message: String,
}

#[derive(Debug, Serialize)]
struct RunResponse {
    event: RunEvent,
    trace: RunTrace,
    record_dir: String,
    artifacts: Vec<FileEntry>,
}

#[derive(Debug, Serialize)]
struct ArtifactContent {
    path: String,
    kind: String,
    text: Option<String>,
    data_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowUiMetadata {
    workflow_id: String,
    node_positions: serde_json::Value,
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("src-tauri lives under apps/vision-studio")
        .to_path_buf()
}

fn relative_string(path: &Path) -> String {
    path.strip_prefix(project_root())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("parse {}: {err}", path.display()))
}

fn list_files(dir: &Path, extension: &str) -> Vec<FileEntry> {
    let mut files = fs::read_dir(dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| {
            let path = entry.path();
            let meta = entry.metadata().ok()?;
            if !meta.is_file() {
                return None;
            }
            let ext = path.extension()?.to_string_lossy().to_ascii_lowercase();
            if ext != extension {
                return None;
            }
            Some(FileEntry {
                name: path.file_name()?.to_string_lossy().to_string(),
                path: relative_string(&path),
                size: meta.len(),
            })
        })
        .collect::<Vec<_>>();
    files.sort_by(|a, b| a.name.cmp(&b.name));
    files
}

fn list_workflow_files(dir: &Path) -> Vec<FileEntry> {
    let mut files = list_files(dir, "json")
        .into_iter()
        .filter(|file| !file.name.ends_with(".ui.json"))
        .collect::<Vec<_>>();
    files.sort_by(|a, b| a.name.cmp(&b.name));
    files
}

fn workflow_ui_metadata_path(workflow_path: &Path) -> PathBuf {
    let stem = workflow_path
        .file_stem()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "workflow".to_string());
    workflow_path.with_file_name(format!("{stem}.ui.json"))
}

fn list_record_artifacts(record_dir: &Path) -> Vec<FileEntry> {
    let mut artifacts = Vec::new();
    let mut stack = vec![record_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_dir() {
                stack.push(path);
            } else if path
                .file_name()
                .is_some_and(|name| name != "trace.json" && name != "event.json")
            {
                artifacts.push(FileEntry {
                    name: path
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    path: relative_string(&path),
                    size: meta.len(),
                });
            }
        }
    }
    artifacts.sort_by(|a, b| a.path.cmp(&b.path));
    artifacts
}

fn workflow_for_execution(workflow: &Workflow) -> Workflow {
    workflow.clone()
}

fn project_file(path: &str) -> Result<PathBuf, String> {
    let root = project_root();
    let full = root.join(path);
    let canonical = full
        .canonicalize()
        .map_err(|err| format!("open {}: {err}", full.display()))?;
    let canonical_root = root
        .canonicalize()
        .map_err(|err| format!("open project root {}: {err}", root.display()))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!("path is outside project root: {path}"));
    }
    Ok(canonical)
}

fn mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/png",
    }
}

fn is_image_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp"
    )
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[tauri::command]
fn export_schema() -> Vec<serde_json::Value> {
    vision_runtime::export_schema()
}

#[tauri::command]
fn runtime_status() -> serde_json::Value {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "schema_count": vision_runtime::export_schema().len(),
        "runtime": "vision-runtime",
        "project_root": relative_string(&project_root()),
    })
}

#[tauri::command]
fn list_examples() -> Vec<FileEntry> {
    list_workflow_files(&project_root().join("examples").join("workflows"))
}

#[tauri::command]
fn list_studio_workflows() -> Vec<FileEntry> {
    list_workflow_files(&project_root().join("studio_workflows"))
}

#[tauri::command]
fn load_workflow(path: String) -> Result<Workflow, String> {
    read_json_file(&project_root().join(path))
}

#[tauri::command]
fn load_workflow_ui_metadata(path: String) -> Result<Option<WorkflowUiMetadata>, String> {
    let workflow_path = project_root().join(path);
    let metadata_path = workflow_ui_metadata_path(&workflow_path);
    if !metadata_path.exists() {
        return Ok(None);
    }
    read_json_file(&metadata_path).map(Some)
}

#[tauri::command]
fn save_workflow(
    workflow: Workflow,
    ui_metadata: Option<WorkflowUiMetadata>,
) -> Result<FileEntry, String> {
    let dir = project_root().join("studio_workflows");
    fs::create_dir_all(&dir).map_err(|err| format!("create {}: {err}", dir.display()))?;
    let file_name = format!("{}.json", workflow.id);
    let path = dir.join(file_name);
    let data = serde_json::to_vec_pretty(&workflow).map_err(|err| err.to_string())?;
    fs::write(&path, data).map_err(|err| format!("write {}: {err}", path.display()))?;
    if let Some(metadata) = ui_metadata {
        let metadata_path = workflow_ui_metadata_path(&path);
        let metadata_data = serde_json::to_vec_pretty(&metadata).map_err(|err| err.to_string())?;
        fs::write(&metadata_path, metadata_data)
            .map_err(|err| format!("write {}: {err}", metadata_path.display()))?;
    }
    let meta = fs::metadata(&path).map_err(|err| err.to_string())?;
    Ok(FileEntry {
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default(),
        path: relative_string(&path),
        size: meta.len(),
    })
}

#[tauri::command]
fn validate_workflow(workflow: Workflow) -> serde_json::Value {
    let registry = vision_runtime::registry();
    let report = validate_workflow_core(&workflow, &registry);
    serde_json::json!({
        "ok": report.ok,
        "nodes": report.nodes,
        "edges": report.edges,
        "message": if report.ok { "workflow schema check passed".to_string() } else { format!("workflow validation failed: {} issue(s)", report.issues.len()) },
        "issues": report.issues,
    })
}

#[tauri::command]
fn export_workflow_json(workflow: Workflow) -> Result<String, String> {
    serde_json::to_string_pretty(&workflow).map_err(|err| err.to_string())
}

#[tauri::command]
fn run_workflow(workflow: Workflow) -> Result<RunResponse, String> {
    let registry = vision_runtime::registry();
    let execution_workflow = workflow_for_execution(&workflow);
    let root = project_root();
    let previous_dir = env::current_dir().map_err(|err| format!("read current dir: {err}"))?;
    env::set_current_dir(&root)
        .map_err(|err| format!("set current dir {}: {err}", root.display()))?;
    let run_result = Executor::new(&registry).run(&execution_workflow, root.join("records"));
    let restore_result = env::set_current_dir(&previous_dir);
    if let Err(err) = restore_result {
        return Err(format!(
            "restore current dir {}: {err}",
            previous_dir.display()
        ));
    }
    let trace = run_result.map_err(|err| err.to_string())?;
    let record_dir = root.join("records").join(&trace.run_id);
    let event = read_json_file::<RunEvent>(&record_dir.join("event.json"))?;
    let artifacts = list_record_artifacts(&record_dir);
    Ok(RunResponse {
        event,
        trace,
        record_dir: relative_string(&record_dir),
        artifacts,
    })
}

#[tauri::command]
fn list_records() -> Vec<RecordEntry> {
    let records_dir = project_root().join("records");
    let mut records = fs::read_dir(&records_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            let event = read_json_file::<RunEvent>(&path.join("event.json")).ok()?;
            Some(RecordEntry {
                run_id: event.run_id,
                workflow_id: event.workflow_id,
                result: event.result,
                status: event.status,
                total_duration_ms: event.metrics.total_duration_ms,
                fps: event.metrics.fps,
                path: relative_string(&path),
            })
        })
        .collect::<Vec<_>>();
    records.sort_by(|a, b| b.run_id.cmp(&a.run_id));
    records
}

#[tauri::command]
fn node_smoke_status() -> NodeSmokeStatus {
    let records_dir = project_root().join("records");
    let best = fs::read_dir(&records_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| {
            let path = entry.path().join("node_smoke_results.json");
            let modified = fs::metadata(&path).ok()?.modified().ok()?;
            let operators = read_json_file::<Vec<NodeSmokeOperatorStatus>>(&path).ok()?;
            Some((operators.len(), modified, path, operators))
        })
        .max_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let Some((_, _, path, operators)) = best else {
        return NodeSmokeStatus {
            total: 0,
            pass: 0,
            fail: 0,
            source: None,
            operators: Vec::new(),
        };
    };

    let pass = operators
        .iter()
        .filter(|item| item.status.eq_ignore_ascii_case("PASS"))
        .count();
    let fail = operators
        .iter()
        .filter(|item| item.status.eq_ignore_ascii_case("FAIL"))
        .count();
    NodeSmokeStatus {
        total: operators.len(),
        pass,
        fail,
        source: Some(relative_string(&path)),
        operators,
    }
}

#[tauri::command]
fn read_trace(run_id: String) -> Result<RunTrace, String> {
    read_json_file(
        &project_root()
            .join("records")
            .join(run_id)
            .join("trace.json"),
    )
}

#[tauri::command]
fn read_artifact(path: String) -> Result<ArtifactContent, String> {
    let full = project_file(&path)?;
    if is_image_file(&full) {
        let bytes = fs::read(&full).map_err(|err| format!("read {}: {err}", full.display()))?;
        let data_url = format!("data:{};base64,{}", mime_type(&full), base64_encode(&bytes));
        return Ok(ArtifactContent {
            path,
            kind: "image".to_string(),
            text: None,
            data_url: Some(data_url),
        });
    }
    let text =
        fs::read_to_string(&full).map_err(|err| format!("read {}: {err}", full.display()))?;
    Ok(ArtifactContent {
        path,
        kind: "text".to_string(),
        text: Some(text),
        data_url: None,
    })
}

#[tauri::command]
fn list_models() -> Vec<FileEntry> {
    list_files(&project_root().join("models"), "onnx")
}

#[tauri::command]
fn model_catalog() -> Result<serde_json::Value, String> {
    read_json_file(&project_root().join("models").join("registry.json"))
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            export_schema,
            runtime_status,
            list_examples,
            list_studio_workflows,
            load_workflow,
            load_workflow_ui_metadata,
            save_workflow,
            validate_workflow,
            export_workflow_json,
            run_workflow,
            list_records,
            node_smoke_status,
            read_trace,
            read_artifact,
            list_models,
            model_catalog,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Vision Studio");
}
