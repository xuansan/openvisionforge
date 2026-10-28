# vision-runtime/src

`vision-runtime` assembles the operator registries from the feature crates and is the runtime-facing
boundary used by both the CLI and Tauri Studio.

## Files

```text
lib.rs       Registers OpenCV, inference, IO, and core operators; exports the shared schema.
stream.rs    Runs an existing single-frame DAG repeatedly for a stream of image files.
```

## Stream Data Flow

```text
folder image files
  -> FolderFrameSource
  -> StreamFrame { FrameContext, image_path }
  -> bounded prefetch queue
  -> clone workflow and replace opencv.image.read.path
  -> existing vision_core::Executor
  -> per-frame RunTrace + StreamRunTrace
```

`FrameContext` carries the frame number, source URI, timestamp, device ID, trigger ID, and batch ID.
The current folder-stream MVP uses the source URI and timestamp directly; camera and hardware trigger
adapters will populate the device and trigger fields later. Existing OpenCV, inference, rule, output,
and record nodes remain ordinary single-frame nodes and are deliberately not duplicated for streaming.

`StreamConfig` controls queue capacity and the maximum number of frames. `BoundedFrameQueue` now
implements the synchronous queue policy semantics: `block`/`drop_newest` reject a push when full,
and `drop_oldest` evicts the oldest frame. Threaded producer/consumer orchestration is intentionally
deferred until the video and camera inputs need it.
