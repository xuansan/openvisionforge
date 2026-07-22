# vision-io

`vision-io` contains non-vision workflow nodes:

```text
record.*    trace, result, image, and report records
storage.*   local/object-storage style file movement
comm.*      outbound communication messages
action.*    external action or device commands
db.*        database operations
```

These are nodes, not image operators. They consume and produce `vision-core`
`Value` data just like OpenCV nodes, so they can be composed in the same
workflow DAG.

## Current Nodes

```text
record.image.save
record.result.json
record.trace
record.report.csv

storage.local.write
storage.local.read
storage.nas.write
storage.nas.read
storage.rustfs.upload
storage.s3.upload
storage.oss.upload

comm.websocket.send
comm.http.post
comm.webhook.send
comm.mqtt.publish
comm.rabbitmq.publish
comm.kafka.publish
comm.redis.publish
comm.grpc.call

action.signal.output
action.io.digital_write
action.io.analog_write
action.plc.write
action.alarm.trigger
action.robot.call
action.camera.trigger

db.mongodb.insert
db.mongodb.find
db.mysql.insert
db.mysql.query
db.pg.insert
db.pg.query
db.milvus.insert
db.milvus.search
```

## MVP Behavior

Local record and storage nodes perform real filesystem writes.

Network, message queue, object storage, database, PLC, IO, and robot nodes
currently write structured outbox or manifest JSON files under
`records/<run_id>/`, or to an explicit `outbox`/`manifest` path. This keeps
workflow IDs, ports, params, trace, and UI/agent integration stable while
avoiding heavy runtime dependencies in the MVP. Later, feature-gated clients can
replace the outbox implementation without changing workflow files.

## Parameter Metadata

Communication, database, and storage nodes now expose `ParamSchema` metadata
through `OperatorSchema.params`. The CLI can print these schemas for UI or agent
use:

```bash
cargo run -p vision-cli -- describe-operator comm.kafka.publish
cargo run -p vision-cli -- describe-operator db.milvus.search
cargo run -p vision-cli -- describe-operator storage.nas.write
```

## Feature Skeleton

`vision-io` defines empty feature flags for future real SDK integrations:

```text
comm-http
comm-websocket
comm-redis
comm-kafka
comm-rabbitmq
comm-mqtt
comm-grpc
db-mongodb
db-mysql
db-postgres
db-milvus
storage-rustfs
storage-s3
storage-oss
```

The current MVP does not pull these SDKs into the binary. When a real backend is
implemented, the matching client crate should be attached to the relevant
feature so deployment builds stay small.

## File Layout Rule

Each supported node has its own Rust source file. For example:

```text
comm.kafka.publish
  src/nodes/comm/kafka_publish.rs

comm.websocket.send
  src/nodes/comm/websocket_send.rs

storage.rustfs.upload
  src/nodes/storage/rustfs_upload.rs

storage.nas.write
  src/nodes/storage/nas_write.rs

action.plc.write
  src/nodes/action/plc_write.rs

db.mongodb.insert
  src/nodes/db/mongodb_insert.rs

db.mongodb.find
  src/nodes/db/mongodb_find.rs

db.mysql.insert
  src/nodes/db/mysql_insert.rs

db.mysql.query
  src/nodes/db/mysql_query.rs

db.pg.insert
  src/nodes/db/pg_insert.rs

db.pg.query
  src/nodes/db/pg_query.rs

db.milvus.insert
  src/nodes/db/milvus_insert.rs

db.milvus.search
  src/nodes/db/milvus_search.rs

record.image.save
  src/nodes/record/image_save.rs
```

The category `mod.rs` files only aggregate registration and shared helpers.
