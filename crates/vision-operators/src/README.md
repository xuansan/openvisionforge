# vision-operators/src

当前只有一个 `lib.rs`，所有 mock operator 都在这里。

## 主要入口

```text
register_all(registry)
  把所有内置 mock operator 注册到 OperatorRegistry。
```

每个节点通过 `schema(...)` 注册：

```text
id
name
category
inputs
optional_inputs
outputs
run fn
```

## 当前数据流

触发拍照流程：

```text
trigger.software.mock
  -> Value::Trigger

source.camera.mock
  trigger -> Value::Image + Value::Capture

opencv.color.to_gray.mock
  image -> gray image

opencv.threshold.fixed.mock
  image -> mask

opencv.blob.detect.mock
  mask -> regions

dl.yolo.detect.mock
  image + model_id params -> detections

judge.area_range
  regions -> passed + failed_regions

judge.count_range
  detections -> passed + failed_count

logic.all_pass
  passed -> status

output.signal.mock
  status -> signal

record.save_json.mock
  status + optional business inputs -> result.json
```

## 添加新 mock 节点的最小步骤

```text
1. 写一个 fn(ctx: &NodeContext) -> Result<BTreeMap<String, Value>>
2. 在 register_all 里注册 schema
3. 在 examples/workflows 里加一个 workflow 验证
4. cargo test
```

先不要为一个节点拆文件。等这个文件真的难读了再拆。
