fn main() {
    // ONNX Runtime is loaded dynamically by the `ort/load-dynamic` feature.
    // Keeping this build script lets us add platform-specific runtime checks later
    // without changing the crate manifest again.
}
