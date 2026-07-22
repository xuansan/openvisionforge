use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    println!("cargo:rerun-if-changed=cpp/opencv_bridge.cpp");
    println!("cargo:rerun-if-changed=cpp/bridge_common.hpp");
    println!("cargo:rerun-if-changed=cpp/color_ops.cpp");
    println!("cargo:rerun-if-changed=cpp/draw_ops.cpp");
    println!("cargo:rerun-if-changed=cpp/edge_ops.cpp");
    println!("cargo:rerun-if-changed=cpp/enhance_ops.cpp");
    println!("cargo:rerun-if-changed=cpp/filter_ops.cpp");
    println!("cargo:rerun-if-changed=cpp/image_ops.cpp");
    println!("cargo:rerun-if-changed=cpp/measure_ops.cpp");
    println!("cargo:rerun-if-changed=cpp/morph_ops.cpp");
    println!("cargo:rerun-if-changed=cpp/threshold_ops.cpp");
    println!("cargo:rerun-if-changed=cpp/transform_ops.cpp");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set"));
    let exe = out_dir.join("opencv_bridge.exe");

    if cfg!(target_os = "windows") {
        build_windows(&exe);
    } else {
        panic!("vision-opencv currently builds the OpenCV bridge only on Windows");
    }

    println!("cargo:rustc-env=OPENCV_BRIDGE_EXE={}", exe.display());
}

fn build_windows(exe: &Path) {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let workspace_dir = manifest_dir
        .ancestors()
        .nth(2)
        .expect("workspace dir")
        .to_path_buf();
    let env_opencv_dir = env::var_os("OPENCV_DIR")
        .map(PathBuf::from)
        .filter(|path| path.join("include/opencv2/opencv.hpp").exists());
    let opencv_dir =
        env_opencv_dir.unwrap_or_else(|| workspace_dir.join("vendor/opencv5/opencv/build"));
    let include_dir = opencv_dir.join("include");
    let lib_dir = opencv_dir.join("x64/vc16/lib");
    let bin_dir = opencv_dir.join("x64/vc16/bin");
    let source = manifest_dir.join("cpp/opencv_bridge.cpp");
    let obj = exe.with_extension("obj");
    let vcvars = find_vsdevcmd().expect("Visual Studio VsDevCmd.bat was not found");

    fs::create_dir_all(exe.parent().expect("exe parent")).expect("create bridge out dir");

    let batch = exe.with_file_name("build_opencv_bridge.bat");
    let command = format!(
        "@echo off\r\ncall \"{}\" -arch=x64 -host_arch=x64 >nul\r\nif errorlevel 1 exit /b 1\r\ncl /nologo /EHsc /std:c++17 /I\"{}\" \"{}\" /Fo\"{}\" /Fe\"{}\" /link /LIBPATH:\"{}\" opencv_world500.lib\r\n",
        vcvars.display(),
        include_dir.display(),
        source.display(),
        obj.display(),
        exe.display(),
        lib_dir.display()
    );
    fs::write(&batch, command).expect("write bridge build batch");
    let status = Command::new("cmd")
        .args(["/C", batch.to_str().expect("batch path")])
        .status()
        .expect("run cl");
    if !status.success() {
        panic!("failed to build OpenCV bridge");
    }

    let dll = bin_dir.join("opencv_world500.dll");
    let target_dll = exe
        .parent()
        .expect("exe parent")
        .join("opencv_world500.dll");
    fs::copy(&dll, &target_dll).expect("copy opencv_world500.dll");
}

fn find_vsdevcmd() -> Option<PathBuf> {
    [
        r"C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat",
        r"C:\Program Files\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat",
        r"C:\Program Files\Microsoft Visual Studio\2022\Professional\Common7\Tools\VsDevCmd.bat",
        r"C:\Program Files\Microsoft Visual Studio\2022\Enterprise\Common7\Tools\VsDevCmd.bat",
    ]
    .into_iter()
    .map(PathBuf::from)
    .find(|path| path.exists())
}
