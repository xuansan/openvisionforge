# Vision Studio

React + Tauri desktop UI for the Rust vision runtime.

## What works now

- React loads operator schemas from the Tauri command `export_schema`.
- Tauri calls `vision-runtime`, so the UI and `vision-cli export-schema` use the same Rust node registry.
- The first screen shows node categories, node details, parameter schema, IO schema, and a minimal workflow JSON export preview.

## Commands

```powershell
npm install
npm run build
npm run tauri:dev
```

## Data flow

```text
React UI
  -> @tauri-apps/api invoke("export_schema")
  -> Tauri command
  -> vision-runtime::export_schema()
  -> vision-core OperatorRegistry
  -> vision-opencv / vision-infer / vision-io / vision-operators
```
