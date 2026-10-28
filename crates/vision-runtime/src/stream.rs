use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use vision_core::{
    Executor, FrameContext, Result, StreamConfig, StreamFrame, StreamFrameResult, StreamMetrics,
    StreamRunTrace, VisionError, Workflow,
};

pub struct FolderFrameSource {
    frames: VecDeque<PathBuf>,
    next_frame_id: u64,
    config: StreamConfig,
}

impl FolderFrameSource {
    pub fn open(folder: impl AsRef<Path>, config: StreamConfig) -> Result<Self> {
        let folder = folder.as_ref();
        let mut paths = fs::read_dir(folder)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| is_image(path))
            .collect::<Vec<_>>();
        paths.sort();
        if paths.is_empty() {
            return Err(VisionError::Operator {
                node: "stream.folder".to_string(),
                message: format!("no supported image files in {}", folder.display()),
            });
        }
        Ok(Self {
            frames: paths.into(),
            next_frame_id: 1,
            config,
        })
    }

    pub fn next_frame(&mut self) -> Option<StreamFrame> {
        let image_path = self.frames.pop_front()?;
        let frame_id = self.next_frame_id;
        self.next_frame_id += 1;
        Some(StreamFrame {
            context: FrameContext {
                frame_id,
                captured_at_unix_ms: now_unix_ms(),
                source_uri: image_path.to_string_lossy().into_owned(),
                device_id: self.config.device_id.clone(),
                trigger_id: None,
                batch_id: self.config.batch_id.clone(),
            },
            image_path,
        })
    }
}

pub struct StreamExecutor<'a> {
    executor: Executor<'a>,
}

/// Small synchronous bounded queue used by future camera/video producers and the current folder source.
pub struct BoundedFrameQueue {
    frames: VecDeque<StreamFrame>,
    capacity: usize,
    policy: vision_core::BackpressurePolicy,
}

impl BoundedFrameQueue {
    pub fn new(capacity: usize, policy: vision_core::BackpressurePolicy) -> Result<Self> {
        if capacity == 0 {
            return Err(VisionError::Operator {
                node: "stream.queue".into(),
                message: "capacity must be greater than zero".into(),
            });
        }
        Ok(Self {
            frames: VecDeque::with_capacity(capacity),
            capacity,
            policy,
        })
    }

    pub fn push(&mut self, frame: StreamFrame) -> bool {
        if self.frames.len() < self.capacity {
            self.frames.push_back(frame);
            return true;
        }
        match self.policy {
            vision_core::BackpressurePolicy::Block => false,
            vision_core::BackpressurePolicy::DropNewest => false,
            vision_core::BackpressurePolicy::DropOldest => {
                self.frames.pop_front();
                self.frames.push_back(frame);
                true
            }
        }
    }

    pub fn pop(&mut self) -> Option<StreamFrame> {
        self.frames.pop_front()
    }
    pub fn len(&self) -> usize {
        self.frames.len()
    }
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

impl<'a> StreamExecutor<'a> {
    pub fn new(registry: &'a vision_core::OperatorRegistry) -> Self {
        Self {
            executor: Executor::new(registry),
        }
    }

    pub fn run_folder(
        &self,
        workflow: &Workflow,
        folder: impl AsRef<Path>,
        records_dir: impl AsRef<Path>,
        config: StreamConfig,
    ) -> Result<StreamRunTrace> {
        if config.queue_capacity == 0 {
            return Err(VisionError::Operator {
                node: "stream".to_string(),
                message: "queue_capacity must be greater than zero".to_string(),
            });
        }
        ensure_read_node(workflow)?;
        let mut source = FolderFrameSource::open(folder, config.clone())?;
        let start = Instant::now();
        let mut queue = VecDeque::with_capacity(config.queue_capacity);
        let mut trace = StreamRunTrace {
            workflow_id: workflow.id.clone(),
            frames: Vec::new(),
            metrics: StreamMetrics::default(),
        };

        loop {
            while queue.len() < config.queue_capacity
                && within_limit(&config, trace.metrics.received_frames)
            {
                let Some(frame) = source.next_frame() else {
                    break;
                };
                trace.metrics.received_frames += 1;
                queue.push_back(frame);
                trace.metrics.max_queue_depth = trace.metrics.max_queue_depth.max(queue.len());
            }
            let Some(frame) = queue.pop_front() else {
                break;
            };
            let frame_start = Instant::now();
            let mut frame_workflow = workflow.clone();
            apply_frame(&mut frame_workflow, &frame, &records_dir)?;
            match self.executor.run(&frame_workflow, records_dir.as_ref()) {
                Ok(run) => {
                    trace.metrics.processed_frames += 1;
                    let result = final_result(&run);
                    trace.frames.push(StreamFrameResult {
                        context: frame.context,
                        run_id: Some(run.run_id),
                        result,
                        duration_ms: round3(frame_start.elapsed().as_secs_f64() * 1000.0),
                        error: None,
                    });
                }
                Err(error) => {
                    trace.metrics.failed_frames += 1;
                    trace.frames.push(StreamFrameResult {
                        context: frame.context,
                        run_id: None,
                        result: "ERROR".to_string(),
                        duration_ms: round3(frame_start.elapsed().as_secs_f64() * 1000.0),
                        error: Some(error.to_string()),
                    });
                }
            }
        }

        trace.metrics.total_duration_ms = round3(start.elapsed().as_secs_f64() * 1000.0);
        if trace.metrics.processed_frames > 0 {
            trace.metrics.average_frame_ms = round3(
                trace
                    .frames
                    .iter()
                    .map(|frame| frame.duration_ms)
                    .sum::<f64>()
                    / trace.metrics.processed_frames as f64,
            );
        }
        trace.metrics.processing_fps = fps(
            trace.metrics.processed_frames,
            trace.metrics.total_duration_ms,
        );
        let trace_path = records_dir.as_ref().join("stream_trace.json");
        fs::create_dir_all(records_dir.as_ref())?;
        fs::write(trace_path, serde_json::to_vec_pretty(&trace)?)?;
        Ok(trace)
    }
}

fn apply_frame(
    workflow: &mut Workflow,
    frame: &StreamFrame,
    records_dir: impl AsRef<Path>,
) -> Result<()> {
    let mut replaced = false;
    for node in &mut workflow.nodes {
        if node.operator == "opencv.image.read" {
            node.params["path"] = serde_json::Value::String(frame.context.source_uri.clone());
            replaced = true;
        }
        if node.operator == "opencv.image.write" {
            let output = node
                .params
                .get("output")
                .and_then(|value| value.as_str())
                .unwrap_or("stream_output.png");
            node.params["output"] = serde_json::Value::String(
                frame_output_path(records_dir.as_ref(), output, frame.context.frame_id)
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    if !replaced {
        return Err(VisionError::Operator {
            node: "stream".to_string(),
            message: "workflow needs an opencv.image.read node as its stream input".to_string(),
        });
    }
    workflow.metadata.insert(
        "frame_context".to_string(),
        serde_json::to_value(&frame.context)?,
    );
    Ok(())
}

fn ensure_read_node(workflow: &Workflow) -> Result<()> {
    if workflow
        .nodes
        .iter()
        .any(|node| node.operator == "opencv.image.read")
    {
        Ok(())
    } else {
        Err(VisionError::Operator {
            node: "stream".to_string(),
            message: "workflow needs an opencv.image.read node as its stream input".to_string(),
        })
    }
}

fn frame_output_path(records_dir: &Path, output: &str, frame_id: u64) -> PathBuf {
    let original = Path::new(output);
    let stem = original
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("image");
    let extension = original
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("png");
    records_dir
        .join("stream_output")
        .join(format!("{stem}_frame_{frame_id:06}.{extension}"))
}

fn final_result(run: &vision_core::RunTrace) -> String {
    run.nodes
        .iter()
        .rev()
        .find_map(|node| {
            node.outputs
                .iter()
                .find(|port| port.port == "final")
                .map(|port| port.value.clone())
        })
        .unwrap_or_else(|| "UNKNOWN".to_string())
}

fn is_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .as_deref(),
        Some("jpg" | "jpeg" | "png" | "bmp" | "tif" | "tiff")
    )
}

fn within_limit(config: &StreamConfig, received_frames: u64) -> bool {
    config
        .max_frames
        .is_none_or(|limit| received_frames < limit)
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn fps(frames: u64, duration_ms: f64) -> f64 {
    if duration_ms <= 0.0 {
        0.0
    } else {
        round3(frames as f64 * 1000.0 / duration_ms)
    }
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_a_unique_output_path_per_frame() {
        let path = frame_output_path(Path::new("records"), "demo/output/result.png", 12);
        assert_eq!(
            path,
            PathBuf::from("records/stream_output/result_frame_000012.png")
        );
    }

    #[test]
    fn drop_oldest_keeps_the_newest_frame() {
        let context = |id| vision_core::StreamFrame {
            context: vision_core::FrameContext {
                frame_id: id,
                captured_at_unix_ms: 0,
                source_uri: format!("{id}.png"),
                device_id: None,
                trigger_id: None,
                batch_id: None,
            },
            image_path: PathBuf::from(format!("{id}.png")),
        };
        let mut queue =
            BoundedFrameQueue::new(1, vision_core::BackpressurePolicy::DropOldest).unwrap();
        assert!(queue.push(context(1)));
        assert!(queue.push(context(2)));
        assert_eq!(queue.pop().unwrap().context.frame_id, 2);
    }
}
