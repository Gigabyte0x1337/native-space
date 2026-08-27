// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Runs the Native Space CLI and stdio MCP server.

mod mcp;

use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
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
    /// Synthesize a verified program for one classical complex projection.
    Frequency {
        /// Exact-state source whose result contains the indexed samples.
        file: String,
        /// First positive INDEX direction in the projected window.
        #[arg(long, default_value_t = 1)]
        first_index: u64,
        /// Number of projected samples in the finite window.
        #[arg(long)]
        samples: usize,
        /// Maximum absolute classical reconstruction error.
        #[arg(long, default_value = "1e-12")]
        maximum_error: String,
    },
    /// Synthesize the smallest supported continuation from indexed observations.
    Untrace {
        /// Maximum exact ratio of held-out observation indexes allowed to disagree.
        #[arg(long, default_value = "0")]
        maximum_error_ratio: String,
        /// Existing exact-state observation document.
        #[arg(required_unless_present = "input")]
        file: Option<String>,
        /// Ordered JSON/NSBATCH states or scalar CSV.
        #[arg(long, conflicts_with = "file")]
        input: Option<String>,
        /// Generated source or the exact operation pattern as CSV.
        #[arg(long, value_enum, default_value_t = UntraceOutput::Source)]
        output: UntraceOutput,
    },
    /// Run one unary function over independent data points for a fixed step count.
    Batch {
        /// Exact-state source containing the selected unary function.
        file: String,
        /// Unary source-function name.
        #[arg(long)]
        function: String,
        /// Ordered JSON or NSBATCH binary data file.
        #[arg(long)]
        data: String,
        /// Sequential applications per data point.
        #[arg(long)]
        steps: u64,
        /// Execution device; neither backend silently falls back to the other.
        #[arg(long, value_enum, default_value_t = BatchBackend::Cpu)]
        backend: BatchBackend,
    },
    /// Pack readable batch data into the versioned binary input format.
    PackData {
        /// JSON batch data to validate and pack.
        input: String,
        /// Binary data file to create.
        output: String,
    },
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum BatchBackend {
    Cpu,
    Gpu,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum UntraceOutput {
    Source,
    PatternCsv,
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
        Command::Frequency {
            file,
            first_index,
            samples,
            maximum_error,
        } => frequency_file(&file, first_index, samples, &maximum_error),
        Command::Untrace {
            maximum_error_ratio,
            file,
            input,
            output,
        } => match input.as_deref().or(file.as_deref()) {
            Some(source) => untrace_file(source, input.is_some(), &maximum_error_ratio, output),
            None => report_error("untrace requires a source document or --input data file"),
        },
        Command::Batch {
            file,
            function,
            data,
            steps,
            backend,
        } => batch_file(&file, &function, &data, steps, backend).await,
        Command::PackData { input, output } => pack_data_file(&input, &output),
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

fn pack_data_file(input: &str, output: &str) -> ExitCode {
    native_space_language::batch::pack_data(input, output).map_or_else(
        |error| report_error(&error.to_string()),
        |count| {
            println!("Packed {count} data points: {output}");
            ExitCode::SUCCESS
        },
    )
}

fn frequency_file(file: &str, first_index: u64, samples: usize, maximum_error: &str) -> ExitCode {
    let result = read_document(file).and_then(|document| {
        let Document::State(program) = document else {
            return Err("frequency expects an exact-state source document".into());
        };
        let state =
            native_space_language::core::interpret(&program).map_err(|error| error.to_string())?;
        native_space_language::frequency::synthesize(
            &state,
            first_index,
            samples,
            maximum_error,
            &program.source_name,
        )
        .map(|frequency| frequency.to_data())
        .map_err(|error| error.to_string())
    });
    result.map_or_else(|error| report_error(&error), |value| print_json(&value))
}

async fn batch_file(
    file: &str,
    function: &str,
    data: &str,
    steps: u64,
    backend: BatchBackend,
) -> ExitCode {
    let result = async {
        let document = read_document(file)?;
        let Document::State(program) = document else {
            return Err("batch expects an exact-state source document".into());
        };
        let inputs =
            native_space_language::batch::read_data(data).map_err(|error| error.to_string())?;
        match backend {
            BatchBackend::Cpu => {
                native_space_language::batch::execute_cpu(&program, function, &inputs, steps)
                    .map(|results| {
                        native_space_language::batch::output_data("cpu", steps, &inputs, &results)
                    })
                    .map_err(|error| error.to_string())
            }
            BatchBackend::Gpu => {
                let execution =
                    native_space_language::gpu::execute(&program, function, &inputs, steps)
                        .await
                        .map_err(|error| error.to_string())?;
                let mut output = native_space_language::batch::output_data(
                    "gpu",
                    steps,
                    &inputs,
                    &execution.results,
                );
                output
                    .as_object_mut()
                    .expect("batch output is an object")
                    .insert("adapter".into(), execution.adapter_name.into());
                Ok(output)
            }
        }
    }
    .await;
    result.map_or_else(|error| report_error(&error), |value| print_json(&value))
}

fn untrace_file(
    file: &str,
    data_input: bool,
    maximum_error_ratio: &str,
    output: UntraceOutput,
) -> ExitCode {
    let result = if data_input {
        untrace_data(file, maximum_error_ratio)
    } else {
        read_document(file).and_then(|document| {
            let Document::State(program) = document else {
                return Err("untrace expects an exact-state observation document".into());
            };
            let state = native_space_language::core::interpret(&program)
                .map_err(|error| error.to_string())?;
            native_space_language::continuation::synthesize(
                &state,
                maximum_error_ratio,
                &program.source_name,
                program.result.span(),
            )
            .map_err(|error| error.to_string())
        })
    }
    .and_then(|continuation| match output {
        UntraceOutput::Source => Ok(continuation.source().to_owned()),
        UntraceOutput::PatternCsv => continuation
            .pattern_csv()
            .map_err(|error| error.to_string()),
    });
    result.map_or_else(
        |error| report_error(&error),
        |source| {
            print!("{source}");
            ExitCode::SUCCESS
        },
    )
}

fn untrace_data(
    file: &str,
    maximum_error_ratio: &str,
) -> Result<native_space_language::continuation::Continuation, String> {
    let extension = Path::new(file)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    if extension.as_deref() == Some("csv") {
        let state = native_space_language::continuation::read_observations_csv(file)
            .map_err(|error| error.to_string())?;
        return native_space_language::continuation::synthesize(
            &state,
            maximum_error_ratio,
            file,
            None,
        )
        .map_err(|error| error.to_string());
    }
    let inputs =
        native_space_language::batch::read_data(file).map_err(|error| error.to_string())?;
    let values = inputs
        .into_iter()
        .map(native_space_language::batch::DataPoint::into_state)
        .collect::<Vec<_>>();
    native_space_language::continuation::synthesize_states(&values, maximum_error_ratio, file)
        .map_err(|error| error.to_string())
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
