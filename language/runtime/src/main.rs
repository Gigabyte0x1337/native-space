// SPDX-License-Identifier: AGPL-3.0-or-later
//! File execution and a small stdio MCP boundary share the same exact evaluator.
use clap::{Parser, Subcommand};
use native_space_language::{
    camera,
    runtime::{Session, Value},
};
use std::path::PathBuf;
mod mcp;
#[derive(Parser, Debug)]
#[command(version, about = "Native Space 2: exact L/A/M programs")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand, Debug)]
enum Command {
    Run { file: PathBuf },
    Check { file: PathBuf },
    Inspect { file: PathBuf },
    Mcp,
}
fn run_source(source: &str, name: &str) -> Result<String, String> {
    let session = Session::new(source, name)?;
    match session.graph.format.as_str() {
        "number" => Ok(session.output.number_readout()?.to_string()),
        "vector" => serde_json::to_string(
            &session
                .output
                .scalar()?
                .state()
                .coordinates()
                .map(|v| v.to_string()),
        )
        .map_err(|e| e.to_string()),
        "program" => session.output.to_json(),
        _ => {
            let v = match session.output {
                Value::Observation(ref o) => o.evaluate(1_000_000)?,
                ref v => v.clone(),
            };
            v.to_json()
        }
    }
}
fn execute() -> Result<(), String> {
    match Cli::parse().command {
        Command::Mcp => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?
            .block_on(mcp::run()),
        command => {
            let file = match &command {
                Command::Run { file } | Command::Check { file } | Command::Inspect { file } => file,
                _ => unreachable!(),
            };
            let source = std::fs::read_to_string(file).map_err(|e| e.to_string())?;
            let name = file.display().to_string();
            let text = match command {
                Command::Run { .. } => run_source(&source, &name)?,
                Command::Check { .. } => {
                    let s = Session::new(&source, &name)?;
                    if let Value::Observation(o) = s.output {
                        o.evaluate(1_000_000)?;
                    }
                    "Program executed successfully; this is not a universal proof.".into()
                }
                Command::Inspect { .. } => {
                    let s = Session::new(&source, &name)?;
                    let p = s.output.scalar()?;
                    serde_json::json!({"state":p.state().coordinates().map(|x|x.to_string()),"native":camera::raw(p.state()).ok(),"classical":camera::classical(p.state()).ok(),"log_ratio":camera::log_ratio(p.state()).ok()}).to_string()
                }
                _ => unreachable!(),
            };
            println!("{text}");
            Ok(())
        }
    }
}
fn main() {
    if let Err(error) = execute() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
