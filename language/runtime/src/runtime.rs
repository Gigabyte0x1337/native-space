// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared graph execution. Coordinates, retained inputs, and index metadata are distinct.
use crate::{
    algebra::{Rational, Split, State, Transform, rational},
    syntax::{BUILTINS, Expr, Graph, Kind},
};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

pub const MAX_STEPS: u64 = 1_000_000;
const MAX_CALL_DEPTH: usize = 128;
#[derive(Clone, Debug)]
pub struct Scalar(pub Arc<History>);
#[derive(Debug)]
pub struct History {
    pub state: State,
    pub operation: String,
    pub inputs: Vec<Scalar>,
}
impl Drop for History {
    fn drop(&mut self) {
        let mut pending = std::mem::take(&mut self.inputs);
        while let Some(Scalar(node)) = pending.pop() {
            if let Ok(mut unique) = Arc::try_unwrap(node) {
                pending.append(&mut unique.inputs);
            }
        }
    }
}
impl Scalar {
    pub fn literal(state: State) -> Self {
        Self::result(state, "state", Vec::new())
    }
    pub fn result(state: State, op: &str, inputs: Vec<Scalar>) -> Self {
        Self(Arc::new(History {
            state,
            operation: op.into(),
            inputs,
        }))
    }
    pub fn state(&self) -> &State {
        &self.0.state
    }
    /// Topological history records; node addresses are metadata, never geometric axes.
    pub fn records(&self) -> Result<Vec<HistoryRecord>, String> {
        let mut stack = vec![(self.clone(), false)];
        let mut seen = BTreeMap::new();
        let mut records = Vec::new();
        while let Some((node, visited)) = stack.pop() {
            let key = Arc::as_ptr(&node.0) as usize;
            if seen.contains_key(&key) {
                continue;
            }
            if records.len() + stack.len() > 100_000 {
                return Err("retained history exceeds export budget".into());
            }
            if !visited {
                stack.push((node.clone(), true));
                for input in node.0.inputs.iter().rev() {
                    stack.push((input.clone(), false));
                }
            } else {
                let inputs = node
                    .0
                    .inputs
                    .iter()
                    .map(|s| seen[&(Arc::as_ptr(&s.0) as usize)])
                    .collect();
                seen.insert(key, records.len());
                records.push(HistoryRecord {
                    state: node.0.state.clone(),
                    operation: node.0.operation.clone(),
                    inputs,
                });
            }
        }
        Ok(records)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryRecord {
    pub state: State,
    pub operation: String,
    pub inputs: Vec<usize>,
}
impl Serialize for Scalar {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.records()
            .map_err(serde::ser::Error::custom)?
            .serialize(s)
    }
}
impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let records = Vec::<HistoryRecord>::deserialize(d)?;
        if records.is_empty() || records.len() > 100_000 {
            return Err(serde::de::Error::custom("invalid history size"));
        }
        let mut nodes: Vec<Scalar> = Vec::new();
        for r in records {
            if r.inputs.iter().any(|i| *i >= nodes.len()) {
                return Err(serde::de::Error::custom("invalid history edge"));
            }
            let inputs = r.inputs.iter().map(|i| nodes[*i].clone()).collect();
            nodes.push(Self::result(r.state, &r.operation, inputs));
        }
        Ok(nodes.pop().expect("nonempty"))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Value {
    State(Scalar),
    Text(String),
    List(Vec<Value>),
    Record(BTreeMap<String, Value>),
    Function(Function),
    Program(Arc<Program>),
    Observation(Observation),
    Transform(Transform),
    Framed { local: Scalar, frame: Transform },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Function {
    pub graph: Arc<Graph>,
    pub name: String,
    pub bindings: Vec<Value>,
    pub scope: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub seed: Value,
    pub step: Function,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub program: Arc<Program>,
    pub index: BigUint,
}
#[derive(Debug)]
pub struct Session {
    pub graph: Arc<Graph>,
    pub output: Value,
    pub scope: BTreeMap<String, Value>,
}
struct Budget {
    left: u64,
}
impl Budget {
    fn step(&mut self) -> Result<(), String> {
        if self.left == 0 {
            return Err("execution budget exhausted".into());
        }
        self.left -= 1;
        Ok(())
    }
}
impl Value {
    pub fn number(n: Rational) -> Self {
        Self::State(Scalar::literal(State::number(n)))
    }
    pub fn state(s: State) -> Self {
        Self::State(Scalar::literal(s))
    }
    pub fn scalar(&self) -> Result<Scalar, String> {
        match self {
            Self::State(s) => Ok(s.clone()),
            Self::Framed { local, frame } => Ok(Scalar::result(
                frame.decode(local.state()),
                "decode",
                vec![local.clone()],
            )),
            Self::Observation(o) => o.evaluate(MAX_STEPS)?.scalar(),
            _ => Err("expected a scalar Native state".into()),
        }
    }
    pub fn number_readout(&self) -> Result<Rational, String> {
        Ok(self.scalar()?.state().decode()?.1)
    }
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
    pub fn from_json(s: &str) -> Result<Self, String> {
        if s.len() > 16_000_000 {
            return Err("native document exceeds 16 MB".into());
        }
        let value: Self = serde_json::from_str(s).map_err(|e| e.to_string())?;
        value.validate(0)?;
        Ok(value)
    }
    fn validate(&self, depth: usize) -> Result<(), String> {
        if depth > 128 {
            return Err("value nesting exceeds 128".into());
        }
        match self {
            Self::Function(f) => {
                crate::syntax::validate(&f.graph)?;
                let def = f
                    .graph
                    .functions
                    .get(&f.name)
                    .ok_or("function is absent from graph")?;
                if f.bindings.len() > def.parameters.len() {
                    return Err("excess function bindings".into());
                }
                for v in f.bindings.iter().chain(f.scope.values()) {
                    v.validate(depth + 1)?;
                }
            }
            Self::Program(p) => {
                p.seed.validate(depth + 1)?;
                Self::Function(p.step.clone()).validate(depth + 1)?;
                p.validate()?;
            }
            Self::Observation(o) => Self::Program(Arc::clone(&o.program)).validate(depth + 1)?,
            Self::List(xs) => {
                for x in xs {
                    x.validate(depth + 1)?;
                }
            }
            Self::Record(xs) => {
                for x in xs.values() {
                    x.validate(depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    /// Field reflection exposes values, including explicit binding presence and exact index.
    fn fields(&self) -> Result<BTreeMap<String, Value>, String> {
        let num = |n: Rational| Value::number(n);
        Ok(match self {
            Self::Record(r) => r.clone(),
            Self::State(s) => BTreeMap::from(
                [
                    ("l", &s.state().l),
                    ("a", &s.state().a),
                    ("m", &s.state().m),
                ]
                .map(|(name, n)| {
                    (
                        name.into(),
                        Value::State(Scalar::result(
                            State::number(n.clone()),
                            name,
                            vec![s.clone()],
                        )),
                    )
                }),
            ),
            Self::Program(p) => BTreeMap::from([
                ("seed".into(), p.seed.clone()),
                ("step".into(), Self::Function(p.step.clone())),
            ]),
            Self::Observation(o) => BTreeMap::from([
                ("program".into(), Self::Program(Arc::clone(&o.program))),
                (
                    "index".into(),
                    num(Rational::from_integer(o.index.clone().into())),
                ),
            ]),
            Self::Function(f) => BTreeMap::from([
                (
                    "graph".into(),
                    json_value(&serde_json::to_value(&f.graph).map_err(|e| e.to_string())?)?,
                ),
                ("name".into(), Self::Text(f.name.clone())),
                ("bindings".into(), Self::List(f.bindings.clone())),
                ("scope".into(), Self::Record(f.scope.clone())),
            ]),
            Self::Framed { local, frame } => BTreeMap::from([
                ("local".into(), Self::State(local.clone())),
                ("frame".into(), Self::Transform(frame.clone())),
            ]),
            _ => return Err("value has no structural fields".into()),
        })
    }
}
impl Session {
    pub fn new(source: &str, name: &str) -> Result<Self, String> {
        let graph = Arc::new(crate::syntax::parse(source, name)?);
        let mut budget = Budget { left: MAX_STEPS };
        let mut scope = BTreeMap::new();
        for (name, e) in &graph.bindings {
            let v = eval(&graph, e, &scope, &mut budget, 0)?;
            scope.insert(name.clone(), v);
        }
        let output = eval(&graph, &graph.output, &scope, &mut budget, 0)?;
        Ok(Self {
            graph,
            output,
            scope,
        })
    }
    pub fn function(&self, name: &str) -> Result<Function, String> {
        if !self.graph.functions.contains_key(name) {
            return Err(format!("unknown function {name}"));
        }
        Ok(Function {
            graph: Arc::clone(&self.graph),
            name: name.into(),
            bindings: Vec::new(),
            scope: self.scope.clone(),
        })
    }
}
impl Function {
    pub fn remaining(&self) -> usize {
        self.graph.functions[&self.name].parameters.len() - self.bindings.len()
    }
    pub fn call(&self, args: Vec<Value>) -> Result<Value, String> {
        call(self, args, &mut Budget { left: MAX_STEPS }, 0)
    }
}
impl Program {
    pub fn new(seed: Value, step: Function) -> Result<Arc<Self>, String> {
        let p = Self { seed, step };
        p.validate()?;
        Ok(Arc::new(p))
    }
    fn validate(&self) -> Result<(), String> {
        if self.step.remaining() != 1 {
            return Err("program step needs exactly one unbound parameter".into());
        }
        Ok(())
    }
    pub fn observe(self: &Arc<Self>, index: BigUint) -> Observation {
        Observation {
            program: Arc::clone(self),
            index,
        }
    }
}
impl Observation {
    pub fn successor(&self) -> Self {
        Self {
            program: Arc::clone(&self.program),
            index: &self.index + BigUint::from(1u8),
        }
    }
    pub fn evaluate(&self, maximum: u64) -> Result<Value, String> {
        let count = self
            .index
            .to_u64()
            .filter(|k| *k <= maximum)
            .ok_or("observation exceeds repetition budget")?;
        let mut value = self.program.seed.clone();
        let mut budget = Budget { left: MAX_STEPS };
        for _ in 0..count {
            value = call(&self.program.step, vec![value], &mut budget, 0)?;
        }
        Ok(value)
    }
}

type Scope = Arc<BTreeMap<String, Value>>;
enum Task {
    Eval(Arc<Graph>, Arc<Expr>, Scope, usize),
    Finish(Arc<Graph>, Arc<Expr>, Scope, usize, usize),
    Reflect(Arc<Graph>, Arc<Expr>, Arc<Expr>, Scope, usize),
    Call(Function, Vec<Value>, usize),
    Return(Vec<Scalar>),
}
fn call(
    f: &Function,
    args: Vec<Value>,
    budget: &mut Budget,
    depth: usize,
) -> Result<Value, String> {
    machine(vec![Task::Call(f.clone(), args, depth)], budget)
}
fn eval(
    g: &Arc<Graph>,
    e: &Arc<Expr>,
    scope: &BTreeMap<String, Value>,
    budget: &mut Budget,
    depth: usize,
) -> Result<Value, String> {
    machine(
        vec![Task::Eval(
            Arc::clone(g),
            Arc::clone(e),
            Arc::new(scope.clone()),
            depth,
        )],
        budget,
    )
}
fn machine(mut tasks: Vec<Task>, budget: &mut Budget) -> Result<Value, String> {
    let mut values = Vec::new();
    while let Some(task) = tasks.pop() {
        budget.step()?;
        match task {
            Task::Return(mut inputs) => {
                let value = values.pop().expect("function result");
                values.push(match value {
                    Value::State(s) => {
                        inputs.insert(0, s.clone());
                        Value::State(Scalar::result(s.state().clone(), "call", inputs))
                    }
                    other => other,
                });
            }
            Task::Call(f, args, depth) => {
                if depth >= MAX_CALL_DEPTH {
                    return Err(format!("call depth exceeds 128 in {}", f.name));
                }
                let mut bindings = f.bindings.clone();
                bindings.extend(args);
                let def = &f.graph.functions[&f.name];
                if bindings.len() > def.parameters.len() {
                    return Err(format!("too many arguments to {}", f.name));
                }
                if bindings.len() < def.parameters.len() {
                    values.push(Value::Function(Function { bindings, ..f }));
                    continue;
                }
                let retained = bindings.iter().flat_map(retained_scalars).collect();
                let mut scope = f.scope.clone();
                scope.extend(def.parameters.iter().cloned().zip(bindings));
                tasks.push(Task::Return(retained));
                tasks.push(Task::Eval(
                    Arc::clone(&f.graph),
                    Arc::clone(&def.body),
                    Arc::new(scope),
                    depth + 1,
                ));
            }
            Task::Eval(g, e, scope, depth) => {
                let at = |message: String| {
                    format!("{}:{}:{}: {message}", g.source_name, e.line, e.column)
                };
                match &e.kind {
                    Kind::Number(n) => values.push(Value::number(rational(n).map_err(at)?)),
                    Kind::Text(t) => values.push(Value::Text(t.clone())),
                    Kind::Name(n) => {
                        let v = if let Some(v) = scope.get(n) {
                            v.clone()
                        } else if g.functions.contains_key(n) {
                            Value::Function(Function {
                                graph: Arc::clone(&g),
                                name: n.clone(),
                                bindings: Vec::new(),
                                scope: (*scope).clone(),
                            })
                        } else {
                            return Err(at(format!("unknown name {n}")));
                        };
                        values.push(v);
                    }
                    Kind::Call(callee, args) if matches!(&callee.kind,Kind::Name(n)if n=="reflect") =>
                    {
                        if args.len() != 3 {
                            return Err(
                                at("reflect requires subject, pattern, replacement".into()),
                            );
                        }
                        tasks.push(Task::Reflect(
                            Arc::clone(&g),
                            Arc::clone(&args[1]),
                            Arc::clone(&args[2]),
                            Arc::clone(&scope),
                            depth,
                        ));
                        tasks.push(Task::Eval(g, Arc::clone(&args[0]), scope, depth));
                    }
                    _ => {
                        let mut children: Vec<Arc<Expr>> = match &e.kind {
                            Kind::List(xs) => xs.clone(),
                            Kind::Record(xs) => xs.values().cloned().collect(),
                            Kind::Call(f, xs) => {
                                let mut c = Vec::new();
                                if !matches!(&f.kind,Kind::Name(n)if BUILTINS.contains(&n.as_str()))
                                {
                                    c.push(Arc::clone(f));
                                }
                                c.extend(xs.iter().cloned());
                                c
                            }
                            _ => unreachable!(),
                        };
                        tasks.push(Task::Finish(
                            Arc::clone(&g),
                            Arc::clone(&e),
                            Arc::clone(&scope),
                            depth,
                            children.len(),
                        ));
                        while let Some(child) = children.pop() {
                            tasks.push(Task::Eval(
                                Arc::clone(&g),
                                child,
                                Arc::clone(&scope),
                                depth,
                            ));
                        }
                    }
                }
            }
            Task::Reflect(g, pattern, replacement, scope, depth) => {
                let subject = values.pop().expect("evaluated subject");
                let mut bindings = (*scope).clone();
                if matches(&subject, &pattern, &mut bindings)? {
                    tasks.push(Task::Eval(g, replacement, Arc::new(bindings), depth));
                } else {
                    values.push(Value::List(Vec::new()));
                }
            }
            Task::Finish(g, e, _scope, depth, count) => {
                let mut args = values.split_off(values.len() - count);
                match &e.kind {
                    Kind::List(_) => values.push(Value::List(args)),
                    Kind::Record(fields) => {
                        values.push(Value::Record(fields.keys().cloned().zip(args).collect()))
                    }
                    Kind::Call(f, _) => {
                        if let Kind::Name(n) = &f.kind {
                            if BUILTINS.contains(&n.as_str()) {
                                let value = builtin(n, args).map_err(|err| {
                                    format!("{}:{}:{}: {err}", g.source_name, e.line, e.column)
                                })?;
                                values.push(value);
                                continue;
                            }
                        }
                        let target = args.remove(0);
                        let f = match target {
                            Value::Function(f) => f,
                            Value::Record(r) => function_from_fields(r)?,
                            _ => {
                                return Err(format!(
                                    "{}:{}:{}: expected a function graph",
                                    g.source_name, e.line, e.column
                                ));
                            }
                        };
                        tasks.push(Task::Call(f, args, depth));
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    values.pop().ok_or("execution produced no value".into())
}
fn expect_len(values: &[Value], n: usize) -> Result<(), String> {
    if values.len() != n {
        Err(format!("expected {n} arguments"))
    } else {
        Ok(())
    }
}
fn builtin(name: &str, v: Vec<Value>) -> Result<Value, String> {
    let s = |i: usize| v[i].scalar();
    let n = |i: usize| v[i].number_readout();
    match name {
        "state" => {
            expect_len(&v, 3)?;
            let inputs = v.iter().map(Value::scalar).collect::<Result<Vec<_>, _>>()?;
            Ok(Value::State(Scalar::result(
                State::new(n(0)?, n(1)?, n(2)?),
                "state",
                inputs,
            )))
        }
        "add" | "multiply" => {
            let mut state = if name == "add" {
                State::zero()
            } else {
                State::one()
            };
            let inputs = v.iter().map(Value::scalar).collect::<Result<Vec<_>, _>>()?;
            for p in &inputs {
                state = if name == "add" {
                    state.add(p.state())
                } else {
                    state.multiply(p.state())
                };
            }
            let out = Scalar::result(state, name, inputs);
            // Binary/unary arithmetic preserves the first explicit computational frame.
            if let Some(Value::Framed { frame, .. }) =
                v.iter().find(|x| matches!(x, Value::Framed { .. }))
            {
                return Ok(Value::Framed {
                    local: Scalar::result(frame.encode(out.state()), "encode", vec![out]),
                    frame: frame.clone(),
                });
            }
            Ok(Value::State(out))
        }
        "negate" | "inverse" => {
            expect_len(&v, 1)?;
            let p = s(0)?;
            let q = if name == "negate" {
                p.state().negate()
            } else {
                p.state().inverse()?
            };
            in_frame(Scalar::result(q, name, vec![p]), &v)
        }
        "split" => {
            expect_len(&v, 2)?;
            let route = match &v[1] {
                Value::Text(t) if t == "add" => Split::Add,
                Value::Text(t) if t == "multiply" => Split::Multiply,
                _ => return Err("split route is \"add\" or \"multiply\"".into()),
            };
            let p = s(0)?;
            in_frame(Scalar::result(p.state().split(route), "split", vec![p]), &v)
        }
        "transform" if v.len() == 1 => {
            let Value::List(rows) = &v[0] else {
                return Err("transform expects a 3x3 rational matrix".into());
            };
            if rows.len() != 3 {
                return Err("matrix needs three rows".into());
            }
            let mut matrix = Vec::new();
            for row in rows {
                let Value::List(cols) = row else {
                    return Err("matrix row must be a list".into());
                };
                if cols.len() != 3 {
                    return Err("matrix needs three columns".into());
                }
                matrix.push(
                    cols.iter()
                        .map(Value::number_readout)
                        .collect::<Result<Vec<_>, _>>()?,
                );
            }
            Ok(Value::Transform(Transform::new(std::array::from_fn(
                |i| std::array::from_fn(|j| matrix[i][j].clone()),
            ))?))
        }
        "transform" | "mutate" => {
            expect_len(&v, 2)?;
            let Value::Transform(t) = &v[1] else {
                return Err("expected a transform".into());
            };
            let p = s(0)?;
            let q = Scalar::result(t.encode(p.state()), name, vec![p]);
            if name == "mutate" {
                Ok(Value::State(q))
            } else {
                Ok(Value::Framed {
                    local: q,
                    frame: t.clone(),
                })
            }
        }
        "decode" => {
            expect_len(&v, 1)?;
            Ok(Value::State(s(0)?))
        }
        "program" => {
            expect_len(&v, 2)?;
            let Value::Function(f) = &v[1] else {
                return Err("program requires a callable step".into());
            };
            Ok(Value::Program(Program::new(v[0].clone(), f.clone())?))
        }
        "observe" => {
            expect_len(&v, 2)?;
            let Value::Program(p) = &v[0] else {
                return Err("observe requires a program".into());
            };
            let k = n(1)?;
            if !k.is_integer() {
                return Err("observation index must be an exact nonnegative integer".into());
            }
            let k = k
                .to_integer()
                .to_biguint()
                .ok_or("negative observation index")?;
            Ok(Value::Observation(p.observe(k)))
        }
        _ => Err(format!("invalid call to {name}")),
    }
}
fn matches(
    value: &Value,
    pattern: &Expr,
    bindings: &mut BTreeMap<String, Value>,
) -> Result<bool, String> {
    match &pattern.kind {
        Kind::Name(name) => {
            if name == "_" {
                return Ok(true);
            }
            if let Some(old) = bindings.get(name) {
                return Ok(old.to_json()? == value.to_json()?);
            }
            bindings.insert(name.clone(), value.clone());
            Ok(true)
        }
        Kind::Number(n) => Ok(value.number_readout().ok() == Some(rational(n)?)),
        Kind::Text(t) => Ok(matches!(value,Value::Text(x)if x==t)),
        Kind::List(ps) => {
            let Value::List(vs) = value else {
                return Ok(false);
            };
            if ps.len() != vs.len() {
                return Ok(false);
            }
            for (v, p) in vs.iter().zip(ps) {
                if !matches(v, p, bindings)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Kind::Record(ps) => {
            let Ok(fields) = value.fields() else {
                return Ok(false);
            };
            for (n, p) in ps {
                let Some(v) = fields.get(n) else {
                    return Ok(false);
                };
                if !matches(v, p, bindings)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Kind::Call(f, ps) => {
            let Kind::Name(name) = &f.kind else {
                return Err("reflect constructor must have a name".into());
            };
            let fields = value.fields()?;
            let names: &[&str] = match name.as_str() {
                "state" => &["l", "a", "m"],
                "program" => &["seed", "step"],
                "observe" => &["program", "index"],
                _ => return Err("use state, program, observe, or record patterns".into()),
            };
            if ps.len() != names.len() {
                return Err("wrong reflection constructor arity".into());
            }
            for (n, p) in names.iter().zip(ps) {
                let Some(v) = fields.get(*n) else {
                    return Ok(false);
                };
                if !matches(v, p, bindings)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
    }
}
fn json_value(j: &serde_json::Value) -> Result<Value, String> {
    match j {
        serde_json::Value::String(s) => Ok(Value::Text(s.clone())),
        serde_json::Value::Number(n) => Ok(Value::number(rational(&n.to_string())?)),
        serde_json::Value::Array(xs) => Ok(Value::List(
            xs.iter().map(json_value).collect::<Result<_, _>>()?,
        )),
        serde_json::Value::Object(xs) => Ok(Value::Record(
            xs.iter()
                .map(|(k, v)| Ok((k.clone(), json_value(v)?)))
                .collect::<Result<_, String>>()?,
        )),
        _ => Err("unsupported graph data".into()),
    }
}
fn value_json(v: &Value) -> Result<serde_json::Value, String> {
    match v {
        Value::Text(s) => Ok(serde_json::Value::String(s.clone())),
        Value::State(s) => {
            let n = s.state().decode()?.1;
            if !n.is_integer() {
                return Err("graph number must be an integer".into());
            }
            serde_json::from_str(&n.to_integer().to_string()).map_err(|e| e.to_string())
        }
        Value::List(xs) => Ok(serde_json::Value::Array(
            xs.iter().map(value_json).collect::<Result<_, _>>()?,
        )),
        Value::Record(xs) => Ok(serde_json::Value::Object(
            xs.iter()
                .map(|(k, v)| Ok((k.clone(), value_json(v)?)))
                .collect::<Result<_, String>>()?,
        )),
        _ => Err("expected structural graph fields".into()),
    }
}
fn function_from_fields(mut r: BTreeMap<String, Value>) -> Result<Function, String> {
    let graph = r.remove("graph").ok_or("missing graph")?;
    let graph: Graph = serde_json::from_value(value_json(&graph)?).map_err(|e| e.to_string())?;
    crate::syntax::validate(&graph)?;
    let Some(Value::Text(name)) = r.remove("name") else {
        return Err("missing function name".into());
    };
    let Some(Value::List(bindings)) = r.remove("bindings") else {
        return Err("missing bindings".into());
    };
    let Some(Value::Record(scope)) = r.remove("scope") else {
        return Err("missing scope".into());
    };
    if !r.is_empty() {
        return Err("unknown function fields".into());
    }
    let value = Value::Function(Function {
        graph: Arc::new(graph),
        name,
        bindings,
        scope,
    });
    value.validate(0)?;
    let Value::Function(f) = value else {
        unreachable!()
    };
    Ok(f)
}
fn retained_scalars(v: &Value) -> Vec<Scalar> {
    let mut pending = vec![v];
    let mut result = Vec::new();
    while let Some(v) = pending.pop() {
        match v {
            Value::State(s) => result.push(s.clone()),
            Value::Framed { local, .. } => result.push(local.clone()),
            Value::List(xs) => pending.extend(xs),
            Value::Record(xs) => pending.extend(xs.values()),
            _ => {}
        }
    }
    result
}
fn in_frame(out: Scalar, values: &[Value]) -> Result<Value, String> {
    if let Some(Value::Framed { frame, .. }) =
        values.iter().find(|v| matches!(v, Value::Framed { .. }))
    {
        let state = frame.encode(out.state());
        Ok(Value::Framed {
            local: Scalar::result(state, "encode", vec![out]),
            frame: frame.clone(),
        })
    } else {
        Ok(Value::State(out))
    }
}
