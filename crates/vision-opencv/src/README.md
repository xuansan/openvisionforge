# vision-opencv/src

This folder contains the Rust node layer for OpenCV-backed operators.

Rust files define operator IDs, categories, ports, parameters, Value wrapping,
and calls to the backend. The pixel-level OpenCV algorithms live in `cpp/`.

## Layout

```text
lib.rs
  Registers all OpenCV operator groups.

backend.rs
  Owns the persistent OpenCV worker process and shared helpers:
  bridge_image, bridge_json, image_path, artifact_path, params helpers.

schema.rs
  Small helper for building OperatorSchema values.

ops/image.rs
  opencv.image.read
  opencv.image.write
  opencv.image.clone
  opencv.image.resize
  opencv.image.crop

ops/color.rs
  opencv.color.to_gray
  opencv.color.convert

ops/threshold.rs
  opencv.threshold.fixed
  opencv.threshold.otsu
  opencv.threshold.triangle
  opencv.threshold.adaptive_mean
  opencv.threshold.adaptive_gaussian
  opencv.threshold.range

ops/filter.rs
  opencv.filter.blur
  opencv.filter.gaussian
  opencv.filter.median
  opencv.filter.sharpen

ops/morph.rs
  opencv.morph.erode
  opencv.morph.dilate
  opencv.morph.open
  opencv.morph.close
  opencv.morph.gradient
  opencv.morph.tophat
  opencv.morph.blackhat

ops/measure.rs
  opencv.measure.binary_ratio
  opencv.measure.pixel_count

ops/transform.rs
  opencv.transform.rotate
  opencv.transform.flip

ops/edge.rs
  opencv.edge.canny
```

## Adding A New OpenCV Operator

1. Add or update the Rust file under `ops/`.
2. Register an `OperatorSchema` with stable operator ID and ports.
3. Call the backend through `bridge_image()` or `bridge_json()`.
4. Add the matching C++ command in `cpp/`.
5. Add a workflow example when the operator introduces a new data path.
