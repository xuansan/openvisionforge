use std::{env, fs, path::PathBuf};

use vision_core::{Executor, Result, StreamConfig, Workflow};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let registry = vision_runtime::registry();

    let args = env::args().collect::<Vec<_>>();
    match args.get(1).map(String::as_str) {
        Some("list-operators") => {
            for op in registry.list() {
                println!("{}\t{}\t{}", op.id, op.category, op.name);
            }
        }
        Some("list-operators-json") | Some("export-schema") => {
            println!(
                "{}",
                serde_json::to_string_pretty(&vision_runtime::export_schema())?
            );
        }
        Some("describe-operator") => {
            let id = args
                .get(2)
                .expect("usage: vision-cli describe-operator <operator-id>");
            let op = registry.get(id)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&vision_runtime::operator_json(op))?
            );
        }
        Some("run") => {
            let path = args.get(2).expect("usage: vision-cli run <workflow.json>");
            let text = fs::read_to_string(path)?;
            let workflow: Workflow = serde_json::from_str(&text)?;
            let trace = Executor::new(&registry).run(&workflow, PathBuf::from("records"))?;
            let event_path = format!("records/{}/event.json", trace.run_id);
            let event: serde_json::Value = serde_json::from_str(&fs::read_to_string(&event_path)?)?;
            println!("Workflow loaded: {}", workflow.name);
            println!("Run: {}", trace.run_id);
            println!("Nodes: {}", trace.nodes.len());
            println!("Result: {}", event["result"].as_str().unwrap_or("UNKNOWN"));
            println!(
                "Processing: {} ms, frames {}, fps {}",
                trace.metrics.total_duration_ms, trace.metrics.processed_frames, trace.metrics.fps
            );
            println!("Trace: records/{}/trace.json", trace.run_id);
            println!("Event: {event_path}");
        }
        Some("run-folder") => {
            let workflow_path = args
                .get(2)
                .expect("usage: vision-cli run-folder <workflow.json> <image-folder> [max-frames]");
            let folder = args
                .get(3)
                .expect("usage: vision-cli run-folder <workflow.json> <image-folder> [max-frames]");
            let max_frames = args.get(4).and_then(|value| value.parse::<u64>().ok());
            let workflow: Workflow = serde_json::from_str(&fs::read_to_string(workflow_path)?)?;
            let trace = vision_runtime::StreamExecutor::new(&registry).run_folder(
                &workflow,
                folder,
                PathBuf::from("records"),
                StreamConfig {
                    max_frames,
                    ..Default::default()
                },
            )?;
            println!("Workflow loaded: {}", workflow.name);
            println!(
                "Frames: received {}, processed {}, failed {}, dropped {}",
                trace.metrics.received_frames,
                trace.metrics.processed_frames,
                trace.metrics.failed_frames,
                trace.metrics.dropped_frames
            );
            println!(
                "Processing: {} ms, fps {}",
                trace.metrics.total_duration_ms, trace.metrics.processing_fps
            );
            println!("Stream trace: records/stream_trace.json");
        }
        _ => {
            println!("usage:");
            println!("  vision-cli list-operators");
            println!("  vision-cli list-operators-json");
            println!("  vision-cli export-schema");
            println!("  vision-cli describe-operator <operator-id>");
            println!("  vision-cli run <workflow.json>");
            println!("  vision-cli run-folder <workflow.json> <image-folder> [max-frames]");
        }
    }
    Ok(())
}
