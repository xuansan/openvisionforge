# vision-cli/src

这里目前只有 `main.rs`。

## main.rs 做什么

```text
run()
  创建 OperatorRegistry
  注册 vision-operators 的 mock 节点
  根据命令执行 list-operators 或 run
```

## run 命令的数据流

```text
workflow path
  -> fs::read_to_string
  -> serde_json::from_str::<Workflow>
  -> Executor::run(workflow, "records")
  -> 读取 event.json
  -> 打印 Result / Trace / Event
```

## 后续什么时候拆文件

现在不拆。

等出现下面情况再拆：

```text
命令超过 3 个
参数开始有 option / flag
需要独立错误码
需要输出 JSON 格式
```

到那时再考虑 `clap` 和 `commands.rs`。
