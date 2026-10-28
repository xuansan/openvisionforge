import React, { useCallback, useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import {
  Background,
  ConnectionMode,
  Controls,
  Handle,
  MarkerType,
  MiniMap,
  Position,
  ReactFlow,
  applyEdgeChanges,
  applyNodeChanges,
  type Connection,
  type Edge,
  type EdgeChange,
  type Node,
  type NodeChange,
  type NodeProps,
  type OnReconnect,
  type ReactFlowInstance,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import "./styles.css";

type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };

type FileEntry = {
  name: string;
  path: string;
  size: number;
};

type ModelCatalog = {
  version?: number;
  models?: Record<string, {
    family?: string;
    version?: string;
    task?: string;
    format?: string;
    status?: string;
    path?: string;
    size_bytes?: number;
    sha256?: string;
    input_width?: number;
    input_height?: number;
    recommended_backend?: string;
    validated_at?: string;
  }>;
};

type ParamSchema = {
  key: string;
  label: string;
  kind: string;
  required: boolean;
  default?: string | null;
  description: string;
  options: string[];
  group?: string | null;
  order: number;
  advanced: boolean;
  unit?: string | null;
  placeholder?: string | null;
  min?: number | null;
  max?: number | null;
  step?: number | null;
  accept: string[];
  deployment_required: boolean;
  overridable: boolean;
};

type PortSchema = {
  name: string;
  value_type: string;
  required: boolean;
};

type OperatorSchema = {
  id: string;
  name: string;
  category: string;
  inputs: string[];
  optional_inputs: string[];
  outputs: string[];
  input_ports?: PortSchema[];
  optional_input_ports?: PortSchema[];
  output_ports?: PortSchema[];
  params: ParamSchema[];
};

type RuntimeStatus = {
  version: string;
  schema_count: number;
  runtime: string;
  project_root: string;
};

type WorkflowNode = {
  id: string;
  operator: string;
  params?: Record<string, JsonValue> | null;
  execution?: {
    run_when?: "always" | "all_ok" | "any_ok" | "all_ng" | "any_ng";
    always_run?: boolean;
    short_circuit_on_ng?: boolean;
  };
};

type WorkflowEdge = {
  from_node: string;
  from_port: string;
  to_node: string;
  to_port: string;
};

type Workflow = {
  id: string;
  name: string;
  version: string;
  metadata?: Record<string, JsonValue>;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
};

type WorkflowUiMetadata = {
  workflow_id: string;
  node_positions: Record<string, { x: number; y: number }>;
};

type PortTrace = {
  port: string;
  value: string;
};

type NodeTrace = {
  node_id: string;
  operator: string;
  inputs: PortTrace[];
  outputs: PortTrace[];
  status: string;
  metrics: {
    duration_ms: number;
    input_images: number;
    output_images: number;
  };
};

type RunTrace = {
  run_id: string;
  workflow_id: string;
  nodes: NodeTrace[];
  metrics: {
    total_duration_ms: number;
    processed_frames: number;
    image_outputs: number;
    fps: number;
  };
};

type RunEvent = {
  run_id: string;
  workflow_id: string;
  status: string;
  result: string;
  metrics: RunTrace["metrics"];
};

type RunResponse = {
  event: RunEvent;
  trace: RunTrace;
  record_dir: string;
  artifacts: FileEntry[];
};

type ArtifactContent = {
  path: string;
  kind: string;
  text?: string | null;
  data_url?: string | null;
};

type RecordEntry = {
  run_id: string;
  workflow_id: string;
  result: string;
  status: string;
  total_duration_ms: number;
  fps: number;
  path: string;
};

type NodeSmokeOperatorStatus = {
  id: string;
  category: string;
  name: string;
  status: "PASS" | "FAIL" | string;
  exit_code: number;
  ms: number;
  message: string;
};

type NodeSmokeStatus = {
  total: number;
  pass: number;
  fail: number;
  source?: string | null;
  operators: NodeSmokeOperatorStatus[];
};

type ValidateResult = {
  ok: boolean;
  nodes: number;
  edges: number;
  message: string;
  issues?: RustValidationIssue[];
};

type RustValidationIssue = {
  level: "error" | "warning";
  code: string;
  message: string;
  node_id?: string | null;
  edge_id?: string | null;
  port?: string | null;
};

type ValidationIssue = {
  level: "error" | "warning";
  scope: "workflow" | "node" | "edge" | "param";
  nodeId?: string;
  edgeId?: string;
  message: string;
};

type NodeUiConfig = {
  displayName?: string;
  defaultParams: Record<string, JsonValue>;
  hiddenAdvanced: boolean;
};

type WorkflowSource = {
  kind: "example" | "studio" | "imported" | "new";
  label: string;
  path?: string;
};

type CommandState = {
  action: string;
  state: "idle" | "running" | "ok" | "error";
  message: string;
};

type ViewId = "workflow" | "nodes" | "models" | "records" | "data" | "devices" | "environment" | "settings" | "about";

const emptyWorkflow: Workflow = {
  id: "new_workflow",
  name: "new_workflow",
  version: "0.1.0",
  nodes: [],
  edges: [],
};

const navItems: Array<{ id: ViewId; label: string }> = [
  { id: "workflow", label: "工作流" },
  { id: "nodes", label: "节点管理" },
  { id: "models", label: "模型" },
  { id: "records", label: "运行记录" },
  { id: "data", label: "数据" },
  { id: "devices", label: "设备" },
  { id: "environment", label: "环境" },
  { id: "settings", label: "设置" },
  { id: "about", label: "关于" },
];

const categoryMap: Record<string, string> = {
  Source: "输入源",
  Trigger: "触发",
  OpenCv: "OpenCV",
  Infer: "推理",
  Ai: "AI",
  Judge: "判定",
  Logic: "逻辑",
  Record: "记录",
  Output: "输出",
  Communication: "通信",
  Storage: "存储",
  Database: "数据库",
  Action: "动作",
};

const nodeTypes = {
  workflowNode: WorkflowFlowNode,
};

function App() {
  const [activeView, setActiveView] = useState<ViewId>("workflow");
  const [operators, setOperators] = useState<OperatorSchema[]>([]);
  const [status, setStatus] = useState<RuntimeStatus | null>(null);
  const [examples, setExamples] = useState<FileEntry[]>([]);
  const [studioWorkflows, setStudioWorkflows] = useState<FileEntry[]>([]);
  const [models, setModels] = useState<FileEntry[]>([]);
  const [modelCatalog, setModelCatalog] = useState<ModelCatalog | null>(null);
  const [records, setRecords] = useState<RecordEntry[]>([]);
  const [nodeSmokeStatus, setNodeSmokeStatus] = useState<NodeSmokeStatus | null>(null);
  const [workflow, setWorkflow] = useState<Workflow>(emptyWorkflow);
  const [flowNodes, setFlowNodes] = useState<Node[]>([]);
  const [flowEdges, setFlowEdges] = useState<Edge[]>([]);
  const [selectedNodeId, setSelectedNodeId] = useState("");
  const [selectedOperatorId, setSelectedOperatorId] = useState("");
  const [nodeQuery, setNodeQuery] = useState("");
  const [runResponse, setRunResponse] = useState<RunResponse | null>(null);
  const [inspectorOpen, setInspectorOpen] = useState(false);
  const [libraryOpen, setLibraryOpen] = useState(true);
  const [mainNavOpen, setMainNavOpen] = useState(true);
  const [resultOpen, setResultOpen] = useState(false);
  const [jsonOpen, setJsonOpen] = useState(false);
  const [rustValidationOk, setRustValidationOk] = useState(false);
  const [validationIssues, setValidationIssues] = useState<ValidationIssue[]>([]);
  const [nodeUiConfig, setNodeUiConfig] = useState<Record<string, NodeUiConfig>>({});
  const [workflowSource, setWorkflowSource] = useState<WorkflowSource>({ kind: "new", label: "未保存 workflow" });
  const [workflowDirty, setWorkflowDirty] = useState(false);
  const [command, setCommand] = useState<CommandState>({
    action: "空闲",
    state: "idle",
    message: "等待加载、保存、校验或运行 workflow。",
  });
  const [toast, setToast] = useState("");

  useEffect(() => {
    void boot();
  }, []);

  async function boot() {
    setCommand({ action: "启动", state: "running", message: "正在读取 Rust runtime schema、示例流程、模型和运行记录..." });
    try {
      const [schema, runtime, exampleFiles, modelFiles, catalog, recordFiles, smokeStatus] = await Promise.all([
        invoke<OperatorSchema[]>("export_schema"),
        invoke<RuntimeStatus>("runtime_status"),
        invoke<FileEntry[]>("list_examples"),
        invoke<FileEntry[]>("list_models"),
        invoke<ModelCatalog>("model_catalog"),
        invoke<RecordEntry[]>("list_records"),
        invoke<NodeSmokeStatus>("node_smoke_status"),
      ]);
      const savedWorkflowFiles = await invoke<FileEntry[]>("list_studio_workflows");
      setOperators(schema);
      setStatus(runtime);
      setExamples(exampleFiles);
      setStudioWorkflows(savedWorkflowFiles);
      setModels(modelFiles);
      setModelCatalog(catalog);
      setRecords(recordFiles);
      setNodeSmokeStatus(smokeStatus);
      setSelectedOperatorId(schema[0]?.id || "");
      const preferred = exampleFiles.find((item) => item.name === "opencv_binary_ratio_demo.json") || exampleFiles[0];
      if (preferred) {
        await loadWorkflow(preferred.path, false);
      }
      setCommand({ action: "启动", state: "ok", message: `已加载 ${schema.length} 个节点类型、${exampleFiles.length} 个示例 workflow、${savedWorkflowFiles.length} 个已保存 workflow。` });
    } catch (err) {
      fail("启动", err);
    }
  }

  function notify(message: string) {
    setToast(message);
    window.setTimeout(() => setToast(""), 2200);
  }

  function fail(action: string, err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    setCommand({ action, state: "error", message });
    notify(`${action}失败`);
  }

  function confirmDiscardChanges() {
    return !workflowDirty || window.confirm("当前 workflow 有未保存修改，继续操作会覆盖画布。是否继续？");
  }

  function applyWorkflow(
    nextWorkflow: Workflow,
    source: WorkflowSource,
    metadata?: WorkflowUiMetadata,
    dirty = false,
  ) {
    const normalized = normalizeWorkflow(nextWorkflow);
    setWorkflow(normalized);
    syncFlowFromWorkflow(normalized, operators, metadata);
    setSelectedNodeId(normalized.nodes[0]?.id || "");
    setInspectorOpen(false);
    setRunResponse(null);
    setRustValidationOk(false);
    setResultOpen(false);
    setWorkflowSource(source);
    setWorkflowDirty(dirty);
  }

  async function loadWorkflow(path: string, announce = true) {
    if (announce && !confirmDiscardChanges()) return;
    if (announce) setCommand({ action: "加载", state: "running", message: `正在加载 ${path}` });
    try {
      const loaded = await invoke<Workflow>("load_workflow", { path });
      const metadata = await invoke<WorkflowUiMetadata | null>("load_workflow_ui_metadata", { path });
      applyWorkflow(loaded, { kind: path.includes("studio_workflows") ? "studio" : "example", label: path, path }, metadata || undefined);
      if (announce) {
        setCommand({ action: "加载", state: "ok", message: `已加载 ${loaded.name}，${loaded.nodes.length} 个节点。` });
        notify("workflow 已加载");
      }
    } catch (err) {
      fail("加载", err);
    }
  }

  async function openWorkflowFile(file: File) {
    if (!confirmDiscardChanges()) return;
    setCommand({ action: "加载", state: "running", message: `正在打开本地文件 ${file.name}` });
    try {
      const text = await file.text();
      const loaded = JSON.parse(text) as Workflow;
      applyWorkflow(loaded, { kind: "imported", label: `导入：${file.name}` }, undefined, true);
      setCommand({ action: "加载", state: "ok", message: `已打开本地 workflow：${file.name}` });
      notify("本地 workflow 已加载");
    } catch (err) {
      fail("加载", err);
    }
  }

  async function saveWorkflow() {
    setCommand({ action: "保存", state: "running", message: "正在保存 workflow 到 studio_workflows..." });
    try {
      const saved = await invoke<FileEntry>("save_workflow", { workflow, uiMetadata: workflowUiMetadata(workflow, flowNodes) });
      setStudioWorkflows(await invoke<FileEntry[]>("list_studio_workflows"));
      setWorkflowSource({ kind: "studio", label: saved.path, path: saved.path });
      setWorkflowDirty(false);
      setCommand({ action: "保存", state: "ok", message: `已保存到 ${saved.path}` });
      notify("workflow 已保存");
    } catch (err) {
      fail("保存", err);
    }
  }

  async function saveWorkflowAs() {
    const raw = window.prompt("输入新的 workflow id / 文件名", `${workflow.id}_copy`);
    if (!raw) return;
    const id = sanitizeWorkflowId(raw);
    if (!id) {
      notify("workflow id 不能为空");
      return;
    }
    const nextWorkflow = { ...workflow, id, name: id };
    setCommand({ action: "另存为", state: "running", message: `正在保存为 studio_workflows/${id}.json...` });
    try {
      const saved = await invoke<FileEntry>("save_workflow", {
        workflow: nextWorkflow,
        uiMetadata: workflowUiMetadata(nextWorkflow, flowNodes),
      });
      setWorkflow(nextWorkflow);
      setStudioWorkflows(await invoke<FileEntry[]>("list_studio_workflows"));
      setWorkflowSource({ kind: "studio", label: saved.path, path: saved.path });
      setWorkflowDirty(false);
      setCommand({ action: "另存为", state: "ok", message: `已另存为 ${saved.path}` });
      notify("workflow 已另存为新文件");
    } catch (err) {
      fail("另存为", err);
    }
  }

  function newWorkflow() {
    if (!confirmDiscardChanges()) return;
    const id = `workflow_${new Date().toISOString().replace(/[-:.TZ]/g, "").slice(0, 14)}`;
    const nextWorkflow = { ...emptyWorkflow, id, name: id, nodes: [], edges: [] };
    setWorkflow(nextWorkflow);
    setFlowNodes([]);
    setFlowEdges([]);
    setSelectedNodeId("");
    setRunResponse(null);
    setRustValidationOk(false);
    setResultOpen(false);
    setWorkflowSource({ kind: "new", label: "未保存 workflow" });
    setWorkflowDirty(true);
    setCommand({ action: "新建", state: "ok", message: `已新建空白 workflow：${id}` });
    notify("已新建 workflow");
  }

  async function validateWorkflowByRust(nextWorkflow: Workflow, revealIssues = false): Promise<ValidateResult> {
    const result = await invoke<ValidateResult>("validate_workflow", { workflow: nextWorkflow });
    const rustIssues = rustIssuesToUi(result.issues || []);
    setRustValidationOk(result.ok);
    setValidationIssues(rustIssues);
    if (revealIssues && rustIssues.length) setResultOpen(true);
    return result;
  }

  async function validateWorkflow() {
    const localIssues: ValidationIssue[] = [];
    setValidationIssues(localIssues);
    setResultOpen(localIssues.length > 0);
    if (localIssues.some((issue) => issue.level === "error")) {
      setRustValidationOk(false);
      setCommand({
        action: "校验",
        state: "error",
        message: `UI 即时校验失败：${localIssues.filter((issue) => issue.level === "error").length} 个错误。`,
      });
      notify("校验失败");
      return;
    }
    setCommand({ action: "校验", state: "running", message: "正在做纯结构校验，不执行节点。" });
    try {
      const result = await validateWorkflowByRust(workflow, true);
      setCommand({
        action: "校验",
        state: result.ok ? "ok" : "error",
        message: `${result.message}：${result.nodes} 个节点，${result.edges} 条连接。`,
      });
      notify(result.ok ? "校验通过" : "校验失败");
    } catch (err) {
      fail("校验", err);
    }
  }

  async function runWorkflow() {
    const localIssues: ValidationIssue[] = [];
    setValidationIssues(localIssues);
    if (localIssues.some((issue) => issue.level === "error")) {
      setRustValidationOk(false);
      setResultOpen(true);
      setCommand({
        action: "运行",
        state: "error",
        message: "运行前校验失败：请先修复画布连接、必填输入或必填参数。",
      });
      notify("运行被阻止");
      return;
    }
    const result = await validateWorkflowByRust(workflow, true);
    if (!result.ok) {
      setValidationIssues(rustIssuesToUi(result.issues || [{ level: "error", code: "rust_validation_failed", message: result.message }]));
      setResultOpen(true);
      setCommand({ action: "运行", state: "error", message: `Rust 校验失败：${result.message}` });
      notify("运行被阻止");
      return;
    }
    setCommand({ action: "运行", state: "running", message: "正在调用 vision-core Executor 运行当前 workflow..." });
    try {
      const response = await invoke<RunResponse>("run_workflow", { workflow });
      setRunResponse(response);
      setResultOpen(true);
      setRecords(await invoke<RecordEntry[]>("list_records"));
      setCommand({
        action: "运行",
        state: response.event.status === "finished" ? "ok" : "error",
        message: `${response.event.run_id}：${formatRunResult(response.event.result)}，${response.event.metrics.total_duration_ms} ms，FPS ${response.event.metrics.fps}`,
      });
      notify(`运行完成：${formatRunResult(response.event.result)}`);
    } catch (err) {
      fail("运行", err);
    }
  }

  const selectedNode = workflow.nodes.find((node) => node.id === selectedNodeId) || workflow.nodes[0];
  const selectedSchema = selectedNode ? operators.find((operator) => operator.id === selectedNode.operator) || null : null;

  useEffect(() => {
    syncFlowFromWorkflow(workflow, operators);
  }, [operators.length]);

  useEffect(() => {
    let cancelled = false;
    const timeout = window.setTimeout(async () => {
      try {
        const result = await invoke<ValidateResult>("validate_workflow", { workflow });
        if (cancelled) return;
        setRustValidationOk(result.ok);
        setValidationIssues(rustIssuesToUi(result.issues || []));
      } catch {
        if (cancelled) return;
        setRustValidationOk(false);
      }
    }, 250);
    return () => {
      cancelled = true;
      window.clearTimeout(timeout);
    };
  }, [workflow, operators]);

  useEffect(() => {
    const handler = (event: BeforeUnloadEvent) => {
      if (!workflowDirty) return;
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", handler);
    return () => window.removeEventListener("beforeunload", handler);
  }, [workflowDirty]);

  function syncFlowFromWorkflow(nextWorkflow: Workflow, nextOperators = operators, metadata?: WorkflowUiMetadata) {
    setFlowNodes(workflowToFlowNodes(nextWorkflow, nextOperators, metadata));
    setFlowEdges(workflowToFlowEdges(nextWorkflow));
  }

  const updateWorkflowState: React.Dispatch<React.SetStateAction<Workflow>> = (value) => {
    setWorkflow((current) => (typeof value === "function" ? value(current) : value));
    setWorkflowDirty(true);
    setRustValidationOk(false);
  };

  const updateFlowNodesState: React.Dispatch<React.SetStateAction<Node[]>> = (value) => {
    setFlowNodes((current) => (typeof value === "function" ? value(current) : value));
    setWorkflowDirty(true);
  };

  const updateFlowEdgesState: React.Dispatch<React.SetStateAction<Edge[]>> = (value) => {
    setFlowEdges((current) => (typeof value === "function" ? value(current) : value));
    setWorkflowDirty(true);
  };

  function updateNodeParam(nodeId: string, key: string, value: JsonValue) {
    updateWorkflowState((current) => ({
      ...current,
      nodes: current.nodes.map((node) =>
        node.id === nodeId
          ? { ...node, params: { ...((node.params as Record<string, JsonValue>) || {}), [key]: value } }
          : node,
      ),
    }));
  }

  function updateNodeExecution(
    nodeId: string,
    patch: NonNullable<WorkflowNode["execution"]>,
  ) {
    updateWorkflowState((current) => ({
      ...current,
      nodes: current.nodes.map((node) =>
        node.id === nodeId
          ? { ...node, execution: { ...(node.execution || {}), ...patch } }
          : node,
      ),
    }));
  }

  function updateNodeUiConfig(operatorId: string, patch: Partial<NodeUiConfig>) {
    setNodeUiConfig((current) => {
      const existing = current[operatorId] || { defaultParams: {}, hiddenAdvanced: true };
      return { ...current, [operatorId]: { ...existing, ...patch } };
    });
  }

  function updateNodeDefaultParam(operatorId: string, key: string, value: JsonValue) {
    setNodeUiConfig((current) => {
      const existing = current[operatorId] || { defaultParams: {}, hiddenAdvanced: true };
      return {
        ...current,
        [operatorId]: {
          ...existing,
          defaultParams: { ...existing.defaultParams, [key]: value },
        },
      };
    });
  }

  return (
    <div className={`app-shell ${mainNavOpen ? "" : "main-nav-collapsed"}`}>
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">视</div>
          <div className="brand-text">
            <div className="brand-title">Vision Runtime Studio</div>
            <div className="brand-subtitle">Workflow / Schema / Trace</div>
          </div>
        </div>
        <button className="sidebar-toggle" onClick={() => setMainNavOpen((current) => !current)}>
          {mainNavOpen ? "收起菜单" : "展开"}
        </button>
        <nav className="main-nav" aria-label="主导航">
          {navItems.map((item) => (
            <button key={item.id} title={item.label} className={activeView === item.id ? "nav-item active" : "nav-item"} onClick={() => setActiveView(item.id)}>
              <strong>{item.label.slice(0, 1)}</strong>
              <span>{item.label}</span>
            </button>
          ))}
        </nav>
        <div className="sidebar-status">
          <div className="status-row"><span className="status-dot ok"></span><span>{status?.runtime || "vision-runtime"}</span></div>
          <div className="status-row"><span className="status-dot ok"></span><span>节点 <strong>{operators.length}</strong></span></div>
          <div className="status-row"><span className="status-dot warn"></span><span>工作流 <strong>{examples.length}</strong></span></div>
          <div className="version-block"><span>版本</span><strong>v{status?.version || "0.1.0"}-alpha</strong></div>
        </div>
      </aside>

      <main className="main">
        <header className="topbar">
          <div className="project-block">
            <div className="eyebrow">当前 Workflow</div>
            <div className="project-name">{workflow.name}</div>
            <div className="project-source">{workflowDirty ? "未保存 · " : ""}{workflowSource.label}</div>
          </div>
          <div className="recipe-switcher">
            <span>示例</span>
            <select value={examples.find((item) => item.name === `${workflow.id}.json`)?.path || ""} onChange={(event) => event.target.value && loadWorkflow(event.target.value)}>
              <option value="">选择 workflow JSON</option>
              {examples.map((item) => <option key={item.path} value={item.path}>{item.name}</option>)}
            </select>
          </div>
          <div className="recipe-switcher">
            <span>已保存</span>
            <select value={studioWorkflows.find((item) => item.name === `${workflow.id}.json`)?.path || ""} onChange={(event) => event.target.value && loadWorkflow(event.target.value)}>
              <option value="">选择已保存 workflow</option>
              {studioWorkflows.map((item) => <option key={item.path} value={item.path}>{item.name}</option>)}
            </select>
          </div>
          <div className="topbar-actions">
            <button className="ghost-button" onClick={newWorkflow} disabled={command.state === "running"}>新建</button>
            <button className="ghost-button" onClick={saveWorkflow} disabled={command.state === "running"}>保存</button>
            <button className="ghost-button" onClick={saveWorkflowAs} disabled={command.state === "running"}>另存为</button>
            <button className="ghost-button" onClick={validateWorkflow} disabled={command.state === "running"}>校验</button>
            <button className="primary-button" onClick={runWorkflow} disabled={command.state === "running" || !workflow.nodes.length}>
              {command.action === "运行" && command.state === "running" ? "运行中" : rustValidationOk ? "运行" : "校验后运行"}
            </button>
            <button className="danger-button" onClick={() => notify("当前版本还没有长任务停止句柄。")}>停止</button>
          </div>
        </header>

        <CommandStatusBar command={command} runResponse={runResponse} />
        {toast && <div className={command.state === "error" ? "toast show ng" : "toast show ok"}>{toast}</div>}

        <section className={activeView === "workflow" ? "view active" : "view"}>
          <WorkflowPage
            workflow={workflow}
            operators={operators}
            flowNodes={flowNodes}
            flowEdges={flowEdges}
            selectedNode={selectedNode}
            selectedSchema={selectedSchema}
            examples={examples}
            studioWorkflows={studioWorkflows}
            nodeSmokeStatus={nodeSmokeStatus}
            selectedOperatorId={selectedOperatorId}
            nodeQuery={nodeQuery}
            runResponse={runResponse}
            issues={validationIssues}
            inspectorOpen={inspectorOpen}
            libraryOpen={libraryOpen}
            resultOpen={resultOpen}
            jsonOpen={jsonOpen}
            onNodeQuery={setNodeQuery}
            onSelectNode={setSelectedNodeId}
            onSelectOperator={setSelectedOperatorId}
            onLoadWorkflow={loadWorkflow}
            onNewWorkflow={newWorkflow}
            onSaveWorkflow={saveWorkflow}
            onSaveWorkflowAs={saveWorkflowAs}
            onOpenWorkflowFile={openWorkflowFile}
            onValidateWorkflow={validateWorkflowByRust}
            onParamChange={updateNodeParam}
            onExecutionChange={updateNodeExecution}
            onWorkflowChange={updateWorkflowState}
            onFlowNodesChange={updateFlowNodesState}
            onFlowEdgesChange={updateFlowEdgesState}
            onInspectorOpen={setInspectorOpen}
            onLibraryOpen={setLibraryOpen}
            onResultOpen={setResultOpen}
            onJsonOpen={setJsonOpen}
          />
        </section>
        <section className={activeView === "nodes" ? "view active" : "view"}>
          <NodeManager
            operators={operators}
            selectedOperatorId={selectedOperatorId}
            nodeUiConfig={nodeUiConfig}
            nodeSmokeStatus={nodeSmokeStatus}
            onSelect={setSelectedOperatorId}
            onUiConfigChange={updateNodeUiConfig}
            onDefaultParamChange={updateNodeDefaultParam}
          />
        </section>
        <section className={activeView === "models" ? "view active" : "view"}>
          <ModelsPage models={models} catalog={modelCatalog} operators={operators} />
        </section>
        <section className={activeView === "records" ? "view active" : "view"}>
          <RecordsPage records={records} runResponse={runResponse} />
        </section>
        <section className={activeView === "data" ? "view active" : "view"}>
          <DataPage examples={examples} records={records} runResponse={runResponse} />
        </section>
        <section className={activeView === "devices" ? "view active" : "view"}>
          <DevicesPage operators={operators} />
        </section>
        <section className={activeView === "environment" ? "view active" : "view"}>
          <EnvironmentPage status={status} operators={operators} models={models} records={records} />
        </section>
        <section className={activeView === "settings" ? "view active" : "view"}>
          <SettingsPage status={status} />
        </section>
        <section className={activeView === "about" ? "view active" : "view"}>
          <AboutPage status={status} />
        </section>
      </main>
    </div>
  );
}

function CommandStatusBar({ command, runResponse }: { command: CommandState; runResponse: RunResponse | null }) {
  const label = command.state === "running" ? "执行中" : command.state === "ok" ? "成功" : command.state === "error" ? "异常/NG" : "待命";
  return (
    <div className={`command-status-bar ${command.state}`}>
      <div><span>{command.action}</span><strong>{label}</strong></div>
      <p>{command.message}</p>
      {runResponse ? (
        <div className="command-kpis">
          <span>Run ID <strong>{runResponse.event.run_id}</strong></span>
          <span>结果 <strong>{formatRunResult(runResponse.event.result)}</strong></span>
          <span>产物 <strong>{runResponse.artifacts.length}</strong></span>
          <span>FPS <strong>{runResponse.event.metrics.fps}</strong></span>
        </div>
      ) : null}
    </div>
  );
}

function WorkflowPage(props: {
  workflow: Workflow;
  operators: OperatorSchema[];
  flowNodes: Node[];
  flowEdges: Edge[];
  selectedNode?: WorkflowNode;
  selectedSchema: OperatorSchema | null;
  examples: FileEntry[];
  studioWorkflows: FileEntry[];
  nodeSmokeStatus: NodeSmokeStatus | null;
  selectedOperatorId: string;
  nodeQuery: string;
  runResponse: RunResponse | null;
  issues: ValidationIssue[];
  inspectorOpen: boolean;
  libraryOpen: boolean;
  resultOpen: boolean;
  jsonOpen: boolean;
  onNodeQuery: (query: string) => void;
  onSelectNode: (id: string) => void;
  onSelectOperator: (id: string) => void;
  onLoadWorkflow: (path: string) => void;
  onNewWorkflow: () => void;
  onSaveWorkflow: () => void;
  onSaveWorkflowAs: () => void;
  onOpenWorkflowFile: (file: File) => void;
  onValidateWorkflow: (workflow: Workflow) => Promise<ValidateResult>;
  onParamChange: (nodeId: string, key: string, value: JsonValue) => void;
  onExecutionChange: (nodeId: string, patch: NonNullable<WorkflowNode["execution"]>) => void;
  onWorkflowChange: React.Dispatch<React.SetStateAction<Workflow>>;
  onFlowNodesChange: React.Dispatch<React.SetStateAction<Node[]>>;
  onFlowEdgesChange: React.Dispatch<React.SetStateAction<Edge[]>>;
  onInspectorOpen: (open: boolean) => void;
  onLibraryOpen: (open: boolean) => void;
  onResultOpen: (open: boolean) => void;
  onJsonOpen: (open: boolean) => void;
}) {
  return (
    <div className={`runtime-workspace flow-editor ${props.libraryOpen ? "" : "library-collapsed"} ${props.inspectorOpen ? "" : "inspector-collapsed"}`}>
      <NodeLibrary
        operators={props.operators}
        examples={props.examples}
        studioWorkflows={props.studioWorkflows}
        nodeSmokeStatus={props.nodeSmokeStatus}
        query={props.nodeQuery}
        selectedOperatorId={props.selectedOperatorId}
        open={props.libraryOpen}
        onOpen={props.onLibraryOpen}
        onQuery={props.onNodeQuery}
        onSelect={props.onSelectOperator}
        onLoadWorkflow={props.onLoadWorkflow}
        onNewWorkflow={props.onNewWorkflow}
        onSaveWorkflow={props.onSaveWorkflow}
        onSaveWorkflowAs={props.onSaveWorkflowAs}
        onOpenWorkflowFile={props.onOpenWorkflowFile}
      />
      <WorkflowCanvas
        workflow={props.workflow}
        operators={props.operators}
        nodes={props.flowNodes}
        edges={props.flowEdges}
        selectedOperatorId={props.selectedOperatorId}
        selectedNodeId={props.selectedNode?.id || ""}
        issues={props.issues}
        trace={props.runResponse?.trace || null}
        onSelectNode={(id) => {
          props.onSelectNode(id);
          props.onInspectorOpen(true);
        }}
        onValidateWorkflow={props.onValidateWorkflow}
        onWorkflowChange={props.onWorkflowChange}
        onNodesChange={props.onFlowNodesChange}
        onEdgesChange={props.onFlowEdgesChange}
      />
      {props.inspectorOpen ? (
          <NodeInspector
          workflow={props.workflow}
          node={props.selectedNode}
          schema={props.selectedSchema}
          runResponse={props.runResponse}
          onClose={() => props.onInspectorOpen(false)}
          onParamChange={props.onParamChange}
          onExecutionChange={props.onExecutionChange}
        />
      ) : (
        <button className="side-tab right-tab" onClick={() => props.onInspectorOpen(true)}>节点实例</button>
      )}
      <BottomDrawer
        workflow={props.workflow}
        runResponse={props.runResponse}
        issues={props.issues}
        resultOpen={props.resultOpen}
        jsonOpen={props.jsonOpen}
        onResultOpen={props.onResultOpen}
        onJsonOpen={props.onJsonOpen}
      />
    </div>
  );
}

function NodeLibrary({
  operators,
  examples,
  studioWorkflows,
  nodeSmokeStatus,
  query,
  selectedOperatorId,
  open,
  onOpen,
  onQuery,
  onSelect,
  onLoadWorkflow,
  onNewWorkflow,
  onSaveWorkflow,
  onSaveWorkflowAs,
  onOpenWorkflowFile,
}: {
  operators: OperatorSchema[];
  examples: FileEntry[];
  studioWorkflows: FileEntry[];
  nodeSmokeStatus: NodeSmokeStatus | null;
  query: string;
  selectedOperatorId: string;
  open: boolean;
  onOpen: (open: boolean) => void;
  onQuery: (query: string) => void;
  onSelect: (id: string) => void;
  onLoadWorkflow: (path: string) => void;
  onNewWorkflow: () => void;
  onSaveWorkflow: () => void;
  onSaveWorkflowAs: () => void;
  onOpenWorkflowFile: (file: File) => void;
}) {
  const [openGroups, setOpenGroups] = useState<Record<string, boolean>>({});
  const [filesOpen, setFilesOpen] = useState(true);
  const [nodeTypesOpen, setNodeTypesOpen] = useState(true);
  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return operators.filter((operator) => !needle || `${operator.id} ${operator.name} ${operator.category}`.toLowerCase().includes(needle));
  }, [operators, query]);
  const groups = groupBy(filtered, (operator) => rootCategory(operator.category));
  const groupEntries = Object.entries(groups);
  const smokeById = smokeStatusMap(nodeSmokeStatus);
  if (!open) {
    return <button className="side-tab left-tab" onClick={() => onOpen(true)}>节点库</button>;
  }
  return (
    <section className="panel node-library">
      <div className="panel-header">
        <div><h2>节点库</h2><p>来自 Rust OperatorSchema；拖到画布后成为节点实例。</p></div>
        <SmokeSummaryBadge status={nodeSmokeStatus} />
        <button className="ghost-button small" onClick={() => onOpen(false)}>收起</button>
      </div>
      <div className="workflow-file-panel">
        <button className="section-toggle" onClick={() => setFilesOpen((current) => !current)}>
          <strong>{filesOpen ? "▾" : "▸"} 流程文件</strong><span>{examples.length + studioWorkflows.length}</span>
        </button>
        {filesOpen ? (
          <div className="section-body">
            <p>保存到 studio_workflows，每个 workflow 一个独立 JSON 文件。</p>
            <div className="workflow-file-buttons">
              <label className="ghost-button small file-open-button">
                导入JSON
                <input
                  type="file"
                  accept=".json,application/json"
                  onChange={(event) => {
                    const file = event.target.files?.[0];
                    if (file) onOpenWorkflowFile(file);
                    event.currentTarget.value = "";
                  }}
                />
              </label>
              <button className="ghost-button small" onClick={onNewWorkflow}>新建</button>
              <button className="ghost-button small" onClick={onSaveWorkflow}>保存</button>
              <button className="ghost-button small" onClick={onSaveWorkflowAs}>另存为</button>
            </div>
            <select onChange={(event) => event.target.value && onLoadWorkflow(event.target.value)} defaultValue="">
              <option value="">加载 workflow</option>
              {studioWorkflows.length ? <option disabled>-- 已保存 --</option> : null}
              {studioWorkflows.map((item) => <option key={item.path} value={item.path}>{item.name}</option>)}
              {examples.length ? <option disabled>-- 示例 --</option> : null}
              {examples.map((item) => <option key={item.path} value={item.path}>{item.name}</option>)}
            </select>
          </div>
        ) : null}
      </div>
      <div className="workflow-file-panel node-type-panel">
        <button className="section-toggle" onClick={() => setNodeTypesOpen((current) => !current)}>
          <strong>{nodeTypesOpen ? "▾" : "▸"} 节点类型</strong><span>{filtered.length}</span>
        </button>
        {nodeTypesOpen ? (
          <div className="section-body">
            <input className="search-input" value={query} onChange={(event) => onQuery(event.target.value)} placeholder="搜索 operator id / 分类 / 名称" />
            <div className="node-list">
              {groupEntries.map(([category, items]) => {
                const isOpen = openGroups[category] ?? groupEntries.length <= 4;
                return (
                <div className="node-group" key={category}>
                  <button className="node-group-title" onClick={() => setOpenGroups((current) => ({ ...current, [category]: !isOpen }))}>
                    <strong>{isOpen ? "▾" : "▸"} {categoryLabel(category)}</strong><span>{items.length}</span>
                  </button>
                  {isOpen && <div className="node-group-body">
                    {items.map((operator) => (
                      <button
                        key={operator.id}
                        className={operator.id === selectedOperatorId ? "node-card active" : "node-card"}
                        draggable
                        onDragStart={(event) => {
                          event.dataTransfer.setData("application/vision-operator", operator.id);
                          event.dataTransfer.setData("text/plain", operator.id);
                          event.dataTransfer.effectAllowed = "copy";
                        }}
                        onClick={() => onSelect(operator.id)}
                      >
                        <strong>{operator.name}<SmokeBadge status={smokeById.get(operator.id)?.status} /></strong>
                        <span>{operator.id}</span>
                        <em>{operator.inputs.length} 入 / {operator.outputs.length} 出</em>
                      </button>
                    ))}
                  </div>}
                </div>
              )})}
            </div>
          </div>
        ) : null}
      </div>
    </section>
  );
}

function WorkflowCanvas({
  workflow,
  operators,
  nodes,
  edges,
  selectedOperatorId,
  selectedNodeId,
  issues,
  trace,
  onSelectNode,
  onValidateWorkflow,
  onWorkflowChange,
  onNodesChange,
  onEdgesChange,
}: {
  workflow: Workflow;
  operators: OperatorSchema[];
  nodes: Node[];
  edges: Edge[];
  selectedOperatorId: string;
  selectedNodeId: string;
  issues: ValidationIssue[];
  trace: RunTrace | null;
  onSelectNode: (id: string) => void;
  onValidateWorkflow: (workflow: Workflow) => Promise<ValidateResult>;
  onWorkflowChange: React.Dispatch<React.SetStateAction<Workflow>>;
  onNodesChange: React.Dispatch<React.SetStateAction<Node[]>>;
  onEdgesChange: React.Dispatch<React.SetStateAction<Edge[]>>;
}) {
  const [reactFlow, setReactFlow] = useState<ReactFlowInstance | null>(null);
  const [canvasNotice, setCanvasNotice] = useState("");
  const [selectedEdgeId, setSelectedEdgeId] = useState("");
  const nodeTrace = new Map((trace?.nodes || []).map((item) => [item.node_id, item]));
  const issueMap = new Map<string, ValidationIssue[]>();
  for (const issue of issues) {
    if (issue.nodeId) issueMap.set(issue.nodeId, [...(issueMap.get(issue.nodeId) || []), issue]);
    if (issue.edgeId) issueMap.set(issue.edgeId, [...(issueMap.get(issue.edgeId) || []), issue]);
  }
  const graphNodes = nodes.map((node) => {
    const workflowNode = workflow.nodes.find((item) => item.id === node.id);
    const schema = workflowNode ? operators.find((operator) => operator.id === workflowNode.operator) : undefined;
    const traceNode = nodeTrace.get(node.id);
    return {
      ...node,
      selected: node.id === selectedNodeId,
      data: {
        ...node.data,
        schema,
        workflowNode,
        traceNode,
        issues: issueMap.get(node.id) || [],
        onDeleteNode: (nodeId: string) => deleteNodesById([nodeId]),
      },
    };
  });
  const graphEdges = workflow.edges.map((workflowEdge) => {
    const edge = flowEdgeFromWorkflow(workflowEdge);
    return {
      ...edge,
      selected: edge.id === selectedEdgeId,
      className: issueMap.get(edge.id)?.some((issue) => issue.level === "error") ? "invalid-edge" : "",
    };
  });

  const selectedEdge = workflow.edges.find((edge) => edgeKey(edge) === selectedEdgeId) || null;

  function showCanvasNotice(message: string, delay = 1800) {
    setCanvasNotice(message);
    window.setTimeout(() => setCanvasNotice(""), delay);
  }

  function organizeCanvas() {
    if (!workflow.nodes.length) {
      showCanvasNotice("画布没有节点");
      return;
    }
    const positions = layoutWorkflowNodes(workflow);
    onNodesChange((current) => current.map((node) => ({
      ...node,
      position: positions.get(node.id) || node.position,
    })));
    window.setTimeout(() => {
      reactFlow?.fitView({ padding: 0.18, duration: 300 });
    }, 0);
    showCanvasNotice("画布已自动整理");
  }

  function deleteNodesById(nodeIds: string[]) {
    if (!nodeIds.length) return;
    onNodesChange((current) => current.filter((node) => !nodeIds.includes(node.id)));
    onEdgesChange((current) => current.filter((edge) => !nodeIds.includes(edge.source) && !nodeIds.includes(edge.target)));
    onWorkflowChange((current) => ({
      ...current,
      nodes: current.nodes.filter((node) => !nodeIds.includes(node.id)),
      edges: current.edges.filter((edge) => !nodeIds.includes(edge.from_node) && !nodeIds.includes(edge.to_node)),
    }));
    if (nodeIds.includes(selectedNodeId)) onSelectNode("");
    if (selectedEdge && (nodeIds.includes(selectedEdge.from_node) || nodeIds.includes(selectedEdge.to_node))) {
      setSelectedEdgeId("");
    }
    showCanvasNotice(`已删除节点：${nodeIds.join(", ")}`);
  }

  function deleteEdgesById(edgeIds: string[]) {
    if (!edgeIds.length) return;
    onEdgesChange((current) => current.filter((edge) => !edgeIds.includes(edge.id)));
    onWorkflowChange((current) => ({
      ...current,
      edges: current.edges.filter((edge) => !edgeIds.includes(edgeKey(edge))),
    }));
    if (edgeIds.includes(selectedEdgeId)) setSelectedEdgeId("");
    showCanvasNotice("已删除连接");
  }

  function addOperatorNode(operatorId: string, position?: { x: number; y: number }) {
    const schema = operators.find((operator) => operator.id === operatorId);
    if (!schema) {
      showCanvasNotice(`未找到节点类型：${operatorId}`, 2400);
      return;
    }
    const id = uniqueNodeId(workflow.nodes, operatorId);
    const flowPosition = position || {
      x: 80 + (workflow.nodes.length % 4) * 260,
      y: 80 + Math.floor(workflow.nodes.length / 4) * 170,
    };
    const params = defaultsFromSchema(schema);
    const nextNode: WorkflowNode = { id, operator: operatorId, params };
    const flowNode = flowNodeFromWorkflow(nextNode, schema, flowPosition);
    onNodesChange((current) => [...current, flowNode]);
    onWorkflowChange((current) => ({ ...current, nodes: [...current.nodes, nextNode] }));
    onSelectNode(id);
    setSelectedEdgeId("");
    showCanvasNotice(`已添加节点：${id}`);
  }

  const onDrop = useCallback((event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    const operatorId = event.dataTransfer.getData("application/vision-operator") || event.dataTransfer.getData("text/plain");
    if (!operatorId) return;
    const rect = event.currentTarget.getBoundingClientRect();
    const position = reactFlow
      ? reactFlow.screenToFlowPosition({ x: event.clientX, y: event.clientY })
      : { x: event.clientX - rect.left - 90, y: event.clientY - rect.top - 28 };
    addOperatorNode(operatorId, { x: position.x - 90, y: position.y - 28 });
  }, [operators, reactFlow, workflow.nodes]);

  const onConnect = useCallback(async (connection: Connection) => {
    if (!connection.source || !connection.target || !connection.sourceHandle || !connection.targetHandle) return;
    const edge = normalizeConnection(connection, workflow, operators);
    if (!edge) {
      showCanvasNotice("没有找到可用的连接方向，请从输出端口连到输入端口", 2400);
      return;
    }
    const issue = validateConnection(edge, workflow, operators);
    if (issue) {
      showCanvasNotice(issue.message, 2400);
      return;
    }
    const exists = workflow.edges.some((item) => edgeKey(item) === edgeKey(edge));
    if (exists) return;
    const nextWorkflow = { ...workflow, edges: [...workflow.edges, edge] };
    try {
      const result = await onValidateWorkflow(nextWorkflow);
      if (!result.ok) {
        showCanvasNotice(firstRustIssueMessage(result), 3200);
        return;
      }
    } catch (err) {
      showCanvasNotice(`Rust validation failed: ${err instanceof Error ? err.message : String(err)}`, 3200);
      return;
    }
    onEdgesChange((current) => [...current, flowEdgeFromWorkflow(edge)]);
    onWorkflowChange(nextWorkflow);
    setSelectedEdgeId(edgeKey(edge));
    showCanvasNotice(`已连接：${edge.from_node}.${edge.from_port} -> ${edge.to_node}.${edge.to_port}`);
  }, [onEdgesChange, onValidateWorkflow, onWorkflowChange, operators, workflow]);

  const onReconnect: OnReconnect = useCallback(async (oldEdge, connection) => {
    if (!connection.source || !connection.target || !connection.sourceHandle || !connection.targetHandle) return;
    const workflowWithoutOldEdge = {
      ...workflow,
      edges: workflow.edges.filter((edge) => edgeKey(edge) !== oldEdge.id),
    };
    const nextEdge = normalizeConnection(connection, workflowWithoutOldEdge, operators);
    if (!nextEdge) {
      showCanvasNotice("没有找到可用的连接方向，请从输出端口连到输入端口", 2400);
      return;
    }
    const issue = validateConnection(nextEdge, workflowWithoutOldEdge, operators);
    if (issue) {
      showCanvasNotice(issue.message, 2400);
      return;
    }
    const nextWorkflow = {
      ...workflow,
      edges: [...workflow.edges.filter((edge) => edgeKey(edge) !== oldEdge.id), nextEdge],
    };
    try {
      const result = await onValidateWorkflow(nextWorkflow);
      if (!result.ok) {
        showCanvasNotice(firstRustIssueMessage(result), 3200);
        return;
      }
    } catch (err) {
      showCanvasNotice(`Rust validation failed: ${err instanceof Error ? err.message : String(err)}`, 3200);
      return;
    }
    onEdgesChange((current) => [
      ...current.filter((edge) => edge.id !== oldEdge.id),
      flowEdgeFromWorkflow(nextEdge),
    ]);
    onWorkflowChange(nextWorkflow);
    setSelectedEdgeId(edgeKey(nextEdge));
    showCanvasNotice(`已修改连接：${nextEdge.from_node}.${nextEdge.from_port} -> ${nextEdge.to_node}.${nextEdge.to_port}`);
  }, [onEdgesChange, onValidateWorkflow, onWorkflowChange, operators, workflow]);

  const handleNodesChange = useCallback((changes: NodeChange[]) => {
    const removedIds = changes
      .filter((change) => change.type === "remove")
      .map((change) => change.id);
    if (removedIds.length) {
      deleteNodesById(removedIds);
      return;
    }
    onNodesChange((current) => applyNodeChanges(changes, current));
  }, [onNodesChange]);

  const handleEdgesChange = useCallback((changes: EdgeChange[]) => {
    const removedIds = changes
      .filter((change) => change.type === "remove")
      .map((change) => change.id);
    if (removedIds.length) {
      deleteEdgesById(removedIds);
      return;
    }
    onEdgesChange((current) => applyEdgeChanges(changes, current));
  }, [onEdgesChange, onWorkflowChange]);

  return (
    <section className="panel canvas-panel runtime-canvas-panel">
      <div className="panel-header canvas-header">
        <div className="canvas-title-block"><h2>Workflow 画布</h2><p>{workflow.id} / {workflow.version}；拖拽节点、连接端口会实时生成 Rust workflow JSON。</p></div>
        <div className="canvas-tools">
          <button className="ghost-button small" onClick={() => addOperatorNode(selectedOperatorId)} disabled={!selectedOperatorId}>添加节点</button>
          <button className="ghost-button small" onClick={organizeCanvas} disabled={!workflow.nodes.length}>自动整理</button>
          <span className="badge soft">{workflow.nodes.length} 节点</span>
          <span className="badge soft">{workflow.edges.length} 连接</span>
          <span className={issues.some((issue) => issue.level === "error") ? "tag ng" : "tag ok"}>{issues.some((issue) => issue.level === "error") ? "校验有误" : "结构正常"}</span>
        </div>
      </div>
      <div
        className="workflow-canvas runtime-canvas react-flow-host"
        onDrop={onDrop}
        onDragOver={(event) => {
          event.preventDefault();
          event.dataTransfer.dropEffect = "copy";
        }}
      >
        {canvasNotice ? <div className="canvas-notice">{canvasNotice}</div> : null}
        {selectedEdge ? (
          <div className="canvas-selection-actions">
            <span>{`${selectedEdge.from_node}.${selectedEdge.from_port} -> ${selectedEdge.to_node}.${selectedEdge.to_port}`}</span>
            <button onClick={() => deleteEdgesById([edgeKey(selectedEdge)])}>删除连接</button>
          </div>
        ) : null}
        <ReactFlow
          nodes={graphNodes}
          edges={graphEdges}
          nodeTypes={nodeTypes}
          connectionMode={ConnectionMode.Loose}
          onNodesChange={handleNodesChange}
          onEdgesChange={handleEdgesChange}
          onConnect={onConnect}
          onReconnect={onReconnect}
          onInit={setReactFlow}
          onNodeClick={(_: React.MouseEvent, node: Node) => {
            setSelectedEdgeId("");
            onSelectNode(node.id);
          }}
          onEdgeClick={(_: React.MouseEvent, edge: Edge) => {
            setSelectedEdgeId(edge.id);
            onSelectNode("");
          }}
          onPaneClick={() => setSelectedEdgeId("")}
          deleteKeyCode={["Backspace", "Delete"]}
          edgesFocusable
          edgesReconnectable
          nodesFocusable
          fitView
        >
          <Background />
          <MiniMap pannable zoomable />
          <Controls />
        </ReactFlow>
      </div>
    </section>
  );
}

function WorkflowFlowNode({ data, selected }: NodeProps) {
  const schema = data.schema as OperatorSchema | undefined;
  const node = data.workflowNode as WorkflowNode | undefined;
  const traceNode = data.traceNode as NodeTrace | undefined;
  const issues = (data.issues as ValidationIssue[] | undefined) || [];
  const onDeleteNode = data.onDeleteNode as ((nodeId: string) => void) | undefined;
  const hasError = issues.some((issue) => issue.level === "error");
  const hasWarning = issues.some((issue) => issue.level === "warning");
  const requiredInputs = schema ? inputPorts(schema) : [];
  const optionalInputs = schema ? optionalInputPorts(schema) : [];
  const outputs = schema ? outputPorts(schema) : [];
  return (
    <div className={`rf-workflow-node ${selected ? "selected" : ""} ${hasError ? "has-error" : hasWarning ? "has-warning" : ""}`}>
      {node && onDeleteNode ? (
        <button
          className="node-delete-button nodrag nopan"
          title="删除节点"
          onClick={(event) => {
            event.stopPropagation();
            onDeleteNode(node.id);
          }}
        >
          ×
        </button>
      ) : null}
      {[...requiredInputs, ...optionalInputs].map((port, index) => (
        <Handle key={`in-${port.name}`} type="target" position={Position.Left} id={port.name} style={{ top: 28 + index * 18 }} />
      ))}
      <div className="rf-node-head"><span>{categoryLabel(rootCategory(schema?.category || "Other"))}</span><strong>{node?.id || "node"}</strong></div>
      <div className="rf-node-operator">{node?.operator || "unknown"}</div>
      <div className="rf-node-ports">
        <div>
          <span>输入</span>
          {(requiredInputs.length ? requiredInputs : [{ name: "无", value_type: "any", required: true }]).map((port) => <em key={port.name}>{portLabel(port)}</em>)}
          {optionalInputs.map((port) => <em className="optional" key={port.name}>{portLabel(port)}</em>)}
        </div>
        <div>
          <span>输出</span>
          {(outputs.length ? outputs : [{ name: "无", value_type: "any", required: true }]).map((port) => <em key={port.name}>{portLabel(port)}</em>)}
        </div>
      </div>
      <div className="rf-node-foot">{traceNode ? `${traceNode.metrics.duration_ms} ms` : `${requiredInputs.length + optionalInputs.length} 入 / ${outputs.length} 出`}</div>
      {issues.length ? <div className="rf-node-issue">{issues[0].message}</div> : null}
      {outputs.map((port, index) => (
        <Handle key={`out-${port.name}`} type="source" position={Position.Right} id={port.name} style={{ top: 28 + index * 18 }} />
      ))}
    </div>
  );
}

function NodeInspector({ workflow, node, schema, runResponse, onClose, onParamChange, onExecutionChange }: {
  workflow: Workflow;
  node?: WorkflowNode;
  schema: OperatorSchema | null;
  runResponse: RunResponse | null;
  onClose: () => void;
  onParamChange: (nodeId: string, key: string, value: JsonValue) => void;
  onExecutionChange: (nodeId: string, patch: NonNullable<WorkflowNode["execution"]>) => void;
}) {
  if (!node) {
    return <section className="panel inspector"><div className="panel-header"><h2>节点实例</h2></div><div className="empty-state">当前 workflow 没有节点。</div></section>;
  }
  const params = (node.params || {}) as Record<string, JsonValue>;
  const incoming = workflow.edges.filter((edge) => edge.to_node === node.id);
  const outgoing = workflow.edges.filter((edge) => edge.from_node === node.id);
  const nodeTrace = runResponse?.trace.nodes.find((item) => item.node_id === node.id);
  const nodeArtifacts = (runResponse?.artifacts || []).filter((file) =>
    file.path.includes(`/${node.id}.`) || file.path.includes(`\\${node.id}.`) || file.name.includes(node.id),
  );
  return (
    <section className="panel inspector">
      <div className="panel-header"><div><h2>节点实例</h2><p>{node.id}</p></div><button className="ghost-button small" onClick={onClose}>收起</button></div>
      <div className="inspector-body">
        <div className="field-group">
          <h3>实例</h3>
          <div className="node-summary"><strong>{node.operator}</strong><span>实例参数只写入 workflow JSON，schema 来自 Rust。</span></div>
        </div>
        <div className="field-group">
          <h3>输入输出</h3>
          <SchemaMini title="输入" items={schema ? inputPorts(schema).map(portLabel) : []} />
          <SchemaMini title="可选输入" items={schema ? optionalInputPorts(schema).map(portLabel) : []} />
          <SchemaMini title="输出" items={schema ? outputPorts(schema).map(portLabel) : []} />
        </div>
        <div className="field-group">
          <h3>连接关系</h3>
          <ConnectionMini title="上游输入" items={incoming.map((edge) => `${edge.from_node}.${edge.from_port} -> ${edge.to_port}`)} />
          <ConnectionMini title="下游输出" items={outgoing.map((edge) => `${edge.from_port} -> ${edge.to_node}.${edge.to_port}`)} />
        </div>
        <div className="field-group">
          <h3>执行策略</h3>
          <div className="field">
            <label>执行条件<span>由 Rust runtime 判定</span></label>
            <select
              value={node.execution?.run_when || "always"}
              onChange={(event) => onExecutionChange(node.id, { run_when: event.target.value as NonNullable<WorkflowNode["execution"]>["run_when"] })}
            >
              <option value="always">始终执行</option>
              <option value="all_ok">输入结果全部 OK</option>
              <option value="any_ok">任一输入结果 OK</option>
              <option value="all_ng">输入结果全部 NG</option>
              <option value="any_ng">任一输入结果 NG</option>
            </select>
          </div>
          <label className="checkbox-row">
            <input type="checkbox" checked={node.execution?.always_run || false} onChange={(event) => onExecutionChange(node.id, { always_run: event.target.checked })} />
            短路后仍执行
          </label>
          <label className="checkbox-row">
            <input type="checkbox" checked={node.execution?.short_circuit_on_ng || false} onChange={(event) => onExecutionChange(node.id, { short_circuit_on_ng: event.target.checked })} />
            本节点 NG 时触发短路
          </label>
        </div>
        <div className="field-group">
          <h3>参数</h3>
          <div className="node-run-card">
            {nodeTrace ? (
              <>
                <strong>最近运行：{nodeTrace.status} · {nodeTrace.metrics.duration_ms} ms</strong>
                <span>输入：{nodeTrace.inputs.map((item) => `${item.port}: ${item.value}`).join(" / ") || "-"}</span>
                <span>输出：{nodeTrace.outputs.map((item) => `${item.port}: ${item.value}`).join(" / ") || "-"}</span>
                {nodeArtifacts.length ? <span>产物：{nodeArtifacts.map((file) => file.name).join(" / ")}</span> : null}
              </>
            ) : (
              <span>最近运行：还没有本节点的运行记录。</span>
            )}
          </div>
          {(schema?.params.length ? schema.params : inferParams(params)).map((param) => (
            <ParamField key={param.key} param={param} value={params[param.key]} onChange={(value) => onParamChange(node.id, param.key, parseParamValue(param, value))} />
          ))}
        </div>
      </div>
    </section>
  );
}

function ParamField({ param, value, onChange }: { param: ParamSchema; value: JsonValue | undefined; onChange: (value: string) => void }) {
  const stringValue = value === undefined || value === null ? param.default || "" : typeof value === "object" ? JSON.stringify(value) : String(value);
  if (param.options?.length) {
    return (
      <div className="field">
        <label>{param.label || param.key}</label>
        <select value={stringValue} onChange={(event) => onChange(event.target.value)}>
          <option value="">未设置</option>
          {param.options.map((option) => <option key={option} value={option}>{option}</option>)}
        </select>
      </div>
    );
  }
  return (
    <div className="field">
      <label>{param.label || param.key}<span>{param.required ? "必填" : "可选"} / {param.kind}</span></label>
      <input value={stringValue} placeholder={param.placeholder || param.description} onChange={(event) => onChange(event.target.value)} />
    </div>
  );
}

function BottomDrawer({
  workflow,
  runResponse,
  issues,
  resultOpen,
  jsonOpen,
  onResultOpen,
  onJsonOpen,
}: {
  workflow: Workflow;
  runResponse: RunResponse | null;
  issues: ValidationIssue[];
  resultOpen: boolean;
  jsonOpen: boolean;
  onResultOpen: (open: boolean) => void;
  onJsonOpen: (open: boolean) => void;
}) {
  const open = resultOpen || jsonOpen;
  return (
    <section className={open ? "panel runtime-bottom-drawer open" : "panel runtime-bottom-drawer"}>
      <div className="panel-header">
        <div><h2>校验 / JSON / 运行</h2><p>默认不占画布空间，需要时展开。</p></div>
        <div className="canvas-tools">
          <button className={resultOpen ? "primary-button small" : "ghost-button small"} onClick={() => onResultOpen(!resultOpen)}>校验结果</button>
          <button className={jsonOpen ? "primary-button small" : "ghost-button small"} onClick={() => onJsonOpen(!jsonOpen)}>JSON</button>
        </div>
      </div>
      {open ? (
        <div className="bottom-drawer-body">
          {resultOpen ? (
            <section className="schema-panel">
              <h3>校验与运行摘要</h3>
              {issues.length ? issues.map((issue, index) => <div className={issue.level === "error" ? "issue-row error" : "issue-row warning"} key={`${issue.message}-${index}`}>{issue.message}</div>) : <div className="issue-row ok">UI 即时校验未发现错误，运行前仍会调用 Rust 最终校验。</div>}
              {runResponse ? <div className="issue-row ok">最近运行：{runResponse.event.run_id} / {formatRunResult(runResponse.event.result)} / {runResponse.event.metrics.total_duration_ms} ms / FPS {runResponse.event.metrics.fps}</div> : null}
            </section>
          ) : null}
          {resultOpen ? <ArtifactPreviewPanel runResponse={runResponse} /> : null}
          {jsonOpen ? (
            <section className="schema-panel json-panel">
              <h3>Rust Workflow JSON</h3>
              <pre className="workflow-json-preview">{JSON.stringify(workflow, null, 2)}</pre>
            </section>
          ) : null}
        </div>
      ) : null}
    </section>
  );
}

function ArtifactPreviewPanel({ runResponse }: { runResponse: RunResponse | null }) {
  const artifacts = runResponse?.artifacts || [];
  const [selectedPath, setSelectedPath] = useState("");
  const [content, setContent] = useState<ArtifactContent | null>(null);
  const [error, setError] = useState("");
  const selected = artifacts.find((file) => file.path === selectedPath) || artifacts[0];

  useEffect(() => {
    if (!selected) {
      setSelectedPath("");
      setContent(null);
      setError("");
      return;
    }
    if (!selectedPath || !artifacts.some((file) => file.path === selectedPath)) {
      setSelectedPath(selected.path);
    }
  }, [runResponse?.event.run_id, artifacts.length]);

  useEffect(() => {
    if (!selected) return;
    let cancelled = false;
    setError("");
    invoke<ArtifactContent>("read_artifact", { path: selected.path })
      .then((value) => {
        if (!cancelled) setContent(value);
      })
      .catch((err) => {
        if (!cancelled) {
          setContent(null);
          setError(err instanceof Error ? err.message : String(err));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [selected?.path]);

  return (
    <section className="schema-panel artifact-preview-panel">
      <h3>运行产物预览</h3>
      {!runResponse ? <div className="empty-state compact">运行后会在这里显示图片、JSON 和 display 节点产物。</div> : null}
      {runResponse ? (
        <div className="artifact-preview-layout">
          <div className="artifact-list">
            {artifacts.length ? artifacts.map((file) => (
              <button
                key={file.path}
                className={selected?.path === file.path ? "artifact-row active" : "artifact-row"}
                onClick={() => setSelectedPath(file.path)}
              >
                <strong>{file.name}</strong>
                <span>{file.path}</span>
              </button>
            )) : <div className="empty-state compact">本次运行没有额外产物。</div>}
          </div>
          <div className="artifact-preview">
            {error ? <div className="issue-row error">{error}</div> : null}
            {!error && content?.kind === "image" && content.data_url ? (
              <img src={content.data_url} alt={selected?.name || "artifact"} />
            ) : null}
            {!error && content?.kind !== "image" ? (
              <pre>{content?.text || ""}</pre>
            ) : null}
          </div>
        </div>
      ) : null}
    </section>
  );
}

function NodeManager({
  operators,
  selectedOperatorId,
  nodeUiConfig,
  nodeSmokeStatus,
  onSelect,
  onUiConfigChange,
  onDefaultParamChange,
}: {
  operators: OperatorSchema[];
  selectedOperatorId: string;
  nodeUiConfig: Record<string, NodeUiConfig>;
  nodeSmokeStatus: NodeSmokeStatus | null;
  onSelect: (id: string) => void;
  onUiConfigChange: (operatorId: string, patch: Partial<NodeUiConfig>) => void;
  onDefaultParamChange: (operatorId: string, key: string, value: JsonValue) => void;
}) {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("all");
  const categories = Array.from(new Set(operators.map((operator) => rootCategory(operator.category)))).sort();
  const filtered = operators.filter((operator) => {
    const matchesCategory = category === "all" || rootCategory(operator.category) === category;
    const needle = query.trim().toLowerCase();
    const matchesQuery = !needle || `${operator.id} ${operator.name} ${operator.category}`.toLowerCase().includes(needle);
    return matchesCategory && matchesQuery;
  });
  const groups = groupBy(filtered, (operator) => rootCategory(operator.category));
  const selected = operators.find((operator) => operator.id === selectedOperatorId) || operators[0];
  const smokeById = smokeStatusMap(nodeSmokeStatus);
  return (
    <div className="page-panel">
      <div className="page-header"><div><h1>节点管理</h1><p>这里展示节点类型定义，来源是 Rust registry 导出的 OperatorSchema。</p></div><span className="badge">{operators.length} 个节点</span><SmokeSummaryBadge status={nodeSmokeStatus} /></div>
      <div className="manager-filter-bar">
        <input className="search-input" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="搜索节点类型" />
        <select value={category} onChange={(event) => setCategory(event.target.value)}>
          <option value="all">全部分类</option>
          {categories.map((item) => <option key={item} value={item}>{categoryLabel(item)}</option>)}
        </select>
      </div>
      <div className="node-manager-layout">
        <section className="node-type-list">
          {Object.entries(groups).map(([category, items]) => (
            <div className="node-manager-group" key={category}>
              <div className="node-group-title"><strong>{categoryLabel(category)}</strong><span>{items.length}</span></div>
              {items.map((operator) => (
                <button key={operator.id} className={operator.id === selected?.id ? "node-type-row active" : "node-type-row"} onClick={() => onSelect(operator.id)}>
                  <span>{operator.category}<SmokeBadge status={smokeById.get(operator.id)?.status} /></span><strong>{operator.name}</strong><em>{operator.id}</em>
                </button>
              ))}
            </div>
          ))}
        </section>
        {selected && (
          <OperatorDetail
            operator={selected}
            config={nodeUiConfig[selected.id] || { defaultParams: {}, hiddenAdvanced: true }}
            smoke={smokeById.get(selected.id)}
            smokeSource={nodeSmokeStatus?.source || ""}
            onConfigChange={(patch) => onUiConfigChange(selected.id, patch)}
            onDefaultParamChange={(key, value) => onDefaultParamChange(selected.id, key, value)}
          />
        )}
      </div>
    </div>
  );
}

function OperatorDetail({ operator, config, smoke, smokeSource, onConfigChange, onDefaultParamChange }: {
  operator: OperatorSchema;
  config: NodeUiConfig;
  smoke?: NodeSmokeOperatorStatus;
  smokeSource: string;
  onConfigChange: (patch: Partial<NodeUiConfig>) => void;
  onDefaultParamChange: (key: string, value: JsonValue) => void;
}) {
  return (
    <section className="node-type-detail">
      <div className="detail-hero"><span className="badge">{categoryLabel(rootCategory(operator.category))}</span><SmokeBadge status={smoke?.status} /><h2>{operator.name}</h2><p>{operator.id}</p></div>
      <div className="schema-grid">
        <section className="schema-panel">
          <h3>参数 Schema 与 UI 默认值</h3>
          {operator.params.length ? operator.params.map((param) => (
            <ParamSchemaRow
              key={param.key}
              param={param}
              defaultValue={config.defaultParams[param.key]}
              onDefaultChange={(value) => onDefaultParamChange(param.key, parseParamValue(param, value))}
            />
          )) : <div className="empty-state">无显式参数</div>}
        </section>
        <section className="schema-panel"><h3>输入输出</h3><SchemaMini title="输入" items={inputPorts(operator).map(portLabel)} /><SchemaMini title="可选输入" items={optionalInputPorts(operator).map(portLabel)} /><SchemaMini title="输出" items={outputPorts(operator).map(portLabel)} /><SchemaMini title="Rust 分类" items={[operator.category]} /></section>
        <section className="schema-panel"><h3>Smoke Test</h3><SmokeDetail smoke={smoke} source={smokeSource} /></section>
        <section className="schema-panel">
          <h3>UI 层配置</h3>
          <div className="field"><label>显示名称覆盖</label><input value={config.displayName || ""} onChange={(event) => onConfigChange({ displayName: event.target.value })} placeholder={operator.name} /></div>
          <label className="checkbox-row"><input type="checkbox" checked={config.hiddenAdvanced} onChange={(event) => onConfigChange({ hiddenAdvanced: event.target.checked })} /> 默认折叠高级参数</label>
          <p className="muted-note">这里编辑的是 UI 默认配置，不会修改 Rust 里的 operator id、端口或运行函数。</p>
        </section>
      </div>
    </section>
  );
}

function ParamSchemaRow({ param, defaultValue, onDefaultChange }: { param: ParamSchema; defaultValue?: JsonValue; onDefaultChange?: (value: string) => void }) {
  return (
    <div className="schema-row compact-schema-row">
      <div><strong>{param.label || param.key}</strong><span>{param.key}</span></div>
      <div><span>类型</span><strong>{param.kind}</strong></div>
      <div><span>默认</span><strong>{param.default || "-"}</strong></div>
      <div><span>约束</span><strong>{param.options?.join(" / ") || param.description || "-"}</strong></div>
      {onDefaultChange ? <p><input className="inline-input" value={defaultValue === undefined ? "" : String(defaultValue)} placeholder="UI 默认值覆盖" onChange={(event) => onDefaultChange(event.target.value)} /></p> : null}
    </div>
  );
}

function ModelsPage({ models, catalog, operators }: { models: FileEntry[]; catalog: ModelCatalog | null; operators: OperatorSchema[] }) {
  const inferOps = operators.filter((operator) => ["Infer", "Ai"].includes(rootCategory(operator.category)) || operator.id.includes("yolo") || operator.id.includes("onnx"));
  return (
    <div className="page-panel">
      <div className="page-header"><div><h1>模型</h1><p>模型文件是资源，workflow 节点通过 model_path、backend、labels 等参数引用。</p></div><span className="badge">{models.length} 个 ONNX</span></div>
      <div className="schema-grid">
        <section className="schema-panel"><h3>模型目录状态</h3>{Object.entries(catalog?.models || {}).map(([id, model]) => <div className="usage-item" key={id}><strong>{id} <span className={`tag ${model.status === "ready" ? "ok" : "ng"}`}>{model.status || "unknown"}</span></strong><span>{model.task || "-"} · {model.format || "-"} · {model.input_width || "-"}x{model.input_height || "-"} · {model.recommended_backend || "-"}</span><small>{model.path || ""} · SHA256 {model.sha256?.slice(0, 12) || "-"}</small></div>)}{!Object.keys(catalog?.models || {}).length ? models.map((model) => <div className="usage-item" key={model.path}><strong>{model.name}</strong><span>{model.path}</span></div>) : null}</section>
        <section className="schema-panel"><h3>可引用推理节点</h3>{inferOps.map((operator) => <div className="usage-item" key={operator.id}><strong>{operator.name}</strong><span>{operator.id}</span></div>)}</section>
      </div>
    </div>
  );
}

function RecordsPage({ records, runResponse }: { records: RecordEntry[]; runResponse: RunResponse | null }) {
  return (
    <div className="page-panel">
      <div className="page-header"><div><h1>运行记录</h1><p>读取 records/*/event.json，当前运行的 trace 会在右侧详情里展示。</p></div><span className="badge">{records.length} 条</span></div>
      <div className="schema-grid">
        <section className="schema-panel">
          <h3>Event 列表</h3>
          <div className="table-wrap"><table className="data-table"><thead><tr><th>Run ID</th><th>Workflow</th><th>结果</th><th>耗时</th><th>FPS</th></tr></thead><tbody>
            {records.map((record) => <tr key={record.run_id}><td>{record.run_id}</td><td>{record.workflow_id}</td><td><span className={resultTagClass(record.result)}>{formatRunResult(record.result)}</span></td><td>{record.total_duration_ms} ms</td><td>{record.fps}</td></tr>)}
          </tbody></table></div>
        </section>
        <section className="schema-panel">
          <h3>当前 Trace</h3>
          {(runResponse?.trace.nodes || []).map((node) => <div className="usage-item" key={node.node_id}><strong>{node.node_id}</strong><span>{node.outputs.map((item) => `${item.port}: ${item.value}`).join(" / ") || "无输出"}</span></div>)}
        </section>
      </div>
    </div>
  );
}

function DataPage({ examples, records, runResponse }: { examples: FileEntry[]; records: RecordEntry[]; runResponse: RunResponse | null }) {
  return (
    <div className="page-panel">
      <div className="page-header"><div><h1>数据</h1><p>先围绕 workflow 输入、运行输出、artifact 做资源视图，后续再接数据集和标注。</p></div></div>
      <div className="status-grid">
        <div className="status-card"><span>示例 Workflow</span><strong>{examples.length}</strong></div>
        <div className="status-card"><span>运行记录</span><strong>{records.length}</strong></div>
        <div className="status-card"><span>当前产物</span><strong>{runResponse?.artifacts.length || 0}</strong></div>
      </div>
      <div className="sample-grid">
        {(runResponse?.artifacts || []).map((file) => <div className="sample-card" key={file.path}><div className="sample-thumb"><span>{file.name.slice(0, 2)}</span></div><strong>{file.name}</strong><p>{file.path}</p></div>)}
      </div>
    </div>
  );
}

function DevicesPage({ operators }: { operators: OperatorSchema[] }) {
  const deviceOps = operators.filter((operator) => ["Source", "Trigger", "Action"].includes(rootCategory(operator.category)) || operator.id.includes("camera") || operator.id.includes("trigger"));
  return (
    <div className="page-panel">
      <div className="page-header"><div><h1>设备</h1><p>设备页先映射到输入源、触发、IO 动作节点；真实 SDK 后续作为节点实现接入。</p></div></div>
      <div className="device-layout">
        {deviceOps.map((operator) => <section className="device-card" key={operator.id}><div className="device-head"><span className="device-type">{operator.category}</span><strong>{operator.name}</strong></div><dl><div><dt>节点</dt><dd>{operator.id}</dd></div><div><dt>接口</dt><dd>{operator.outputs.join(", ") || "action"}</dd></div></dl></section>)}
      </div>
    </div>
  );
}

function EnvironmentPage({ status, operators, models, records }: { status: RuntimeStatus | null; operators: OperatorSchema[]; models: FileEntry[]; records: RecordEntry[] }) {
  return (
    <div className="page-panel">
      <div className="page-header"><h1>环境</h1><span className="badge">v{status?.version || "0.1.0"}</span></div>
      <div className="status-grid">
        <div className="status-card"><span>Runtime</span><strong>{status?.runtime || "-"}</strong></div>
        <div className="status-card"><span>项目根目录</span><strong>{status?.project_root || "-"}</strong></div>
        <div className="status-card"><span>节点 Schema</span><strong>{operators.length}</strong></div>
        <div className="status-card"><span>模型文件</span><strong>{models.length}</strong></div>
        <div className="status-card"><span>运行记录</span><strong>{records.length}</strong></div>
      </div>
    </div>
  );
}

function SettingsPage({ status }: { status: RuntimeStatus | null }) {
  return (
    <div className="page-panel settings-page">
      <div className="page-header"><h1>设置</h1><button className="primary-button">保存设置</button></div>
      <div className="settings-grid">
        <label>项目根目录<input value={status?.project_root || ""} readOnly /></label>
        <label>Workflow 保存目录<input value="studio_workflows" readOnly /></label>
        <label>运行记录目录<input value="records" readOnly /></label>
        <label>模型目录<input value="models" readOnly /></label>
      </div>
    </div>
  );
}

function AboutPage({ status }: { status: RuntimeStatus | null }) {
  return (
    <div className="page-panel about-page">
      <div className="page-header"><div><h1>关于</h1><p>Rust-first 开源工业视觉检测 runtime 的可视化开发台。</p></div><span className="badge">v{status?.version || "0.1.0"}-alpha</span></div>
      <div className="about-grid">
        <div><span>界面</span><strong>React + Tauri</strong></div>
        <div><span>核心</span><strong>vision-core Executor</strong></div>
        <div><span>节点来源</span><strong>Rust OperatorSchema</strong></div>
        <div><span>运行记录</span><strong>RunEvent / RunTrace</strong></div>
      </div>
    </div>
  );
}

function SchemaMini({ title, items }: { title: string; items: string[] }) {
  return <div className="schema-mini"><span>{title}</span>{(items.length ? items : ["无"]).map((item) => <strong key={item}>{item}</strong>)}</div>;
}

function ConnectionMini({ title, items }: { title: string; items: string[] }) {
  return (
    <div className="schema-mini connection-mini">
      <span>{title}</span>
      {(items.length ? items : ["未连接"]).map((item) => <strong key={item}>{item}</strong>)}
    </div>
  );
}

function normalizeWorkflow(workflow: Workflow): Workflow {
  return {
    ...workflow,
    nodes: workflow.nodes.map((node) => ({ ...node, params: node.params || {} })),
    edges: workflow.edges || [],
  };
}

function workflowToFlowNodes(workflow: Workflow, operators: OperatorSchema[], metadata?: WorkflowUiMetadata): Node[] {
  const positions = layoutWorkflowNodes(workflow);
  return workflow.nodes.map((node) => flowNodeFromWorkflow(
    node,
    operators.find((operator) => operator.id === node.operator),
    metadata?.node_positions?.[node.id] || positions.get(node.id) || { x: 40, y: 40 },
  ));
}

function workflowToFlowEdges(workflow: Workflow): Edge[] {
  return workflow.edges.map((edge) => flowEdgeFromWorkflow(edge));
}

function flowNodeFromWorkflow(node: WorkflowNode, schema: OperatorSchema | undefined, position: { x: number; y: number }): Node {
  return {
    id: node.id,
    type: "workflowNode",
    position,
    data: { workflowNode: node, schema },
  };
}

function workflowUiMetadata(workflow: Workflow, nodes: Node[]): WorkflowUiMetadata {
  const node_positions = nodes.reduce<Record<string, { x: number; y: number }>>((acc, node) => {
    if (workflow.nodes.some((workflowNode) => workflowNode.id === node.id)) {
      acc[node.id] = { x: node.position.x, y: node.position.y };
    }
    return acc;
  }, {});
  return {
    workflow_id: workflow.id,
    node_positions,
  };
}

function flowEdgeFromWorkflow(edge: WorkflowEdge, invalid = false): Edge {
  return {
    id: edgeKey(edge),
    source: edge.from_node,
    sourceHandle: edge.from_port,
    target: edge.to_node,
    targetHandle: edge.to_port,
    label: `${edge.from_port} -> ${edge.to_port}`,
    markerEnd: { type: MarkerType.ArrowClosed },
    animated: invalid,
    className: invalid ? "invalid-edge" : "",
  };
}

function flowEdgeToWorkflow(edge: Edge): WorkflowEdge | null {
  if (!edge.source || !edge.target || !edge.sourceHandle || !edge.targetHandle) return null;
  return {
    from_node: edge.source,
    from_port: String(edge.sourceHandle),
    to_node: edge.target,
    to_port: String(edge.targetHandle),
  };
}

function workflowEdgeFromConnection(connection: Connection): WorkflowEdge {
  return {
    from_node: String(connection.source),
    from_port: String(connection.sourceHandle),
    to_node: String(connection.target),
    to_port: String(connection.targetHandle),
  };
}

function reverseWorkflowEdgeFromConnection(connection: Connection): WorkflowEdge {
  return {
    from_node: String(connection.target),
    from_port: String(connection.targetHandle),
    to_node: String(connection.source),
    to_port: String(connection.sourceHandle),
  };
}

function normalizeConnection(connection: Connection, workflow: Workflow, operators: OperatorSchema[]): WorkflowEdge | null {
  const forward = workflowEdgeFromConnection(connection);
  if (!validateConnection(forward, workflow, operators)) return forward;
  const reversed = reverseWorkflowEdgeFromConnection(connection);
  if (!validateConnection(reversed, workflow, operators)) return reversed;
  return forward;
}

function edgeKey(edge: WorkflowEdge) {
  return `${edge.from_node}:${edge.from_port}->${edge.to_node}:${edge.to_port}`;
}

function uniqueNodeId(nodes: WorkflowNode[], operatorId: string) {
  const base = operatorId.split(".").slice(-2).join("_").replace(/[^a-zA-Z0-9_]/g, "_") || "node";
  let index = 1;
  let id = `${base}_${index}`;
  const existing = new Set(nodes.map((node) => node.id));
  while (existing.has(id)) {
    index += 1;
    id = `${base}_${index}`;
  }
  return id;
}

function sanitizeWorkflowId(raw: string) {
  return raw
    .trim()
    .replace(/\.json$/i, "")
    .replace(/[^a-zA-Z0-9_-]/g, "_")
    .replace(/_+/g, "_")
    .replace(/^_+|_+$/g, "");
}

function defaultsFromSchema(schema: OperatorSchema): Record<string, JsonValue> {
  const params: Record<string, JsonValue> = {};
  for (const param of schema.params) {
    if (param.default !== undefined && param.default !== null) {
      params[param.key] = parseParamValue(param, param.default);
    }
  }
  return params;
}

function rustIssuesToUi(issues: RustValidationIssue[]): ValidationIssue[] {
  return issues.map((issue) => ({
    level: issue.level,
    scope: issue.edge_id ? "edge" : issue.node_id ? "node" : issue.port ? "param" : "workflow",
    nodeId: issue.node_id || undefined,
    edgeId: issue.edge_id || undefined,
    message: `${issue.code}: ${issue.message}`,
  }));
}

function firstRustIssueMessage(result: ValidateResult): string {
  const issue = result.issues?.[0];
  return issue ? `${issue.code}: ${issue.message}` : result.message;
}

function inputPorts(schema: OperatorSchema): PortSchema[] {
  return schema.input_ports || schema.inputs.map((name) => ({ name, value_type: inferPortTypeName(name), required: true }));
}

function optionalInputPorts(schema: OperatorSchema): PortSchema[] {
  return schema.optional_input_ports || schema.optional_inputs.map((name) => ({ name, value_type: inferPortTypeName(name), required: false }));
}

function allInputPorts(schema: OperatorSchema): PortSchema[] {
  return [...inputPorts(schema), ...optionalInputPorts(schema)];
}

function outputPorts(schema: OperatorSchema): PortSchema[] {
  return schema.output_ports || schema.outputs.map((name) => ({ name, value_type: inferPortTypeName(name), required: true }));
}

function findPort(ports: PortSchema[], name: string): PortSchema | undefined {
  return ports.find((port) => port.name === name);
}

function portTypesCompatible(outputType: string, inputType: string): boolean {
  const output = outputType.toLowerCase();
  const input = inputType.toLowerCase();
  return output === input || output === "any" || input === "any";
}

function inferPortTypeName(name: string): string {
  if (name.startsWith("rule")) return "rule";
  if (["image", "gray", "frame", "annotated"].includes(name)) return "image";
  if (name === "mask") return "mask";
  if (["regions", "blobs"].includes(name)) return "regions";
  if (name === "contours") return "contours";
  if (["measurements", "measurement", "metrics"].includes(name)) return "measurements";
  if (["location", "pose"].includes(name)) return "location";
  if (name === "detections") return "detections";
  if (["classes", "classifications"].includes(name)) return "classifications";
  if (["masks", "segmentations"].includes(name)) return "segmentations";
  if (["poses", "keypoints"].includes(name)) return "poses";
  if (["oriented_boxes", "obb"].includes(name)) return "oriented_boxes";
  if (name === "rules") return "rule";
  if (name === "scale") return "scale";
  if (name === "camera_calibration") return "camera_calibration";
  if (name === "calibration") return "any";
  if (name === "final") return "final";
  if (name === "trigger") return "trigger";
  if (name === "capture") return "capture";
  if (name === "signal") return "signal";
  if (["ok", "passed"].includes(name)) return "bool";
  if (["text", "message", "status"].includes(name)) return "text";
  if (["raw", "stats", "payload", "files", "record", "trace", "result", "failed_count", "failed_regions"].includes(name)) return "json";
  if (name === "data") return "any";
  return "any";
}

function portLabel(port: PortSchema): string {
  return `${port.name}: ${port.value_type}${port.required ? "" : "?"}`;
}

function formatRunResult(result: string): string {
  if (result === "UNKNOWN") return "完成/未判定";
  return result || "完成/未判定";
}

function resultTagClass(result: string): string {
  if (result === "OK") return "tag ok";
  if (result === "NG") return "tag ng";
  return "tag soft";
}

function smokeStatusMap(status: NodeSmokeStatus | null): Map<string, NodeSmokeOperatorStatus> {
  return new Map((status?.operators || []).map((operator) => [operator.id, operator]));
}

function SmokeBadge({ status }: { status?: string }) {
  const normalized = status?.toUpperCase();
  const className = normalized === "PASS" ? "pass" : normalized === "FAIL" ? "fail" : "unknown";
  const label = normalized === "PASS" ? "已通过" : normalized === "FAIL" ? "失败" : "未测试";
  return <span className={`smoke-badge ${className}`}>{label}</span>;
}

function SmokeSummaryBadge({ status }: { status: NodeSmokeStatus | null }) {
  if (!status?.source) {
    return <span className="smoke-summary-badge unknown">Smoke 未测试</span>;
  }
  const className = status.fail > 0 ? "fail" : "pass";
  return (
    <span className={`smoke-summary-badge ${className}`} title={status.source}>
      Smoke {status.pass}/{status.total}{status.fail ? `，失败 ${status.fail}` : ""}
    </span>
  );
}

function SmokeDetail({ smoke, source }: { smoke?: NodeSmokeOperatorStatus; source: string }) {
  if (!smoke) {
    return <div className="smoke-detail"><SmokeBadge /><p>当前节点没有找到 smoke test 记录。</p></div>;
  }
  return (
    <div className="smoke-detail">
      <div><SmokeBadge status={smoke.status} /><strong>{smoke.ms} ms</strong></div>
      <p>{smoke.message || "无错误信息"}</p>
      {source ? <span>{source}</span> : null}
    </div>
  );
}

function clientBlocksTargetPortOccupancy(): boolean {
  return true;
}

function clientBlocksPortTypeCompatibility(): boolean {
  return true;
}

function validateConnection(edge: WorkflowEdge, workflow: Workflow, operators: OperatorSchema[]): ValidationIssue | null {
  const from = workflow.nodes.find((node) => node.id === edge.from_node);
  const to = workflow.nodes.find((node) => node.id === edge.to_node) as WorkflowNode;
  if (!from) return { level: "error", scope: "edge", edgeId: edgeKey(edge), message: `连接来源节点不存在：${edge.from_node}` };
  if (!to) return { level: "error", scope: "edge", edgeId: edgeKey(edge), message: `连接目标节点不存在：${edge.to_node}` };
  const fromSchema = operators.find((operator) => operator.id === from.operator);
  const toSchema = operators.find((operator) => operator.id === to.operator);
  if (!fromSchema) return { level: "error", scope: "node", nodeId: from.id, message: `未知 operator：${from.operator}` };
  if (!toSchema) return { level: "error", scope: "node", nodeId: to.id, message: `未知 operator：${to.operator}` };
  const outputPort = findPort(outputPorts(fromSchema), edge.from_port);
  const inputPort = findPort(allInputPorts(toSchema), edge.to_port);
  if (!outputPort) {
    return { level: "error", scope: "edge", edgeId: edgeKey(edge), message: `${from.id} 没有输出端口 ${edge.from_port}` };
  }
  if (!inputPort) {
    return { level: "error", scope: "edge", edgeId: edgeKey(edge), message: `${to.id} 没有输入端口 ${edge.to_port}` };
  }
  const occupied = workflow.edges.some(
    (item) => item.to_node === edge.to_node && item.to_port === edge.to_port && edgeKey(item) !== edgeKey(edge),
  );
  if (clientBlocksTargetPortOccupancy() && occupied && to) {
    return { level: "error", scope: "edge", edgeId: edgeKey(edge), message: `${to.id}.${edge.to_port} 已有上游连接，单输入端口不能重复连接` };
  }
  if (clientBlocksPortTypeCompatibility() && !portTypesCompatible(outputPort.value_type, inputPort.value_type)) {
    return {
      level: "error",
      scope: "edge",
      edgeId: edgeKey(edge),
      message: `${from.id}.${edge.from_port} (${outputPort.value_type}) 不能连接到 ${to.id}.${edge.to_port} (${inputPort.value_type})`,
    };
  }
  return null;
}

function validateWorkflowLocal(workflow: Workflow, operators: OperatorSchema[]): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const nodeIds = new Set<string>();
  for (const node of workflow.nodes) {
    if (nodeIds.has(node.id)) {
      issues.push({ level: "error", scope: "node", nodeId: node.id, message: `节点 id 重复：${node.id}` });
    }
    nodeIds.add(node.id);
    const schema = operators.find((operator) => operator.id === node.operator);
    if (!schema) {
      issues.push({ level: "error", scope: "node", nodeId: node.id, message: `未知 operator：${node.operator}` });
      continue;
    }
    const params = (node.params || {}) as Record<string, JsonValue>;
    for (const param of schema.params.filter((item) => item.required)) {
      if (params[param.key] === undefined || params[param.key] === null || params[param.key] === "") {
        issues.push({ level: "error", scope: "param", nodeId: node.id, message: `${node.id} 缺少必填参数：${param.key}` });
      }
    }
  }

  const edgeIds = new Set<string>();
  for (const edge of workflow.edges) {
    const id = edgeKey(edge);
    if (edgeIds.has(id)) {
      issues.push({ level: "error", scope: "edge", edgeId: id, message: `连接重复：${id}` });
      continue;
    }
    edgeIds.add(id);
    const issue = validateConnection(edge, workflow, operators);
    if (issue) issues.push(issue);
  }

  for (const node of workflow.nodes) {
    const schema = operators.find((operator) => operator.id === node.operator);
    if (!schema) continue;
    const connected = new Set(workflow.edges.filter((edge) => edge.to_node === node.id).map((edge) => edge.to_port));
    for (const input of schema.inputs) {
      if (!connected.has(input)) {
        issues.push({ level: "error", scope: "node", nodeId: node.id, message: `${node.id} 缺少必填输入：${input}` });
      }
    }
  }

  return issues;
}

function rootCategory(category: string) {
  return category.split("/")[0] || "Other";
}

function categoryLabel(category: string) {
  return categoryMap[category] || category;
}

function groupBy<T>(items: T[], key: (item: T) => string) {
  return items.reduce<Record<string, T[]>>((acc, item) => {
    const group = key(item);
    acc[group] ||= [];
    acc[group].push(item);
    return acc;
  }, {});
}

function layoutNodes(nodes: WorkflowNode[]) {
  const map = new Map<string, { x: number; y: number }>();
  nodes.forEach((node, index) => {
    const column = index % 4;
    const row = Math.floor(index / 4);
    map.set(node.id, { x: 36 + column * 220, y: 64 + row * 124 });
  });
  return map;
}

function layoutWorkflowNodes(workflow: Workflow) {
  if (!workflow.edges.length) return layoutNodes(workflow.nodes);

  const nodeIds = workflow.nodes.map((node) => node.id);
  const nodeIdSet = new Set(nodeIds);
  const incoming = new Map<string, Set<string>>();
  const outgoing = new Map<string, Set<string>>();
  nodeIds.forEach((id) => {
    incoming.set(id, new Set());
    outgoing.set(id, new Set());
  });

  for (const edge of workflow.edges) {
    if (!nodeIdSet.has(edge.from_node) || !nodeIdSet.has(edge.to_node)) continue;
    outgoing.get(edge.from_node)?.add(edge.to_node);
    incoming.get(edge.to_node)?.add(edge.from_node);
  }

  const remainingIncoming = new Map<string, number>();
  nodeIds.forEach((id) => remainingIncoming.set(id, incoming.get(id)?.size || 0));

  const layers = new Map<string, number>();
  const queue = nodeIds.filter((id) => (remainingIncoming.get(id) || 0) === 0);
  queue.forEach((id) => layers.set(id, 0));

  for (let index = 0; index < queue.length; index += 1) {
    const id = queue[index];
    const nextLayer = (layers.get(id) || 0) + 1;
    for (const nextId of outgoing.get(id) || []) {
      layers.set(nextId, Math.max(layers.get(nextId) || 0, nextLayer));
      const nextIncoming = Math.max(0, (remainingIncoming.get(nextId) || 0) - 1);
      remainingIncoming.set(nextId, nextIncoming);
      if (nextIncoming === 0) queue.push(nextId);
    }
  }

  const fallbackLayer = Math.max(0, ...Array.from(layers.values())) + 1;
  nodeIds.forEach((id) => {
    if (!layers.has(id)) layers.set(id, fallbackLayer);
  });

  const layerGroups = groupBy(nodeIds, (id) => String(layers.get(id) || 0));
  const map = new Map<string, { x: number; y: number }>();
  Object.keys(layerGroups)
    .map(Number)
    .sort((a, b) => a - b)
    .forEach((layer) => {
      const ids = layerGroups[String(layer)];
      ids.forEach((id, row) => {
        map.set(id, { x: 60 + layer * 280, y: 70 + row * 150 });
      });
    });
  return map;
}

function inferParams(params: Record<string, JsonValue>): ParamSchema[] {
  return Object.keys(params).map((key, index) => ({
    key,
    label: key,
    kind: typeof params[key] === "number" ? "Number" : typeof params[key] === "boolean" ? "Bool" : "String",
    required: false,
    default: null,
    description: "",
    options: [],
    group: "实例参数",
    order: index,
    advanced: false,
    unit: null,
    placeholder: null,
    min: null,
    max: null,
    step: null,
    accept: [],
    deployment_required: false,
    overridable: true,
  }));
}

function parseParamValue(param: ParamSchema, raw: string): JsonValue {
  const kind = param.kind.toLowerCase();
  if (kind.includes("bool")) return raw === "true" || raw === "1";
  if (kind.includes("number") || kind.includes("integer") || kind.includes("float")) {
    const num = Number(raw);
    return Number.isFinite(num) ? num : raw;
  }
  if (kind.includes("json")) {
    try {
      return JSON.parse(raw) as JsonValue;
    } catch {
      return raw;
    }
  }
  return raw;
}

createRoot(document.getElementById("root")!).render(<App />);
