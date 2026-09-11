// SPDX-License-Identifier: AGPL-3.0-or-later

//! Executes shared operation-strand records with independent binding environments.
//!
//! Coordinate records are the program. The child-address table is a validated
//! execution index over those records, not expanded or substituted source.

use std::sync::Arc;

use super::*;
use crate::retained::State;

mod environment;
mod machine;
pub mod state_data;
mod template;
#[cfg(test)]
mod tests;
mod validation;
enum Bound {
    Partial(Value),
    Ready {
        graph: Arc<Graph>,
        address: usize,
        environment: BTreeMap<String, Value>,
        retained: Vec<State>,
    },
}
use environment::Environment;

// Function entry pointer outside the existing high-direction strand schema.
const FUNCTION_ROOT: u64 = 5;

// Execution limits bound host work, not the Native graph's mathematical meaning.
const MAX_RECORDS: usize = 100_000;
const MAX_CALL_DEPTH: usize = 128;
const MAX_STEPS: usize = 1_000_000;
const ENTRY: u64 = 26;
const BINDING: u64 = 27;

#[derive(Debug)]
struct Signature {
    start: usize,
    end: usize,
    parameters: Vec<String>,
    body: usize,
    variadic: bool,
}

/// One immutable Native program and its validated record-address index.
#[derive(Debug)]
pub struct Graph {
    native: State,
    records: Vec<Coordinate>,
    children: Vec<Vec<usize>>,
    functions: BTreeMap<String, Signature>,
    bindings: Vec<(String, usize)>,
    entry: Option<usize>,
    root: String,
    source: String,
    rules: BTreeMap<usize, crate::value_reflection::Rule>,
}

/// A value or a reference to a graph with its own arguments.
#[derive(Clone, Debug)]
pub enum Value {
    State(State),
    Function(FunctionValue),
    Pack(Vec<Self>),
}

/// A shared graph reference and immutable per-call bindings.
#[derive(Clone, Debug)]
pub struct FunctionValue {
    graph: Arc<Graph>,
    name: String,
    environment: Environment,
}

#[derive(Debug)]
struct Budget {
    remaining: usize,
    depth: usize,
    // Per-run decoded views retain their source state, so an address cannot be
    // reused while cached. Keeping this outside State avoids an Arc cycle.
    loaded: BTreeMap<usize, FunctionValue>,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            remaining: MAX_STEPS,
            depth: 0,
            loaded: BTreeMap::new(),
        }
    }
}

fn diagnostic(message: impl Into<String>, source: &str, span: Option<Span>) -> LanguageError {
    instruction_error("NSG001", message, source, span)
}

impl Graph {
    /// Compile source into one strand without evaluating function bodies.
    ///
    /// # Errors
    /// Returns located errors for invalid records, names, or template rules.
    pub fn compile(program: &Program) -> Result<Arc<Self>, LanguageError> {
        crate::core::validate(program)?;
        let mut start = Coordinate::new(TRACE_START, program.span);
        start.name = Some("$entry".into());
        start.source = Some(program.source_name.clone());
        start.number_a = Some(OPERATION_STRAND_VERSION.to_string());
        let mut records = vec![start];
        for function in &program.functions {
            let mut start = Coordinate::new(FUNCTION_START, function.span);
            start.name = Some(function.name.clone());
            start.number_a = Some(function.parameters.len().to_string());
            start.number_b = Some(u8::from(function.variadic).to_string());
            records.push(start);
            for (position, name) in function.parameters.iter().enumerate() {
                let mut parameter = Coordinate::new(PARAMETER, function.span);
                parameter.name = Some(name.clone());
                parameter.number_a = Some(position.to_string());
                records.push(parameter);
            }
            collect_expression(&function.body, &mut records);
            let mut end = Coordinate::new(FUNCTION_END, function.span);
            end.name = Some(function.name.clone());
            records.push(end);
        }
        for binding in &program.bindings {
            let mut start = Coordinate::new(BINDING, binding.span);
            start.name = Some(binding.name.clone());
            records.push(start);
            collect_expression(&binding.value, &mut records);
        }
        records.push(Coordinate::new(ENTRY, program.span));
        collect_expression(&program.result, &mut records);
        let native = literal(&nest(records.clone()))?;
        Self::indexed(native, records, &program.source_name)
    }

    /// Validate an existing Native strand without reconstructing its function bodies.
    ///
    /// # Errors
    /// Returns a located diagnostic for malformed or oversized Native records.
    pub fn load(native: State, source: &str) -> Result<Arc<Self>, LanguageError> {
        let count = operation_length(native.project(), source, None)?;
        if count > MAX_RECORDS as u64 {
            return Err(diagnostic("program exceeds the record limit", source, None));
        }
        let records = decode_coordinates(native.project(), source, None)?;
        Self::indexed(native, records, source)
    }

    fn indexed(
        native: State,
        records: Vec<Coordinate>,
        source: &str,
    ) -> Result<Arc<Self>, LanguageError> {
        if records.is_empty()
            || records.len() > MAX_RECORDS
            || records[0].kind != TRACE_START
            || records[0].number_a.as_deref() != Some("1")
        {
            return Err(diagnostic("invalid Native program header", source, None));
        }
        let root = required_text(records[0].name.as_deref(), source, None)?.to_owned();
        let mut graph = Self {
            native,
            children: vec![Vec::new(); records.len()],
            records,
            functions: BTreeMap::new(),
            bindings: Vec::new(),
            entry: None,
            root,
            source: source.into(),
            rules: BTreeMap::new(),
        };
        let mut cursor = 1;
        while cursor < graph.records.len() {
            let record = &graph.records[cursor];
            match record.kind {
                FUNCTION_START => {
                    cursor = graph.index_function(cursor, source)?;
                }
                BINDING => {
                    let name =
                        required_text(record.name.as_deref(), source, record.span)?.to_owned();
                    if graph.bindings.iter().any(|(bound, _)| bound == &name) {
                        return Err(diagnostic("duplicate binding", source, record.span));
                    }
                    cursor += 1;
                    graph.bindings.push((name, cursor));
                    graph.index_expression(&mut cursor, 0)?;
                }
                ENTRY => {
                    if graph.entry.is_some() {
                        return Err(diagnostic("duplicate entry", source, record.span));
                    }
                    cursor += 1;
                    graph.entry = Some(cursor);
                    graph.index_expression(&mut cursor, 0)?;
                    if cursor != graph.records.len() {
                        return Err(diagnostic("records after entry", source, None));
                    }
                }
                _ => {
                    return Err(diagnostic(
                        "expected function, binding, or entry",
                        source,
                        record.span,
                    ));
                }
            }
        }
        if graph.entry.is_none() && !graph.functions.contains_key(&graph.root) {
            return Err(diagnostic("missing root function", source, None));
        }
        graph.validate_references()?;
        Ok(Arc::new(graph))
    }

    fn index_function(&mut self, mut cursor: usize, source: &str) -> Result<usize, LanguageError> {
        let record = &self.records[cursor];
        let start_address = cursor;
        let name = required_text(record.name.as_deref(), source, record.span)?.to_owned();
        let count = usize_or_zero(record.number_a.as_deref(), source, record.span)?;
        if count > self.records.len() {
            return Err(diagnostic("invalid parameter count", source, record.span));
        }
        let variadic = match record.number_b.as_deref() {
            None | Some("0") => false,
            Some("1") => true,
            _ => return Err(diagnostic("invalid variadic flag", source, record.span)),
        };
        cursor += 1;
        let mut parameters = Vec::new();
        for position in 0..count {
            let parameter = self
                .records
                .get(cursor)
                .ok_or_else(|| diagnostic("missing parameter", source, None))?;
            if parameter.kind != PARAMETER
                || usize_or_zero(parameter.number_a.as_deref(), source, parameter.span)? != position
            {
                return Err(diagnostic(
                    "invalid parameter record",
                    source,
                    parameter.span,
                ));
            }
            let name = required_text(parameter.name.as_deref(), source, parameter.span)?.to_owned();
            if parameters.contains(&name) {
                return Err(diagnostic("duplicate parameter", source, parameter.span));
            }
            parameters.push(name);
            cursor += 1;
        }
        if variadic && parameters.is_empty() {
            return Err(diagnostic(
                "variadic function requires a parameter",
                source,
                None,
            ));
        }
        let body = cursor;
        self.index_expression(&mut cursor, 0)?;
        let end = self
            .records
            .get(cursor)
            .ok_or_else(|| diagnostic("missing function end", source, None))?;
        if end.kind != FUNCTION_END || end.name.as_deref() != Some(&name) {
            return Err(diagnostic("mismatched function boundary", source, end.span));
        }
        if self
            .functions
            .insert(
                name,
                Signature {
                    start: start_address,
                    end: cursor,
                    parameters,
                    body,
                    variadic,
                },
            )
            .is_some()
        {
            return Err(diagnostic("duplicate function", source, end.span));
        }
        cursor += 1;
        Ok(cursor)
    }

    fn index_expression(&mut self, cursor: &mut usize, depth: usize) -> Result<(), LanguageError> {
        let address = *cursor;
        let record = self
            .records
            .get(address)
            .ok_or_else(|| diagnostic("missing expression record", &self.source, None))?;
        if depth >= MAX_CALL_DEPTH {
            return Err(diagnostic(
                "expression exceeds the nesting limit",
                &self.source,
                record.span,
            ));
        }
        let number = |text: Option<&str>| usize_or_zero(text, &self.source, record.span);
        let count = match record.kind {
            LITERAL | REFERENCE | SPREAD => 0,
            PHASE | INDEX | INDEX_CAPTURE => 1,
            ADD | MULTIPLY | REFLECT => number(record.number_a.as_deref())?,
            CALL => number(record.number_a.as_deref())?
                .checked_add(1)
                .ok_or_else(|| diagnostic("argument count overflow", &self.source, record.span))?,
            _ => {
                return Err(diagnostic(
                    "unknown expression record",
                    &self.source,
                    record.span,
                ));
            }
        };
        if record.kind == PHASE
            && !is_canonical_phase(i64_or_zero(
                record.number_a.as_deref(),
                &self.source,
                record.span,
            )?)
        {
            return Err(diagnostic(
                "phase must be from zero through three",
                &self.source,
                record.span,
            ));
        }
        if count > self.records.len().saturating_sub(address + 1) {
            return Err(diagnostic("invalid child count", &self.source, record.span));
        }
        *cursor += 1;
        for _ in 0..count {
            self.children[address].push(*cursor);
            self.index_expression(cursor, depth + 1)?;
        }
        if self.records[address].kind == REFLECT
            && self.records[address].name.as_deref() == Some("reflect")
        {
            if count != 3 {
                return Err(diagnostic(
                    "reflect requires three children",
                    &self.source,
                    self.records[address].span,
                ));
            }
            let pattern = template::View {
                graph: self,
                address: self.children[address][1],
            };
            let replacement = template::View {
                graph: self,
                address: self.children[address][2],
            };
            let rule = crate::value_reflection::Rule::compile_view(pattern, replacement)
                .map_err(|message| diagnostic(message, &self.source, self.records[address].span))?;
            self.rules.insert(address, rule);
        }
        Ok(())
    }

    /// Return the authoritative Native record state.
    #[must_use]
    pub fn native(&self) -> &State {
        &self.native
    }

    /// Reference a compiled function without binding or evaluating its body.
    ///
    /// # Errors
    /// Returns a diagnostic if the function is absent.
    pub fn function(self: &Arc<Self>, name: &str) -> Result<FunctionValue, LanguageError> {
        if !self.functions.contains_key(name) {
            return Err(diagnostic(
                format!("unknown function {name:?}"),
                &self.source,
                None,
            ));
        }
        Ok(FunctionValue {
            graph: Arc::clone(self),
            name: name.into(),
            environment: Environment::default(),
        })
    }

    /// Run top-level bindings and the entry against this shared graph.
    ///
    /// # Errors
    /// Returns located binding, call, or execution-limit diagnostics.
    pub fn run(self: &Arc<Self>) -> Result<Value, LanguageError> {
        let mut budget = Budget::default();
        let mut environment = BTreeMap::new();
        for (name, address) in &self.bindings {
            let value = self.evaluate(*address, &environment, &mut budget)?;
            environment.insert(name.clone(), value);
        }
        match self.entry {
            Some(address) => self
                .evaluate(address, &environment, &mut budget)
                .map(|value| match value {
                    Value::State(state) => Value::State(
                        state.retaining(
                            &environment
                                .values()
                                .flat_map(Value::dependencies)
                                .collect::<Vec<_>>(),
                        ),
                    ),
                    other => other,
                }),
            None => Ok(Value::Function(self.function(&self.root)?)),
        }
    }

    fn evaluate(
        self: &Arc<Self>,
        address: usize,
        environment: &BTreeMap<String, Value>,
        budget: &mut Budget,
    ) -> Result<Value, LanguageError> {
        machine::evaluate(Arc::clone(self), address, environment.clone(), budget)
    }

    fn reduce(
        self: &Arc<Self>,
        address: usize,
        environment: &BTreeMap<String, Value>,
        values: &[Value],
    ) -> Result<Value, LanguageError> {
        let record = &self.records[address];
        let state = |position: usize| values[position].clone().state(&self.source, record.span);
        let result = match record.kind {
            LITERAL => Value::State(State::scalar(
                NativeScalar::from_text(
                    record.text_a.as_deref().unwrap_or("0"),
                    record.text_b.as_deref().unwrap_or("0"),
                )
                .map_err(|message| diagnostic(message, &self.source, record.span))?,
            )),
            REFERENCE | SPREAD => {
                let name = record.name.as_deref().ok_or_else(|| {
                    diagnostic("missing reference name", &self.source, record.span)
                })?;
                let value = environment
                    .get(name)
                    .cloned()
                    .map_or_else(|| self.function(name).map(Value::Function), Ok)?;
                match value {
                    Value::Pack(items) if record.kind == REFERENCE => {
                        Value::State(indexed_pack(&items, &self.source, record.span)?)
                    }
                    other => other,
                }
            }
            ADD | MULTIPLY => {
                let mut result = if record.kind == ADD {
                    State::zero()
                } else {
                    State::one()
                };
                for value in values.iter().cloned() {
                    let value = value.state(&self.source, record.span)?;
                    result = if record.kind == ADD {
                        result.add(&value)
                    } else {
                        result.multiply(&value)
                    };
                }
                Value::State(result)
            }
            PHASE => Value::State(state(0)?.phase(i64_or_zero(
                record.number_a.as_deref(),
                &self.source,
                record.span,
            )?)),
            INDEX => Value::State(
                state(0)?
                    .index_power(
                        required_u64(record.number_a.as_deref(), &self.source, record.span)?,
                        required_u64(record.number_b.as_deref(), &self.source, record.span)?,
                    )
                    .map_err(|message| diagnostic(message, &self.source, record.span))?,
            ),
            REFLECT if self.rules.contains_key(&address) => {
                Value::State(state(0)?.reflect(&self.rules[&address]))
            }
            _ => {
                return Err(diagnostic(
                    "record is not executable",
                    &self.source,
                    record.span,
                ));
            }
        };
        Ok(match result {
            Value::State(state) => Value::State(state.at_span(record.span)),
            other => other,
        })
    }

    fn export_function(&self, name: &str) -> Result<State, LanguageError> {
        let mut start = self.records[0].clone();
        start.name = Some(name.into());
        start.span = None;
        let mut records = vec![start];
        let mut pending = vec![name.to_owned()];
        let mut seen = BTreeSet::new();
        while let Some(name) = pending.pop() {
            if !seen.insert(name.clone()) {
                continue;
            }
            let signature = self
                .functions
                .get(&name)
                .ok_or_else(|| diagnostic("unknown traced function", &self.source, None))?;
            records.extend_from_slice(&self.records[signature.start..=signature.end]);
            let mut references = Vec::new();
            let mut expressions = vec![signature.body];
            while let Some(address) = expressions.pop() {
                let record = &self.records[address];
                if self.rules.contains_key(&address) {
                    expressions.push(self.children[address][0]);
                } else {
                    expressions.extend(self.children[address].iter().rev());
                }
                let name = match record.kind {
                    REFERENCE => record.name.as_deref(),
                    _ => None,
                };
                if let Some(name) = name {
                    if !signature
                        .parameters
                        .iter()
                        .any(|parameter| parameter == name)
                        && self.functions.contains_key(name)
                    {
                        references.push(name.to_owned());
                    }
                }
            }
            pending.extend(references.into_iter().rev());
        }
        literal(&nest(records))
    }
}

impl FunctionValue {
    /// Decode callable Native graph records and bindings without compiling source.
    ///
    /// # Errors
    /// Rejects malformed graph records, bindings, roots, or decoding limits.
    pub fn from_native(native: &State) -> Result<Self, LanguageError> {
        Self::load(native, "<native-function>")
    }

    pub(crate) fn has_exact_arity(&self, count: usize) -> bool {
        let signature = &self.graph.functions[&self.name];
        !signature.variadic
            && signature
                .parameters
                .len()
                .checked_sub(self.environment.len())
                == Some(count)
    }

    /// Export an unbound function for explicit source-level tooling.
    ///
    /// # Errors
    /// Bound environments cannot be discarded by source-level rewriting.
    pub(crate) fn source_graph(native: &State, source: &str) -> Result<State, LanguageError> {
        let function = Self::load(native, source)?;
        if function.environment.len() != 0 {
            return Err(diagnostic(
                "source tooling requires an unbound function",
                source,
                None,
            ));
        }
        function.graph.export_function(&function.name)
    }

    /// Bind arguments, sharing the graph; evaluate only when its required slots are filled.
    ///
    /// # Errors
    /// Returns located arity, evaluation, or execution-limit diagnostics.
    pub fn call(&self, arguments: Vec<Value>) -> Result<Value, LanguageError> {
        machine::call(
            Value::Function(self.clone()),
            arguments,
            &mut Budget::default(),
            &self.graph.source,
            None,
        )
    }

    /// Check whether two function values share the very same graph allocation.
    #[must_use]
    pub fn shares_graph(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.graph, &other.graph)
    }

    fn bind(&self, arguments: Vec<Value>, span: Option<Span>) -> Result<Bound, LanguageError> {
        let signature = &self.graph.functions[&self.name];
        let bound = self.environment.append(arguments)?;
        let required = signature.parameters.len() - usize::from(signature.variadic);
        if bound.len() < required {
            return Ok(Bound::Partial(Value::Function(Self {
                graph: Arc::clone(&self.graph),
                name: self.name.clone(),
                environment: bound,
            })));
        }
        if !signature.variadic && bound.len() > required {
            return Err(diagnostic(
                "too many function arguments",
                &self.graph.source,
                span,
            ));
        }
        let mut environment = BTreeMap::new();
        for (position, name) in signature.parameters.iter().take(required).enumerate() {
            environment.insert(name.clone(), bound.get(position)?);
        }
        if signature.variadic {
            environment.insert(
                signature.parameters[required].clone(),
                Value::Pack(
                    (required..bound.len())
                        .map(|position| bound.get(position))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
            );
        }
        Ok(Bound::Ready {
            graph: Arc::clone(&self.graph),
            address: signature.body,
            environment,
            retained: (0..bound.len())
                .map(|position| bound.get(position).map(|value| value.dependencies()))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect(),
        })
    }

    /// Return the Native binding records, including explicit zero presence.
    #[must_use]
    pub fn bindings(&self) -> &State {
        self.environment.native()
    }

    fn native(&self, depth: usize) -> Result<State, LanguageError> {
        if depth >= MAX_CALL_DEPTH {
            return Err(diagnostic(
                "function binding nesting limit reached",
                &self.graph.source,
                None,
            ));
        }
        let root = State::scalar(
            NativeScalar::from_text(&self.graph.functions[&self.name].body.to_string(), "0")
                .map_err(|message| diagnostic(message, &self.graph.source, None))?,
        )
        .index_power(FUNCTION_ROOT, 1)
        .map_err(|message| diagnostic(message, &self.graph.source, None))?;
        Ok(self
            .graph
            .native
            .add(&self.environment.portable(depth + 1)?)
            .add(&root))
    }

    fn load(native: &State, source: &str) -> Result<Self, LanguageError> {
        Self::load_nested(native, source, 0)
    }

    fn load_nested(native: &State, source: &str, depth: usize) -> Result<Self, LanguageError> {
        if depth >= MAX_CALL_DEPTH {
            return Err(diagnostic(
                "function binding nesting limit reached",
                source,
                None,
            ));
        }
        let mut program = Vec::new();
        let mut bindings = Vec::new();
        let mut root = None;
        for (index, coefficient) in &native.project().0 {
            if index.0.contains_key(&HEAD_DIRECTION) {
                program.push((index.clone(), coefficient.clone()));
            } else if index.0.len() == 1 && index.0.get(&FUNCTION_ROOT) == Some(&BigUint::one()) {
                if !coefficient.imag.is_zero() || !coefficient.real.is_integer() {
                    return Err(diagnostic("invalid function entry", source, None));
                }
                root = coefficient.real.to_integer().to_usize();
                if root.is_none() {
                    return Err(diagnostic("invalid function entry", source, None));
                }
            } else {
                bindings.push((index.clone(), coefficient.clone()));
            }
        }
        let graph = Graph::load(
            State::from_projection(&NativeState::from_terms(program))
                .retaining(std::slice::from_ref(native)),
            source,
        )?;
        let name = match root {
            Some(root) => graph
                .functions
                .iter()
                .find_map(|(name, signature)| (signature.body == root).then(|| name.clone()))
                .ok_or_else(|| {
                    diagnostic("function entry does not point to a body", source, None)
                })?,
            None => graph.root.clone(),
        };
        let environment = Environment::load(
            &State::from_projection(&NativeState::from_terms(bindings))
                .retaining(std::slice::from_ref(native)),
            depth + 1,
        )?;
        if !graph.functions.contains_key(&name) {
            return Err(diagnostic("missing function root", source, None));
        }
        Ok(Self {
            graph,
            name,
            environment,
        })
    }
}

impl Value {
    /// Call a function value or validate and call a reflected Native program.
    ///
    /// # Errors
    /// Returns an invalid-program, arity, or execution-limit diagnostic.
    pub fn call(self, arguments: Vec<Self>) -> Result<Self, LanguageError> {
        machine::call(self, arguments, &mut Budget::default(), "<call>", None)
    }
    fn dependencies(&self) -> Vec<State> {
        match self {
            Self::State(state) => vec![state.clone()],
            Self::Function(function) => vec![
                function.graph.native.clone(),
                function.environment.native().clone(),
            ],
            Self::Pack(values) => values.iter().flat_map(Self::dependencies).collect(),
        }
    }
    /// Expose values, functions, and bindings as Native data.
    ///
    /// # Errors
    /// Argument packs must be spread rather than read as individual values.
    pub fn native(&self) -> Result<State, LanguageError> {
        match self {
            Self::State(state) => Ok(state.clone()),
            Self::Function(function) => function.native(0),
            Self::Pack(_) => Err(diagnostic("argument pack must be spread", "<call>", None)),
        }
    }
    /// Return a numeric Native state, diagnosing a still-open function or pack.
    ///
    /// # Errors
    /// Returns a located error when the value is not a state.
    pub fn state(self, source: &str, span: Option<Span>) -> Result<State, LanguageError> {
        self.native()
            .map_err(|error| diagnostic(error.0.message, source, span))
    }
}

// Encoding record data is not evaluating the represented function. The strand
// encoder emits only these literal constructors, so no source call can run here.
fn literal(expression: &Expr) -> Result<State, LanguageError> {
    // These are coordinate-record constructors, not represented instructions.
    // Store their canonical data directly; retaining each encoding prefix adds
    // no program information and makes large reflected graphs quadratic.
    Ok(State::from_projection(&literal_coordinates(expression)?))
}

fn literal_coordinates(expression: &Expr) -> Result<NativeState, LanguageError> {
    Ok(match expression {
        Expr::Literal { real, imag, .. } => NativeState::scalar(
            NativeScalar::from_text(real, imag)
                .map_err(|message| diagnostic(message, "<graph>", None))?,
        ),
        Expr::Add { operands, .. } => {
            let fields = operands
                .iter()
                .map(literal_coordinates)
                .collect::<Result<Vec<_>, _>>()?;
            NativeState::from_terms(fields.into_iter().flat_map(|field| field.0))
        }
        Expr::Multiply { operands, .. } => {
            let mut result = NativeState::one();
            for operand in operands {
                result = result.multiply(&literal_coordinates(operand)?);
            }
            result
        }
        Expr::Phase { turns, value, .. } => literal_coordinates(value)?.phase(*turns),
        Expr::Index {
            direction,
            multiplicity,
            value,
            ..
        } => literal_coordinates(value)?
            .index_power(*direction, *multiplicity)
            .map_err(|message| diagnostic(message, "<graph>", None))?,
        _ => return Err(diagnostic("nonliteral record encoding", "<graph>", None)),
    })
}

// Bare pack syntax builds ordinary retained operations; SPREAD keeps the bindings.
// Do not project the items: zero provenance and callable function data must survive.
fn indexed_pack(items: &[Value], source: &str, span: Option<Span>) -> Result<State, LanguageError> {
    let mut result = State::zero();
    for (position, item) in items.iter().enumerate() {
        let direction = crate::core::pack_direction(position, source, span)?;
        let item = item
            .native()?
            .index_power(direction, 1)
            .map_err(|message| diagnostic(message, source, span))?;
        result = result.add(&item);
    }
    Ok(result)
}
