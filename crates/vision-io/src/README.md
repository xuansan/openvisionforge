# vision-io/src

Rust node layer for record, storage, communication, and action nodes.

```text
lib.rs
  Registers all IO node groups.

schema.rs
  Shared OperatorSchema helper.

util.rs
  Shared helpers for params, Value conversion, local file writes, and payload extraction.

nodes/record/
  image_save.rs
  result_json.rs
  trace.rs
  report_csv.rs
  record.image.save
  record.result.json
  record.trace
  record.report.csv

nodes/storage/
  local_write.rs
  local_read.rs
  nas_write.rs
  nas_read.rs
  rustfs_upload.rs
  s3_upload.rs
  oss_upload.rs
  storage.local.write
  storage.local.read
  storage.nas.write
  storage.nas.read
  storage.rustfs.upload
  storage.s3.upload
  storage.oss.upload

nodes/comm/
  websocket_send.rs
  http_post.rs
  webhook_send.rs
  mqtt_publish.rs
  rabbitmq_publish.rs
  kafka_publish.rs
  redis_publish.rs
  grpc_call.rs
  comm.websocket.send
  comm.http.post
  comm.webhook.send
  comm.mqtt.publish
  comm.rabbitmq.publish
  comm.kafka.publish
  comm.redis.publish
  comm.grpc.call

nodes/action/
  signal_output.rs
  io_digital_write.rs
  io_analog_write.rs
  plc_write.rs
  alarm_trigger.rs
  robot_call.rs
  camera_trigger.rs
  action.signal.output
  action.io.digital_write
  action.io.analog_write
  action.plc.write
  action.alarm.trigger
  action.robot.call
  action.camera.trigger

nodes/db/
  mongodb_insert.rs
  mongodb_find.rs
  mysql_insert.rs
  mysql_query.rs
  pg_insert.rs
  pg_query.rs
  milvus_insert.rs
  milvus_search.rs
  db.mongodb.insert
  db.mongodb.find
  db.mysql.insert
  db.mysql.query
  db.pg.insert
  db.pg.query
  db.milvus.insert
  db.milvus.search
```
