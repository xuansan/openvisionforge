# vision-operators

`vision-operators` 放当前内置的 mock 节点。

这些节点用于验证 workflow 数据流，不做真实图像算法。

## 当前节点

```text
source.image.file.mock
source.image.folder.mock
trigger.software.mock
source.camera.mock
opencv.color.to_gray.mock
opencv.threshold.fixed.mock
opencv.blob.detect.mock
dl.yolo.detect.mock
judge.area_range
judge.count_range
logic.all_pass
record.save_json.mock
output.signal.mock
```

## 数据流角色

```text
Source / Trigger
  产生 ImageFrame、TriggerInfo、CaptureInfo。

OpenCv mock
  模拟灰度、阈值、blob 检测的数据形状。

AI mock
  模拟通用模型检测节点输出 detections，不加载模型。

Judge / Logic
  把检测结果转成 OK / NG。

Record / Output
  保存 result.json，模拟现场信号输出。
```

## 设计边界

现在这里不接：

```text
opencv crate
camera sdk
onnxruntime
tensorrt
真实 YOLO decode
```

真实能力后续应新增 crate，例如：

```text
vision-opencv
vision-inference
vision-camera
```

先让 mock 节点把 workflow 协议跑稳。
