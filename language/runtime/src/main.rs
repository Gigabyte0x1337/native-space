// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

//! Runs the Native Space CLI and stdio MCP server.

mod mcp;

use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use mimalloc::MiMalloc;
use native_space_language::expansion::{derive, format_report, relativize_paths};
use native_space_language::{Document, compile, expand_source, inspect, load_document};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Debug, Parser)]
#[command(name = "native-space", version, about = "Native Space 1.0")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum Numeric {
    #[default]
    Exact,
    F64,
}

impl Numeric {
    fn output(
        self,
        state: &native_space_language::retained::State,
        kind: native_space_language::core::OutputKind,
    ) -> Result<serde_json::Value, String> {
        match self {
            Self::Exact => state.output_data(kind),
            Self::F64 => native_space_language::retained::numeric::output(state, kind),
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Evaluate a retained state; exact by default, explicitly rounded with --numeric f64.
    Run {
        file: String,
        /// Pass every ordered data item to one selected source function.
        #[arg(long, requires = "function")]
        data: Option<String>,
        /// Source function receiving the complete finite data pack.
        #[arg(long, requires = "data")]
        function: Option<String>,
        #[arg(long, value_enum, default_value_t = Numeric::Exact)]
        numeric: Numeric,
    },
    /// Parse and verify a state or proof document.
    Check { file: String },
    /// Print the schema-1 document representation.
    Inspect { file: String },
    /// Show each retained branch's exact coordinates without replacing its state.
    View {
        file: String,
        /// Quarter-turns of the observation frame; the stored state is unchanged.
        #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(i64).range(0..=3))]
        turns: i64,
        /// INDEX direction used for k in radius k+1; all other labels remain retained.
        #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u64).range(1..))]
        index_direction: u64,
        #[arg(long, value_enum, default_value_t = Numeric::Exact)]
        numeric: Numeric,
    },
    /// Print the generated pure source after transparent elaboration.
    Expand { file: String },
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
    /// Discover an exact deterministic or relationship-frequency pattern.
    Untrace {
        /// Exact relationship rank; one retains every discovered channel.
        #[arg(long, default_value = "1")]
        rank: String,
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
    /// Search lower relationship ranks under an exact or declared lossy policy.
    RankDescent {
        /// Rank-one reference rows; defaults to the supplied observation count.
        #[arg(long)]
        rows: Option<usize>,
        /// Adaptive search, every positive linear step, or one static target.
        #[arg(long, value_enum, default_value_t = RankDescentStrategy::Adaptive)]
        strategy: RankDescentStrategy,
        /// Exact decrement for linear search; defaults to one quarter.
        #[arg(long)]
        step: Option<String>,
        /// Exact retained-channel fraction for static search.
        #[arg(long)]
        rank: Option<String>,
        /// Exact required row agreement for static search; defaults to one.
        #[arg(long)]
        minimum_agreement: Option<String>,
        /// Existing exact-state observation document.
        #[arg(required_unless_present = "input")]
        file: Option<String>,
        /// Ordered JSON/NSBATCH states or scalar CSV.
        #[arg(long, conflicts_with = "file")]
        input: Option<String>,
        /// Search report or selected native source.
        #[arg(long, value_enum, default_value_t = RankDescentOutput::Report)]
        output: RankDescentOutput,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum RankDescentStrategy {
    Adaptive,
    Linear,
    Static,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum RankDescentOutput {
    Report,
    Source,
}

#[derive(Clone, Copy, Debug)]
struct RankDescentRun<'a> {
    file: &'a str,
    data_input: bool,
    rows: Option<usize>,
    strategy: RankDescentStrategy,
    step: Option<&'a str>,
    rank: Option<&'a str>,
    minimum_agreement: Option<&'a str>,
    output: RankDescentOutput,
}

#[tokio::main]
async fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Run {
            file,
            data,
            function,
            numeric,
        } => match (data.as_deref(), function.as_deref()) {
            (Some(data), Some(function)) => run_data_file(&file, function, data, numeric),
            (None, None) => run_file(&file, numeric),
            _ => report_error("run data and function must be supplied together"),
        },
        Command::Check { file } => check_file(&file),
        Command::Inspect { file } => read_document(&file).map_or_else(
            |error| report_error(&error),
            |document| print_json(&inspect(&document)),
        ),
        Command::View {
            file,
            turns,
            index_direction,
            numeric,
        } => view_file(&file, turns, index_direction, numeric),
        Command::Expand { file } => read_document(&file)
            .and_then(|document| expand_source(&document).map_err(|error| error.to_string()))
            .map_or_else(
                |error| report_error(&error),
                |source| {
                    println!("{source}");
                    ExitCode::SUCCESS
                },
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
            rank,
            file,
            input,
            output,
        } => match input.as_deref().or(file.as_deref()) {
            Some(source) => untrace_file(source, input.is_some(), &rank, output),
            None => report_error("untrace requires a source document or --input data file"),
        },
        Command::RankDescent {
            rows,
            strategy,
            step,
            rank,
            minimum_agreement,
            file,
            input,
            output,
        } => match input.as_deref().or(file.as_deref()) {
            Some(source) => rank_descent_file(&RankDescentRun {
                file: source,
                data_input: input.is_some(),
                rows,
                strategy,
                step: step.as_deref(),
                rank: rank.as_deref(),
                minimum_agreement: minimum_agreement.as_deref(),
                output,
            }),
            None => report_error("rank-descent requires a source document or --input data file"),
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
                let mut output = native_space_language::batch::output_observations(
                    "gpu",
                    steps,
                    &inputs,
                    &execution.states,
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

fn untrace_file(file: &str, data_input: bool, rank: &str, output: UntraceOutput) -> ExitCode {
    let result = if data_input {
        untrace_data(file, rank)
    } else {
        read_document(file).and_then(|document| {
            let Document::State(program) = document else {
                return Err("untrace expects an exact-state observation document".into());
            };
            let state = native_space_language::core::interpret(&program)
                .map_err(|error| error.to_string())?;
            native_space_language::discovery::discover(
                &state,
                rank,
                &program.source_name,
                program.result.span(),
            )
            .map_err(|error| error.to_string())
        })
    }
    .and_then(|pattern| match output {
        UntraceOutput::Source => Ok(pattern.source().to_owned()),
        UntraceOutput::PatternCsv => pattern.pattern_csv().map_err(|error| error.to_string()),
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
    rank: &str,
) -> Result<native_space_language::discovery::DiscoveredPattern, String> {
    let extension = Path::new(file)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    if extension.as_deref() == Some("csv") {
        let state = native_space_language::continuation::read_observations_csv(file)
            .map_err(|error| error.to_string())?;
        return native_space_language::discovery::discover(&state, rank, file, None)
            .map_err(|error| error.to_string());
    }
    let inputs =
        native_space_language::batch::read_data(file).map_err(|error| error.to_string())?;
    let values = inputs
        .into_iter()
        .map(|point| point.projection().clone())
        .collect::<Vec<_>>();
    native_space_language::discovery::discover_states(&values, rank, file)
        .map_err(|error| error.to_string())
}

fn rank_descent_file(run: &RankDescentRun<'_>) -> ExitCode {
    let result = rank_observations(run.file, run.data_input).and_then(|values| {
        let rows = run.rows.unwrap_or(values.len());
        let strategy = match (
            run.strategy,
            run.step,
            run.rank,
            run.minimum_agreement,
        ) {
            (RankDescentStrategy::Adaptive, None, None, None) => {
                native_space_language::rank_descent::RankStrategy::adaptive()
            }
            (RankDescentStrategy::Adaptive, _, _, _) => {
                return Err("adaptive rank descent accepts no rank controls".into());
            }
            (RankDescentStrategy::Linear, step, None, None) => {
                native_space_language::rank_descent::RankStrategy::linear(step.unwrap_or("1/4"))
            }
            (RankDescentStrategy::Linear, _, _, _) => {
                return Err("linear rank descent accepts only --step".into());
            }
            (RankDescentStrategy::Static, None, Some(rank), agreement) => {
                native_space_language::rank_descent::RankStrategy::static_target(
                    rank,
                    agreement.unwrap_or("1"),
                )
            }
            (RankDescentStrategy::Static, _, _, _) => {
                return Err("static rank descent requires --rank, accepts optional --minimum-agreement, and does not accept --step".into());
            }
        };
        native_space_language::rank_descent::descend_states(&values, rows, &strategy, run.file)
            .map_err(|error| error.to_string())
    });
    match result {
        Ok(search) => match run.output {
            RankDescentOutput::Report => print_json(&search.to_data()),
            RankDescentOutput::Source => {
                print!("{}", search.final_pattern().source());
                ExitCode::SUCCESS
            }
        },
        Err(error) => report_error(&error),
    }
}

fn rank_observations(
    file: &str,
    data_input: bool,
) -> Result<Vec<native_space_language::core::NativeState>, String> {
    if data_input {
        let extension = Path::new(file)
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase);
        if extension.as_deref() == Some("csv") {
            let state = native_space_language::continuation::read_observations_csv(file)
                .map_err(|error| error.to_string())?;
            return native_space_language::continuation::indexed_observations(&state, file, None)
                .map(|(_first_index, values)| values)
                .map_err(|error| error.to_string());
        }
        return native_space_language::batch::read_data(file)
            .map(|inputs| {
                inputs
                    .into_iter()
                    .map(|point| point.projection().clone())
                    .collect()
            })
            .map_err(|error| error.to_string());
    }

    read_document(file).and_then(|document| {
        let Document::State(program) = document else {
            return Err("rank-descent expects an exact-state observation document".into());
        };
        let state =
            native_space_language::core::interpret(&program).map_err(|error| error.to_string())?;
        native_space_language::continuation::indexed_observations(
            &state,
            &program.source_name,
            program.result.span(),
        )
        .map(|(_first_index, values)| values)
        .map_err(|error| error.to_string())
    })
}

fn read_document(file: &str) -> Result<Document, String> {
    load_document(file).map_err(|error| error.to_string())
}

fn view_file(file: &str, turns: i64, index_direction: u64, numeric: Numeric) -> ExitCode {
    let result = read_document(file).and_then(|document| {
        let Document::State(program) = document else {
            return Err("view expects an executable state document".into());
        };
        let artifact = native_space_language::compiled::compile(&program)
            .map_err(|error| error.to_string())?;
        let state = native_space_language::compiled::execute_retained(&artifact)
            .map_err(|error| error.to_string())?;
        native_space_language::retained::numeric::view(
            &state,
            index_direction,
            turns,
            numeric == Numeric::F64,
        )
    });
    result.map_or_else(|error| report_error(&error), |view| print_json(&view))
}

fn run_file(file: &str, numeric: Numeric) -> ExitCode {
    let result = read_document(file).and_then(|document| match document {
        Document::State(program) => native_space_language::compiled::compile(&program)
            .and_then(|bytecode| {
                let output_kind = bytecode.output_kind;
                native_space_language::compiled::execute_retained(&bytecode)
                    .map(|state| (state, output_kind))
            })
            .and_then(|(state, output_kind)| {
                numeric.output(&state, output_kind).map_err(|message| {
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

fn run_data_file(file: &str, function: &str, data: &str, numeric: Numeric) -> ExitCode {
    let result = read_document(file).and_then(|document| {
        let Document::State(program) = document else {
            return Err("run --data expects an exact-state source document".into());
        };
        let inputs =
            native_space_language::batch::read_data(data).map_err(|error| error.to_string())?;
        let state = native_space_language::batch::execute_together(&program, function, &inputs)
            .map_err(|error| error.to_string())?;
        numeric.output(&state, program.output_kind)
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
        Some("vector" | "pattern") => print_json(&value["value"]),
        _ => report_error("unknown output kind"),
    }
}

fn check_file(file: &str) -> ExitCode {
    let result = read_document(file).and_then(|document| match document {
        Document::State(program) => {
            let goal = program.goal;
            let direct = native_space_language::retained::interpret(&program)
                .map_err(|error| error.to_string())?;
            let bytecode = native_space_language::compiled::compile(&program)
                .map_err(|error| error.to_string())?;
            let bytecode =
                native_space_language::compiled::Artifact::from_data(&bytecode.to_data())?;
            let compiled = native_space_language::compiled::execute_retained(&bytecode)
                .map_err(|error| error.to_string())?;
            if !direct.same_structure(&compiled) || !direct.same_projection(&compiled) {
                return Err("source execution and the saved Native program disagree".into());
            }
            match goal {
                native_space_language::core::Goal::Emit => Ok(format!(
                    "Valid exact-state document: {}",
                    Path::new(file).display()
                )),
                native_space_language::core::Goal::ProveZero if direct.project().is_zero() => {
                    Ok(format!(
                        "Valid classical-projection zero proof: {}",
                        Path::new(file).display()
                    ))
                }
                native_space_language::core::Goal::ProveZero => Err(format!(
                    "zero proof failed: {} has a nonzero classical projection",
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
                &["identity_phase".to_owned(), "identity_phase".to_owned(),],
                false,
                None,
            ),
            ExitCode::SUCCESS
        );
    }
}
