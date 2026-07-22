# vision-core/src

This folder contains the core Rust source files.

## Files

```text
lib.rs
  Module entrypoint and public exports.

error.rs
  VisionError and Result.

value.rs
  Data protocol exchanged between workflow nodes.

model.rs
  Model asset and inference-layer descriptors. It does not run inference.

workflow.rs
  Workflow / node / edge JSON structures.

operator.rs
  OperatorSchema and OperatorFn.

registry.rs
  OperatorRegistry, mapping operator IDs to schemas.

dag.rs
  Topological ordering from workflow edges.

executor.rs
  Current synchronous workflow executor.

trace.rs
  NodeTrace, RunTrace, RunEvent, and runtime metrics.
```

## Execution Path

```text
Executor::run
  -> validate(workflow)
  -> topo_order(workflow)
  -> collect_inputs(...)
  -> operator.run(...)
  -> save output Values
  -> generate NodeTrace with metrics
  -> write trace.json / event.json
```

## Value Flow Example

```text
opencv.image.read
  output:
    image: Value::Image

opencv.transform.rotate
  input:
    image: Value::Image
  output:
    image: Value::Image

opencv.measure.binary_ratio
  input:
    image: Value::Image
  output:
    stats: Value::Json

judge.area_range
  input:
    regions: Value::Regions
  output:
    passed: Value::Bool
```

## Next Likely Changes

```text
Batch/frame packet accounting
Async or streaming executor
Richer parameter schema validation
Error traces for failed nodes
```
