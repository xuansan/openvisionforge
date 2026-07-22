# vision-opencv

`vision-opencv` provides real OpenCV-backed workflow nodes.

The crate keeps OpenCV behind the operator boundary:

```text
vision-core Value / Workflow
  -> vision-opencv operator
  -> OpenCV backend bridge
  -> vision-core Value
```

`vision-core` does not expose `cv::Mat`. Images still flow through `ImageFrame`
metadata and file-backed `source_uri` paths.

## Current Nodes

```text
opencv.image.read
opencv.image.write
opencv.image.clone
opencv.image.resize
opencv.image.crop
opencv.color.to_gray
opencv.color.convert
opencv.threshold.fixed
opencv.threshold.otsu
opencv.threshold.triangle
opencv.threshold.adaptive_mean
opencv.threshold.adaptive_gaussian
opencv.threshold.range
opencv.filter.blur
opencv.filter.gaussian
opencv.filter.median
opencv.filter.sharpen
opencv.morph.erode
opencv.morph.dilate
opencv.morph.open
opencv.morph.close
opencv.morph.gradient
opencv.morph.tophat
opencv.morph.blackhat
opencv.measure.binary_ratio
opencv.measure.pixel_count
opencv.transform.rotate
opencv.transform.flip
opencv.edge.canny
```

The first backend uses the local OpenCV 5 SDK through a small C++ worker built
by Cargo. The Rust side keeps one worker process alive and sends commands to it,
so normal node execution does not pay process startup cost for every operator.

The public node IDs do not include the OpenCV version, so the backend can later
move to Rust bindings, OpenCV CUDA, or another native implementation without
changing workflow files.

## C++ Backend Layout

```text
cpp/opencv_bridge.cpp
  Process entrypoint and worker protocol.

cpp/bridge_common.hpp
  Shared helpers for image IO, metadata JSON, and command parsing.

cpp/image_ops.cpp
  metadata, write

cpp/color_ops.cpp
  gray, color_convert

cpp/filter_ops.cpp
  blur, gaussian, median, sharpen

cpp/threshold_ops.cpp
  threshold, otsu, triangle, adaptive threshold, range threshold

cpp/measure_ops.cpp
  ratio

cpp/transform_ops.cpp
  rotate, flip

cpp/morph_ops.cpp
  erode, dilate, open, close, gradient, tophat, blackhat

cpp/edge_ops.cpp
  canny
```

## Rust Operator Layout

Each operator category has a Rust file, even when the implementation calls the
OpenCV worker. This keeps node semantics, parameters, ports, and backend calls
easy to inspect.

```text
src/lib.rs
  Module entrypoint and register_all().

src/backend.rs
  OpenCV worker process, bridge requests, path helpers, Value helpers.

src/schema.rs
  OperatorSchema helper.

src/ops/image.rs
  image read/write/clone/resize/crop

src/ops/color.rs
  grayscale and color conversion

src/ops/threshold.rs
  fixed, otsu, triangle, adaptive, and range threshold

src/ops/filter.rs
  blur, gaussian, median, sharpen

src/ops/morph.rs
  morphology operators

src/ops/measure.rs
  binary ratio and pixel count

src/ops/transform.rs
  rotate and flip

src/ops/edge.rs
  canny
```
