# Demo: Folder Binary Image Processing

这个 demo 做一件小事：

```text
读取 input 文件夹内的图片
  -> 转灰度
  -> 按阈值二值化成黑白图
  -> 输出图片分辨率
  -> 统计黑/白像素比例
  -> 保存黑白图片到 output 文件夹
  -> 保存 report.csv
```

## 目录

```text
demo/
  Cargo.toml
  src/main.rs
  input/
  output/
```

## 运行

在 `vision-platform` 目录下：

OpenCV 5 C++ 调用测试：

```powershell
.\demo\opencv_check.exe .\demo\input\d34084cc1f765c877abcdf2d515cacf3.jpg .\demo\output\opencv5_binary.png
```

无依赖脚本版：

```powershell
.\demo\run_demo.ps1
```

Rust 版：

```bash
cd demo
cargo run
```

自定义输入输出和阈值：

```bash
cargo run -- input output 128
```

## 输出

```text
demo/output/<原文件名>_bw.png
demo/output/opencv5_binary.png
demo/output/report.csv
```

`report.csv` 字段：

```text
file,width,height,black_pixels,white_pixels,black_ratio,white_ratio,output
```

## 说明

`opencv_check.cpp` 是最小 OpenCV 5 调用测试：

```text
cv::imread
cv::cvtColor
cv::threshold
cv::countNonZero
cv::imwrite
```

这个 demo 使用 `image` crate 只做 JPEG/PNG 读取和 PNG 保存。

它不接入当前 workflow runtime。原因很简单：这是一个单独演示“文件夹图片处理”的最小程序，先把真实图片 I/O 跑通；后面再把这套逻辑拆成正式 operator。

如果 Rust 依赖下载慢，可以先运行 `run_demo.ps1`。PowerShell 版本不需要下载依赖。
