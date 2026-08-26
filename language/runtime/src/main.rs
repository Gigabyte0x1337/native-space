// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Runs the Native Space CLI and stdio MCP server.

mod mcp;

use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use mimalloc::MiMalloc;
use native_space_language::expansion::{derive, format_report, relativize_paths};
use native_space_language::{Document, compile, inspect, load_document};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Debug, Parser)]
#[command(name = "native-space", version, about = "Native Space 1.0")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse and evaluate an exact state document.
    Run { file: String },
    /// Parse and verify a state or proof document.
    Check { file: String },
    /// Print the schema-1 document representation.
    Inspect { file: String },
    /// Compile to schema-1 bytecode or a recomputable proof certificate.
    Compile { file: String },
    /// Expand one function into primitive operations.
    Derive {
        /// Emit the complete machine-readable report.
        #[arg(long)]
        json: bool,
        /// Load the function and its imports from this `.ns` file.
        #[arg(long)]
        source: Option<String>,
        /// Source-defined function name.
        function: String,
        /// Dynamic function names passed into the selected function.
        arguments: Vec<String>,
    },
    /// Serve the operation-derivation tool over MCP stdio.
    Mcp,
}

#[tokio::main]
async fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Run { file } => run_file(&file),
        Command::Check { file } => check_file(&file),
        Command::Inspect { file } => read_document(&file).map_or_else(
            |error| report_error(&error),
            |document| print_json(&inspect(&document)),
        ),
        Command::Compile { file } => read_document(&file)
            .and_then(|document| compile(&document).map_err(|error| error.to_string()))
            .map_or_else(
                |error| report_error(&error),
                |artifact| print_json(&artifact),
            ),
        Command::Derive {
            json,
            source,
            function,
            arguments,
        } => run_derivation(&function, &arguments, json, source.as_deref()),
        Command::Mcp => match mcp::run_stdio().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("MCP server stopped: {error}");
                ExitCode::from(2)
            }
        },
    }
}

fn read_document(file: &str) -> Result<Document, String> {
    load_document(file).map_err(|error| error.to_string())
}

fn run_file(file: &str) -> ExitCode {
    let result = read_document(file).and_then(|document| match document {
        Document::State(program) => native_space_language::bytecode::compile(&program)
            .and_then(|bytecode| {
                let output_kind = bytecode.output_kind;
                native_space_language::bytecode::execute(&bytecode)
                    .map(|state| (state, output_kind))
            })
            .and_then(|(state, output_kind)| {
                native_space_language::core::output_data(&state, output_kind).map_err(|message| {
                    native_space_language::core::LanguageError(
                        native_space_language::core::Diagnostic {
                            code: "NSO001".into(),
                            message,
                            source_name: program.source_name.clone(),
                            span: program.result.span(),
                        },
                    )
                })
            })
            .map_err(|error| error.to_string()),
        Document::Functions(_) | Document::Logic(_) => Err(
            "function libraries and Boolean proofs are checked with 'native-space check'".into(),
        ),
    });
    result.map_or_else(|error| report_error(&error), |value| print_output(&value))
}

fn print_output(value: &serde_json::Value) -> ExitCode {
    match value.get("kind").and_then(serde_json::Value::as_str) {
        Some("number" | "string") => {
            println!("{}", value["value"].as_str().unwrap_or_default());
            ExitCode::SUCCESS
        }
        Some("boolean") => {
            println!("{}", value["value"].as_bool().unwrap_or(false));
            ExitCode::SUCCESS
        }
        Some("pattern") => print_json(&value["value"]),
        _ => report_error("unknown output kind"),
    }
}

fn check_file(file: &str) -> ExitCode {
    let result = read_document(file).and_then(|document| match document {
        Document::State(program) => {
            let goal = program.goal;
            let direct = native_space_language::core::interpret(&program)
                .map_err(|error| error.to_string())?;
            let bytecode = native_space_language::bytecode::compile(&program)
                .map_err(|error| error.to_string())?;
            let compiled = native_space_language::bytecode::execute(&bytecode)
                .map_err(|error| error.to_string())?;
            if direct != compiled {
                return Err("the evaluator and bytecode machine disagree".into());
            }
            match goal {
                native_space_language::core::Goal::Emit => Ok(format!(
                    "Valid exact-state document: {}",
                    Path::new(file).display()
                )),
                native_space_language::core::Goal::ProveZero if direct.is_zero() => {
                    Ok(format!("Valid zero proof: {}", Path::new(file).display()))
                }
                native_space_language::core::Goal::ProveZero => Err(format!(
                    "zero proof failed: {} evaluates to a nonzero native state",
                    Path::new(file).display()
                )),
            }
        }
        Document::Functions(library) => Ok(format!(
            "Valid function library: {}\nFunctions: {}",
            Path::new(file).display(),
            library.functions.len()
        )),
        Document::Logic(program) => {
            let certificate = native_space_language::logic::compile(&program)
                .map_err(|error| error.to_string())?;
            let report = native_space_language::logic::verify(&certificate)?;
            Ok(format!(
                "Valid Boolean proof: {}\nChecked valuations: {}",
                Path::new(file).display(),
                report.valuation_count
            ))
        }
    });
    result.map_or_else(
        |error| report_error(&error),
        |message| {
            println!("{message}");
            ExitCode::SUCCESS
        },
    )
}

fn print_json(value: &serde_json::Value) -> ExitCode {
    match serde_json::to_string_pretty(value) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => report_error(&format!("could not create output: {error}")),
    }
}

fn report_error(message: &str) -> ExitCode {
    eprintln!("Could not complete: {message}");
    ExitCode::from(2)
}

fn run_derivation(
    function: &str,
    arguments: &[String],
    json: bool,
    source: Option<&str>,
) -> ExitCode {
    let result = if let Some(file) = source {
        let library = match native_space_language::derivation::load(file) {
            Ok(library) => library,
            Err(error) => {
                eprintln!("Could not derive operations: {error}");
                return ExitCode::from(2);
            }
        };
        native_space_language::expansion::derive_from_library(&library, function, arguments)
    } else {
        derive(function, arguments)
    };
    let mut report = match result {
        Ok(report) => report,
        Err(error) => {
            eprintln!("Could not derive operations: {}", error.summary());
            return ExitCode::from(2);
        }
    };

    if let Ok(root) = std::env::current_dir() {
        relativize_paths(&mut report, root);
    }

    if json {
        match serde_json::to_string_pretty(&report) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                eprintln!("Could not create operation report: {error}");
                return ExitCode::from(2);
            }
        }
    } else {
        println!("{}", format_report(&report));
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derivations_succeed_when_functions_expand() {
        assert_eq!(
            run_derivation(
                "axis_subtract",
                &[
                    "identity_orientation".to_owned(),
                    "identity_orientation".to_owned(),
                ],
                false,
                None,
            ),
            ExitCode::SUCCESS
        );
    }
}
