# vision-core

`vision-core` is the stable protocol and minimal runtime layer of the platform.

It owns:

```text
Workflow / WorkflowNode / WorkflowEdge
Value data protocol
ModelAsset data structures
OperatorSchema / OperatorFn
ParamSchema / ParamKind
OperatorRegistry
DAG validation and topological order
Synchronous workflow executor
Trace / Event records
Runtime metrics
VisionError / Result
```

## Operator Parameter Schema

`OperatorSchema` includes `params`, a static list of `ParamSchema` values. This
lets the UI, CLI, and future agent planner discover node parameters without
duplicating hard-coded forms outside Rust.

```text
ParamSchema
  key
  label
  kind
  required
  default
  description
  options
```

The runtime still reads actual node params from `WorkflowNode.params`, so adding
metadata does not change existing workflow files.

It does not own:

```text
OpenCV algorithms
Camera SDKs
ONNX Runtime / TensorRT inference
Databases
UI
Agent APIs
```

## Data Flow

```text
Workflow
  nodes + edges

Executor
  validate workflow
  sort nodes by DAG order
  collect input Values from upstream edges
  call OperatorFn
  store output Values
  write NodeTrace

Records
  trace.json
  event.json
```

Nodes exchange only `Value`:

```text
Image
Mask
Regions
Detections
Trigger
Capture
Signal
Bool
Text
Json
None
```

## Runtime Metrics

The executor records metrics in Rust, so individual operators do not need to
repeat the same timing code.

```text
NodeTrace.metrics.duration_ms
NodeTrace.metrics.input_images
NodeTrace.metrics.output_images

RunTrace.metrics.total_duration_ms
RunTrace.metrics.processed_frames
RunTrace.metrics.image_outputs
RunTrace.metrics.fps
```

`processed_frames` currently counts image frames produced by source-style nodes.
For future folder batch, camera trigger, and video stream modes, this can evolve
into packet-level frame accounting without changing operator IDs.

## Design Rules

```text
Core keeps stable contracts only.
Core does not know business-specific defect meanings.
Core does not expose OpenCV Mat, ONNX tensors, TensorRT buffers, or camera SDK objects.
Operators produce structured Values.
Business meaning belongs in workflow + params + judge + logic.
```
