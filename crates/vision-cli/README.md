# vision-cli

`vision-cli` is the minimal command-line entry point for the current Rust MVP.

It is responsible for:

```text
register built-in operators
load workflow JSON
run Executor
print run result, trace path, processing time, and FPS
print operator schemas for UI and agent planning
```

It is not responsible for:

```text
UI
HTTP API service
agent orchestration service
complex interactive parameter editing
```

## Commands

```bash
cargo run -p vision-cli -- list-operators
cargo run -p vision-cli -- list-operators-json
cargo run -p vision-cli -- describe-operator comm.kafka.publish
cargo run -p vision-cli -- run examples/workflows/io_output_demo.json
cargo run -p vision-cli -- run-folder examples/workflows/opencv_rotate_right_demo.json demo/stream_input 10
```

`list-operators-json`, `export-schema`, and `describe-operator` include `ParamSchema` metadata, so
the future UI can generate parameter panels from Rust node definitions.

`run-folder` is the first Stage 8 stream command. It enumerates supported image files in lexical order,
replaces the workflow's `opencv.image.read.path` for each frame, and reuses the normal Rust DAG
executor. Its aggregate output is written to `records/stream_trace.json`; image write nodes are
redirected to `records/stream_output` with a frame-number suffix so frames cannot overwrite each other.
