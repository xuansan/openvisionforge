# 质量判定与测量节点开发记录

本文记录“算法节点局部判定 + 独立规则节点 + 最终结果聚合”的实现方案，面向 Rust runtime、React/Tauri UI 和后续 LLM workflow 编排。

## 1. 总体目标

工业视觉检测流程不只是图像算法串联，而是完整闭环：

```text
输入/采集
→ 预处理
→ 轮廓/定位/测量/识别
→ 节点内局部 OK/NG
→ 独立规则判断
→ 最终结果聚合
→ 保存/通信/信号输出
```

本次实现把“判定”提升为一等数据结构和一等节点，避免所有业务规则都散落在算法节点内部。

## 2. 核心数据结构

位置：

- `crates/vision-core/src/quality.rs`
- `crates/vision-core/src/value.rs`

新增核心类型：

- `ScaleContext`：比例尺，上游可以将 px 转 mm。
- `MeasurementValue`：单个测量值，包含 `value_px`、`value_mm`、`unit`、`confidence`、`quality_score`、`ok`、`reason`。
- `MeasurementSet`：测量值集合。
- `ContourResult`：轮廓结果，包含外接矩形、面积、周长、圆度、长宽比、中心点。
- `ContourSet`：轮廓集合。
- `LocationResult`：定位结果，包含 found、位置、角度、score、bbox。
- `RuleResult`：单条规则判定结果。
- `FinalResult`：最终质量结果。

`Value` 新增：

- `Contours`
- `Measurements`
- `Location`
- `Rule`
- `Rules`
- `Final`

## 3. 端口类型和校验

位置：

- `crates/vision-core/src/validation.rs`

新增端口类型：

- `Contours`
- `Measurements`
- `Location`
- `Rule`
- `Final`

规则节点的 `data` 输入保持 `Any`，这样可以消费：

- `Json`
- `Contours`
- `Measurements`
- `Detections`
- `Location`
- `Bool`
- `Final`

`status` 端口改为 `Text`，真正的设备/通信信号继续使用 `signal` 端口。

## 4. 独立规则节点

位置：

- `crates/vision-operators/src/rules.rs`

已实现节点：

| 节点 ID | 作用 |
| --- | --- |
| `rule.numeric_range` | 数值范围判断 |
| `rule.count_range` | 数量范围判断 |
| `rule.bool` | 布尔判断 |
| `rule.string_match` | 字符串 equals/contains 判断 |
| `rule.confidence_min` | 置信度阈值判断 |
| `rule.class_allowed` | 类别白名单判断 |
| `rule.region_contains` | 区域包含判断 |
| `logic.and` | 多规则 AND |
| `logic.or` | 多规则 OR |
| `logic.not` | 规则取反 |
| `logic.final_result` | 汇总最终 OK/NG |

聚合类节点支持多个显式输入口：

- `rule`
- `rule2`
- `rule3`
- `rule4`
- `rule5`
- `rules`

这样适合 UI 拖拽连线，也符合当前校验规则中“一个输入端口只能接一条线”的模型。

## 5. 配方/工艺参数节点

位置：

- `crates/vision-operators/src/recipe.rs`

已实现节点：

- `recipe.select`

输出：

```json
{
  "product_code": "default",
  "recipe_version": "v1",
  "parameters": {}
}
```

当前是轻量接口，用于后续 UI 的产品型号、配方版本、阈值参数集管理。

## 6. OpenCV 轮廓/Blob/缺陷节点

位置：

- `crates/vision-opencv/src/ops/analysis.rs`

已实现/注册节点：

| 节点 ID | 当前状态 |
| --- | --- |
| `opencv.analysis.connected_components` | 复用 OpenCV bridge 轮廓分析 |
| `opencv.analysis.find_contours` | 真实调用 OpenCV bridge 的 `findContours` |
| `opencv.analysis.contour_filter` | Rust 侧按面积、圆度、长宽比过滤 |
| `opencv.analysis.area_filter` | Rust 侧面积过滤 |
| `opencv.analysis.roundness` | 输出圆度测量 |
| `opencv.analysis.aspect_ratio` | 输出长宽比测量 |
| `opencv.analysis.bounding_rect` | 输出外接矩形测量 |
| `opencv.analysis.convex_hull` | 已预留节点接口，当前透传轮廓并输出说明性测量 |
| `opencv.analysis.defect_points` | 基于圆度/质量评分做候选缺陷区域输出 |
| `opencv.analysis.holes` | 复用轮廓分析接口 |
| `opencv.analysis.spots` | 复用轮廓分析接口 |
| `opencv.analysis.burrs` | 基于长宽比/圆度输出毛刺候选规则 |
| `opencv.analysis.standard_diff` | 标准件差异比对接口预留 |

## 7. OpenCV 几何测量节点

位置：

- `crates/vision-opencv/src/ops/geometry.rs`

已实现/注册节点：

- `opencv.measure.point_distance`
- `opencv.measure.point_line_distance`
- `opencv.measure.line_width`
- `opencv.measure.circle_diameter`
- `opencv.measure.circle_center_distance`
- `opencv.measure.angle`
- `opencv.measure.parallelism`
- `opencv.measure.perpendicularity`
- `opencv.measure.concentricity`
- `opencv.measure.area`
- `opencv.measure.perimeter`
- `opencv.measure.min_bounding_rect`
- `opencv.measure.contour_size`
- `opencv.measure.gap`
- `opencv.measure.step_height`
- `opencv.measure.flatness`

所有几何测量节点统一输出 `measurements`。

支持局部判定参数：

- `px_per_mm`
- `min_value`
- `max_value`

当设置 `min_value` 或 `max_value` 时，测量值会带上局部 `ok` 和 `reason`。

## 8. OpenCV 定位节点

位置：

- `crates/vision-opencv/src/ops/location.rs`

已注册节点：

- `opencv.locate.template_match`
- `opencv.locate.template_match_rotated`
- `opencv.locate.template_match_multiscale`
- `opencv.locate.feature_match`
- `opencv.locate.circle`
- `opencv.locate.line`
- `opencv.locate.corner`
- `opencv.locate.edge`
- `opencv.locate.blob`
- `opencv.locate.keypoint`
- `opencv.locate.pose`

当前统一输出 `location`，复杂算法后端后续可以逐个替换。

## 9. 示例 Workflow

位置：

- `examples/workflows/quality_contour_judge_demo.json`

流程：

```text
opencv.image.read
→ opencv.color.to_gray
→ opencv.threshold.otsu
→ opencv.analysis.find_contours
→ rule.count_range
→ logic.final_result
→ opencv.image.write
→ record.result.json
```

运行命令：

```powershell
cargo run -p vision-cli -- run examples\workflows\quality_contour_judge_demo.json
```

测试结果：

```text
Result: OK
Nodes: 8
Processing: 119.34 ms
FPS: 8.379
```

输出文件：

- `demo/output/quality_contour_judge_binary.png`
- `demo/output/quality_contour_judge_result.json`
- `records/<run_id>/trace.json`
- `records/<run_id>/event.json`

## 10. UI 联动建议

UI 后续应从 Rust schema 中读取：

- 节点分类。
- 输入端口。
- 输出端口。
- 参数 schema。
- 规则节点参数。
- 可连接端口类型。

规则配置 UI 推荐做成：

```text
选择上游节点
→ 选择输出字段
→ 选择判断类型
→ 填阈值/字符串/类别/ROI
→ 生成 rule 节点参数
```

## 11. 后续替换真实算法的优先级

当前已经完成节点体系和部分真实 OpenCV 调用。下一步建议优先替换这些预留实现：

1. `opencv.locate.template_match`：真实模板匹配。
2. `opencv.locate.circle`：Hough 圆或轮廓圆拟合。
3. `opencv.locate.line`：Hough 线或边缘线拟合。
4. `opencv.analysis.convex_hull`：真实凸包输出。
5. `opencv.analysis.holes`：按层级轮廓检测孔洞。
6. `opencv.analysis.standard_diff`：模板差分或特征差异。
7. `opencv.measure.parallelism/perpendicularity/concentricity`：真实几何对象输入。
8. `opencv.measure.step_height/flatness`：等待 3D/点云数据类型后实现。
