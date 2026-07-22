# demo_onnx

这个目录记录当前 ONNX/YOLO 推理链路已经跑通的结果。

## 已跑通

- 默认 Rust workspace 编译通过。
- `vision-infer/onnxruntime-cpu` feature 编译通过。
- `yolo_infer_demo` 工作流跑通：
  - `opencv.image.read` 读取测试图片。
  - `infer.yolo.detect` 输出 `Detections(1)`。
  - `record.result.json` 保存结果。
  - 总处理耗时 `84.826 ms`。
  - 处理帧数 `1`。
  - FPS `11.789`。

## 运行命令

```powershell
$env:PATH="$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo check
cargo check -p vision-cli --features vision-infer/onnxruntime-cpu
cargo run -p vision-cli -- run examples/workflows/yolo_infer_demo.json
```

## 当前真实 ONNX Runtime 状态

真实 ONNX Runtime 后端代码已经实现，并且 `onnxruntime-cpu` feature 能编译通过。

真实运行还需要本机提供 `onnxruntime.dll`：

```powershell
$env:ORT_DYLIB_PATH="C:\path\to\onnxruntime.dll"
cargo run -p vision-cli --features vision-infer/onnxruntime-cpu -- run examples/workflows/yolo_onnxruntime_person_demo.json
```

本次没有完成真实 ONNX Runtime 运行测试，原因是外部 DLL 下载多次网络超时。代码侧已经切到动态加载模式，避免 Windows 静态链接 ONNX Runtime 时的 MSVC C++ STL 符号不匹配问题。

## 文件说明

- `workflow_yolo_infer_demo.json`: 本次跑通的工作流。
- `event.json`: 本次运行总体结果。
- `trace.json`: 节点级运行记录和耗时。
- `record.json`: 工作流保存的检测结果。
- `run_summary.json`: 为 demo 单独整理的摘要。
