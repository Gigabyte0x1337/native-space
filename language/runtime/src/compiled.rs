// SPDX-License-Identifier: AGPL-3.0-or-later

//! Compiled documents store the Native program, never a pre-executed result.
//!
//! The cached address index is rebuilt once when loading an artifact. Calls then
//! reuse that graph; only environments change. Closed-state bytecode lowering is
//! a separate operation for replaying an already constructed computation.
use crate::{
    core::{Goal, LanguageError, OutputKind, Program},
    retained::State,
    strand::execution::Graph,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug)]
pub struct Artifact {
    graph: Arc<Graph>,
    source_name: String,
    pub goal: Goal,
    pub output_kind: OutputKind,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredProgram {
    schema: String,
    version: u64,
    source_name: String,
    goal: Goal,
    output_kind: OutputKind,
    graph: Value,
}

/// Compile structure without running any function or observation.
///
/// # Errors
/// Returns a located program-validation error.
pub fn compile(program: &Program) -> Result<Artifact, LanguageError> {
    Ok(Artifact {
        graph: Graph::compile(program)?,
        source_name: program.source_name.clone(),
        goal: program.goal,
        output_kind: program.output_kind,
    })
}

impl Artifact {
    /// Select a shared compiled function for repeated host input calls.
    ///
    /// # Errors
    /// Returns a diagnostic if the function is absent.
    pub fn function(
        &self,
        name: &str,
    ) -> Result<crate::strand::execution::FunctionValue, LanguageError> {
        self.graph.function(name)
    }
    #[must_use]
    pub fn to_data(&self) -> Value {
        json!({"schema":"native-space-program", "version":1, "source_name":self.source_name,
            "goal":self.goal, "output_kind":self.output_kind, "graph":self.graph.native().native_data()})
    }

    /// Validate a persisted Native graph and build its reusable execution index.
    ///
    /// # Errors
    /// Rejects malformed metadata or program records.
    pub fn from_data(data: &Value) -> Result<Self, String> {
        let stored: StoredProgram =
            serde_json::from_value(data.clone()).map_err(|error| error.to_string())?;
        if stored.schema != "native-space-program" || stored.version != 1 {
            return Err("unsupported Native program artifact".into());
        }
        let source_name = stored.source_name;
        let graph = Graph::load(State::from_data(&stored.graph)?, &source_name)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            graph,
            source_name,
            goal: stored.goal,
            output_kind: stored.output_kind,
        })
    }
}

/// Run the shared compiled graph with fresh top-level bindings.
///
/// # Errors
/// Returns a located runtime or execution-limit diagnostic.
pub fn execute_retained(program: &Artifact) -> Result<State, LanguageError> {
    program.graph.run()?.state(&program.source_name, None)
}
