use std::{
    collections::BTreeMap,
    env,
    ffi::OsString,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex, OnceLock},
    thread,
};

use vision_core::{ImageFrame, NodeContext, Result, Value, VisionError};

const BRIDGE_EXE: &str = env!("OPENCV_BRIDGE_EXE");
static BRIDGE_WORKER: OnceLock<Mutex<Option<BridgeWorker>>> = OnceLock::new();

pub(crate) fn bridge_image(args: &[&str]) -> Result<ImageFrame> {
    let value = bridge_json(args)?;
    Ok(ImageFrame {
        width: value.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        height: value.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        channels: value.get("channels").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
        pixel_format: value
            .get("pixel_format")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string(),
        source_uri: value
            .get("source_uri")
            .and_then(|v| v.as_str())
            .map(str::to_string),
    })
}

pub(crate) fn bridge_json(args: &[&str]) -> Result<serde_json::Value> {
    let worker = BRIDGE_WORKER.get_or_init(|| Mutex::new(None));
    let mut guard = worker.lock().map_err(|err| VisionError::Operator {
        node: "opencv".to_string(),
        message: format!("OpenCV worker lock failed: {err}"),
    })?;
    if guard.is_none() {
        *guard = Some(BridgeWorker::start()?);
    }
    let value = match guard.as_mut().expect("worker exists").request(args) {
        Ok(value) => value,
        Err(err) => {
            *guard = None;
            return Err(err);
        }
    };
    if let Some(message) = value.get("error").and_then(|v| v.as_str()) {
        return fail(format!("OpenCV bridge failed: {message}"));
    }
    Ok(value)
}

pub(crate) fn image(ctx: &NodeContext<'_>, port: &str) -> Result<ImageFrame> {
    match ctx.inputs.get(port) {
        Some(Value::Image(image)) => Ok(image.clone()),
        _ => fail(format!("expected image input on port {port}")),
    }
}

pub(crate) fn image_path(image: &ImageFrame) -> Result<String> {
    image
        .source_uri
        .clone()
        .filter(|path| !path.is_empty())
        .ok_or_else(|| VisionError::Operator {
            node: "opencv".to_string(),
            message: "image source_uri is required for OpenCV backend".to_string(),
        })
}

pub(crate) fn output_path(ctx: &NodeContext<'_>, key: &str, default_name: &str) -> String {
    ctx.node
        .params
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| {
            ctx.execution
                .records_dir
                .join("artifacts")
                .join(default_name)
                .to_string_lossy()
                .to_string()
        })
}

pub(crate) fn artifact_path(ctx: &NodeContext<'_>, suffix: &str) -> String {
    let file_name = format!("{}_{}", ctx.node.id, suffix);
    PathBuf::from(&ctx.execution.records_dir)
        .join("artifacts")
        .join(Path::new(&file_name))
        .to_string_lossy()
        .to_string()
}

pub(crate) fn one(port: &str, value: Value) -> BTreeMap<String, Value> {
    BTreeMap::from([(port.to_string(), value)])
}

pub(crate) fn number(value: &serde_json::Value, key: &str, default: f64) -> f64 {
    value.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
}

pub(crate) fn integer(value: &serde_json::Value, key: &str, default: i64) -> i64 {
    value.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}

pub(crate) fn text(value: &serde_json::Value, key: &str, default: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

pub(crate) fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(VisionError::Operator {
        node: "opencv".to_string(),
        message: message.into(),
    })
}

struct BridgeWorker {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    stderr_log: Arc<Mutex<String>>,
}

impl BridgeWorker {
    fn start() -> Result<Self> {
        let bridge_path = Path::new(BRIDGE_EXE);
        let bridge_dir = bridge_path.parent().ok_or_else(|| VisionError::Operator {
            node: "opencv".to_string(),
            message: format!("OpenCV bridge path has no parent: {BRIDGE_EXE}"),
        })?;
        let mut child = Command::new(bridge_path)
            .arg("worker")
            .env("PATH", bridge_path_env(bridge_dir))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.take().ok_or_else(|| VisionError::Operator {
            node: "opencv".to_string(),
            message: "failed to open OpenCV worker stdin".to_string(),
        })?;
        let stdout = child.stdout.take().ok_or_else(|| VisionError::Operator {
            node: "opencv".to_string(),
            message: "failed to open OpenCV worker stdout".to_string(),
        })?;
        let stderr_log = Arc::new(Mutex::new(String::new()));
        if let Some(mut stderr) = child.stderr.take() {
            let log = Arc::clone(&stderr_log);
            thread::spawn(move || {
                let mut text = String::new();
                let _ = stderr.read_to_string(&mut text);
                if !text.is_empty() {
                    if let Ok(mut guard) = log.lock() {
                        guard.push_str(&text);
                    }
                }
            });
        }
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            stderr_log,
        })
    }

    fn request(&mut self, args: &[&str]) -> Result<serde_json::Value> {
        if args
            .iter()
            .any(|arg| arg.contains('\t') || arg.contains('\n'))
        {
            return fail("OpenCV bridge args cannot contain tabs or newlines");
        }
        let line = args.join("\t");
        writeln!(self.stdin, "{line}")?;
        self.stdin.flush()?;

        let mut response = String::new();
        self.stdout.read_line(&mut response)?;
        if response.trim().is_empty() {
            let status = match self.child.try_wait() {
                Ok(Some(status)) => format!(" exited with status {status}"),
                Ok(None) => " produced no stdout line".to_string(),
                Err(err) => format!(" status check failed: {err}"),
            };
            let stderr = self
                .stderr_log
                .lock()
                .ok()
                .map(|log| log.trim().to_string())
                .filter(|text| !text.is_empty())
                .unwrap_or_else(|| "no stderr captured".to_string());
            return fail(format!("OpenCV bridge{status}; stderr: {stderr}"));
        }
        serde_json::from_str(response.trim()).map_err(VisionError::from)
    }
}

fn bridge_path_env(bridge_dir: &Path) -> OsString {
    let mut path = OsString::from(bridge_dir.as_os_str());
    let separator = if cfg!(windows) { ";" } else { ":" };
    if let Some(existing) = env::var_os("PATH") {
        path.push(separator);
        path.push(existing);
    }
    path
}
