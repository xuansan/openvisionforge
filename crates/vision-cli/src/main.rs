use std::{env, fs, path::PathBuf};

use vision_core::{Executor, Result, Workflow};

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
        _ => {
            println!("usage:");
            println!("  vision-cli list-operators");
            println!("  vision-cli list-operators-json");
            println!("  vision-cli export-schema");
            println!("  vision-cli describe-operator <operator-id>");
            println!("  vision-cli run <workflow.json>");
        }
    }
    Ok(())
}
