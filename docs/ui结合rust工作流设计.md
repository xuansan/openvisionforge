# UI 结合 Rust 工作流设计

本文档用于约束 Vision Studio UI 和 Rust runtime 的结合方式。当前阶段的核心不是做完整平台，而是先把一个能运行、能编辑、能校验、能生成 Rust 可执行 workflow JSON 的工作流编辑器做扎实。

## 1. 核心定位

UI 是 Rust 视觉检测 runtime 的可视化开发台。

Rust 负责：

- 节点类型定义：`OperatorSchema`
- 工作流数据结构：`Workflow`
- 节点输入输出规则：`inputs / optional_inputs / outputs`
- 参数 schema：`ParamSchema`
- 工作流校验：节点、端口、连接、必填输入
- 执行工作流：`Executor`
- 输出运行记录：`RunEvent / RunTrace / artifacts`

UI 负责：

- 展示 Rust 导出的节点库
- 通过拖拽和连线编辑 workflow
- 编辑节点实例参数
- 实时生成 Rust 可用的 workflow JSON
- 在运行前调用 Rust 校验
- 展示校验结果和必要的运行状态

UI 不能自己定义 runtime 不认识的节点、端口或执行规则。UI 可以保存显示配置、默认值、分组、折叠状态、常用预设，但真正运行时必须回到 Rust 的 `Workflow` 和 `OperatorSchema`。

## 2. 工作流 JSON

画布编辑的最终产物必须是 Rust 可直接使用的 `Workflow`：

```json
{
  "id": "opencv_binary_ratio_demo",
  "name": "opencv_binary_ratio_demo",
  "version": "0.1.0",
  "nodes": [
    {
      "id": "read",
      "operator": "opencv.image.read",
      "params": {
        "path": "demo/input/sample.jpg"
      }
    }
  ],
  "edges": [
    {
      "from_node": "read",
      "from_port": "image",
      "to_node": "gray",
      "to_port": "image"
    }
  ]
}
```

UI 内部可以有额外状态，例如节点坐标、面板折叠、选中节点、缩放比例，但这些不能污染 Rust workflow JSON。建议另存为 UI metadata：

```json
{
  "workflow_id": "opencv_binary_ratio_demo",
  "node_positions": {
    "read": { "x": 120, "y": 80 }
  },
  "panels": {
    "node_library": true,
    "inspector": false,
    "json_preview": true
  }
}
```

## 3. 主界面布局

工作流页面应该以画布为中心。

推荐布局：

```text
┌──────────────────────────────────────────────────────────────┐
│ 顶部工具栏：workflow 选择 / 保存 / 校验 / 运行 / JSON 预览       │
├──────────────┬───────────────────────────────────┬───────────┤
│ 节点库        │ 画布                              │ 节点实例面板 │
│ 可折叠        │ React Flow                         │ 点击节点后出现│
│ 分类展开      │ 拖拽 / 连线 / 缩放 / 平移            │ 实例参数编辑  │
├──────────────┴───────────────────────────────────┴───────────┤
│ 底部抽屉：校验结果 / 运行日志 / trace / artifacts，可隐藏         │
└──────────────────────────────────────────────────────────────┘
```

默认状态：

- 画布最大化显示。
- 节点库默认展开，但分类折叠。
- 节点实例面板默认隐藏。
- 运行结果默认隐藏，只在校验失败或运行后以底部抽屉出现。
- JSON 预览可以作为右侧或底部抽屉，不常驻占用主区域。

## 4. 节点库

节点库来自 Rust：

```text
Tauri command: export_schema()
Rust source: vision_runtime::export_schema()
```

节点库交互：

- 按 `OperatorSchema.category` 分组。
- 分组可展开/收起。
- 支持搜索 operator id、显示名称、分类。
- 支持按大类筛选，例如输入源、OpenCV、推理、判定、记录、通信、存储、数据库。
- 每个节点类型以小卡片显示：
  - `name`
  - `id`
  - 输入端口数量
  - 输出端口数量
  - 是否有必填参数

节点库不能一次性平铺所有节点。节点数量会越来越多，平铺会很难用。

## 5. 画布

画布建议使用 React Flow。

原因：

- 已经支持节点拖拽。
- 已经支持端口连接。
- 已经支持缩放和平移。
- 已经支持选中节点和边。
- 后续可以扩展小地图、自动布局、框选、快捷键。

画布节点对应 Rust `WorkflowNode`：

```ts
type WorkflowNode = {
  id: string;
  operator: string;
  params?: Record<string, unknown>;
};
```

画布连线对应 Rust `WorkflowEdge`：

```ts
type WorkflowEdge = {
  from_node: string;
  from_port: string;
  to_node: string;
  to_port: string;
};
```

React Flow 的 node/edge 只是 UI 表达，保存时必须转换成 Rust workflow JSON。

## 6. 连接规则

连接时 UI 要做即时拦截，但最终规则来自 Rust schema。节点之间不是“随便连线”，而是“输出端口数据类型”和“输入端口数据类型”兼容后才能形成 `WorkflowEdge`。

当前 Rust schema 已导出类型化端口：

```ts
type PortSchema = {
  name: string;
  value_type: string;
  required: boolean;
};
```

当前标准数据类型：

| 类型 | 用途 |
| --- | --- |
| `image` | 原图、灰度图、旋转图、标注图等图像数据 |
| `mask` | 二值化、分割、阈值等产生的掩膜 |
| `regions` | 轮廓、连通域、blob、几何区域集合 |
| `detections` | 目标检测框、类别、置信度集合 |
| `trigger` | 外部触发、软触发、定时触发事件 |
| `capture` | 相机采集请求或采集结果上下文 |
| `signal` | OK/NG、PLC、Webhook、消息队列等输出信号 |
| `bool` | 逻辑判断结果 |
| `text` | OCR、VLM、文本消息 |
| `json` | 结构化结果、统计信息、记录、payload |
| `any` | 临时兼容类型，后续应尽量收敛成明确类型 |

UI 即时规则：

- 只能从输出端口连到输入端口。
- `from_port` 必须存在于源节点 operator 的 `output_ports`。
- `to_port` 必须存在于目标节点 operator 的 `input_ports` 或 `optional_input_ports`。
- `from_port.value_type` 必须和 `to_port.value_type` 兼容。
- 同一个输入端口只能有一条输入边，不区分必填还是可选。
- 完全重复的边不允许。
- 不能连接到不存在的节点。
- 不能形成环，除非 Rust runtime 后续明确支持循环。

Rust 最终规则：

- 重复节点 id 不允许。
- 未知 operator 不允许。
- edge 中的未知节点不允许。
- edge 中的未知输入/输出端口不允许。
- 端口数据类型不兼容不允许。
- 同一个输入端口被多条边连接不允许。
- 完全重复的边不允许。
- 必填输入端口没有连接不允许。
- 必填参数缺失不允许。
- 参数类型、枚举选项、数值范围不满足 schema 不允许。
- DAG 拓扑排序失败不允许。

运行按钮必须依赖 Rust 校验结果。校验不通过时禁止执行。

## 7. 校验设计

校验分为两层。

### 7.1 UI 即时校验

用于提升编辑体验，不代表最终可运行。

触发时机：

- 添加节点
- 删除节点
- 拖拽连线
- 删除连线
- 修改实例参数
- 保存 workflow 前

展示方式：

- 非法连接：边显示红色，提示端口不匹配。
- 缺少必填输入：节点显示警告标记。
- 缺少必填参数：节点显示警告标记，右侧参数项高亮。
- 未知 operator：节点显示错误状态。

### 7.2 Rust 最终校验

用于决定是否允许运行。

触发时机：

- 点击“校验”
- 点击“运行”前自动校验
- 保存 workflow 前可选校验

Tauri 命令：

```text
validate_workflow(workflow)
```

当前返回结构：

```json
{
  "ok": false,
  "nodes": 3,
  "edges": 2,
  "message": "workflow validation failed: 1 issue(s)",
  "issues": [
    {
      "level": "error",
      "code": "missing_required_input",
      "message": "gray missing required input image",
      "node_id": "gray",
      "edge_id": null,
      "port": "image"
    }
  ]
}
```

Rust 公共 API：

```rust
pub fn validate_workflow(workflow: &Workflow, registry: &OperatorRegistry) -> ValidationReport
```

CLI、Tauri、测试、Agent 都必须复用这一套规则。UI 可以提前做同样规则的轻量校验，但不能绕过 Rust 最终校验。

## 8. 节点实例面板

节点实例面板编辑的是 workflow 中某个节点实例的参数。

默认隐藏。

触发：

- 点击画布节点后展开。
- 取消选择后收起。

内容：

- 节点实例 id
- operator id
- 输入端口
- 输出端口
- 实例参数
- 参数 schema 说明
- 当前节点校验错误

参数表单来自 `ParamSchema`：

- `kind` 决定输入控件类型。
- `required` 决定必填校验。
- `default` 用于新建节点时填充默认值。
- `options` 用于 select。
- `min / max / step` 用于数字输入。
- `advanced` 用于默认折叠高级参数。

实例参数变化后，实时更新 workflow JSON。

## 9. 运行结果

运行结果不应该长期占用主画布空间。

推荐做成底部抽屉：

- 默认隐藏。
- 校验失败时自动展开到“校验结果”。
- 运行后自动展开到“运行结果”。
- 可手动收起。

底部抽屉内容：

- 校验错误列表
- 运行状态
- `RunEvent`
- `RunTrace`
- 节点耗时
- artifacts 列表
- 日志输出

当前阶段可以先隐藏复杂结果，仅保留：

- 运行成功/失败
- 总耗时
- FPS
- trace JSON 按需展开

## 10. JSON 预览

画布编辑后要实时生成 Rust workflow JSON。

JSON 预览建议放在可折叠面板中：

- 默认收起。
- 点击“JSON”后展开。
- 支持复制。
- 支持保存。
- 支持校验。

JSON 预览必须只显示 Rust 需要的 workflow，不混入 UI 坐标。

## 11. 节点管理

节点管理看的是节点类型，不是实例。

数据来源：

```text
Rust OperatorSchema
```

节点管理应该支持：

- 分类
- 搜索
- 筛选
- 查看 operator id
- 查看输入输出端口
- 查看参数 schema
- 查看 Rust 来源分类

节点管理中可以编辑的是 UI 层配置，不是 Rust 节点定义本身。

可以编辑：

- UI 显示名称覆盖
- 参数默认值覆盖
- 参数分组
- 常用预设
- 是否默认隐藏高级参数
- 节点颜色/图标
- 帮助说明

不能编辑：

- operator id
- Rust 真实输入端口
- Rust 真实输出端口
- Rust 运行函数
- runtime 不认识的参数规则

如果确实要新增 operator，应该回到 Rust 代码中新增节点文件、注册 schema，然后由 UI 重新同步。

## 12. 推荐实现阶段

### 阶段 1：工作流编辑器骨架

- 引入 React Flow。
- 节点库改成分类可展开。
- 画布支持拖拽节点。
- 点击节点展开实例面板。
- 运行结果面板默认隐藏。
- JSON 预览可折叠。

### 阶段 2：Workflow JSON 同步

- React Flow nodes 转 Rust `WorkflowNode`。
- React Flow edges 转 Rust `WorkflowEdge`。
- 节点实例参数编辑实时更新 JSON。
- 保存 workflow JSON。

### 阶段 3：校验

- UI 即时校验端口和必填项。
- 点击校验调用 Rust `validate_workflow`。
- 校验失败禁止运行。
- 画布节点和边显示错误状态。

### 阶段 4：节点管理

- 节点类型分类、筛选、搜索。
- 节点类型详情。
- UI 默认值和预设配置。
- 配置和 Rust schema 分离保存。

### 阶段 5：运行与结果抽屉

- 点击运行前自动校验。
- 校验通过后调用 Rust `run_workflow`。
- 底部抽屉展示运行摘要、trace、artifact。
- 复杂结果后续再做图像预览和标注叠加。

## 13. 当前优先级

最高优先级是工作流编辑闭环：

```text
Rust schema -> 节点库 -> 拖到画布 -> 连线 -> 编辑实例参数 -> 生成 workflow JSON -> Rust 校验 -> 运行
```

其他页面可以暂时弱化，避免主界面臃肿。
