// SPDX-License-Identifier: AGPL-3.0-or-later

//! Heap continuations keep Native recursion independent of the host stack.
use super::{
    Arc, BTreeMap, Bound, Budget, CALL, FunctionValue, Graph, LanguageError, MAX_CALL_DEPTH,
    SPREAD, Span, State, Value, diagnostic,
};

enum Task {
    Evaluate(Arc<Graph>, usize, Arc<BTreeMap<String, Value>>),
    Finish(Arc<Graph>, usize, Arc<BTreeMap<String, Value>>, Vec<usize>),
    Call(Value, Vec<Value>, String, Option<Span>),
    Return(Vec<State>, Option<Span>),
}

pub(super) fn evaluate(
    graph: Arc<Graph>,
    address: usize,
    environment: BTreeMap<String, Value>,
    budget: &mut Budget,
) -> Result<Value, LanguageError> {
    run(
        vec![Task::Evaluate(graph, address, Arc::new(environment))],
        budget,
    )
}

pub(super) fn call(
    value: Value,
    arguments: Vec<Value>,
    budget: &mut Budget,
    source: &str,
    span: Option<Span>,
) -> Result<Value, LanguageError> {
    run(
        vec![Task::Call(value, arguments, source.into(), span)],
        budget,
    )
}

fn run(mut tasks: Vec<Task>, budget: &mut Budget) -> Result<Value, LanguageError> {
    let mut values = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            Task::Evaluate(graph, address, environment) => {
                let record = &graph.records[address];
                if budget.remaining == 0 {
                    return Err(diagnostic(
                        "execution step limit reached",
                        &graph.source,
                        record.span,
                    ));
                }
                budget.remaining -= 1;
                let mut children = graph.children[address].clone();
                if graph.rules.contains_key(&address) {
                    children.truncate(1);
                }
                tasks.push(Task::Finish(
                    Arc::clone(&graph),
                    address,
                    Arc::clone(&environment),
                    children.clone(),
                ));
                for child in children.into_iter().rev() {
                    tasks.push(Task::Evaluate(
                        Arc::clone(&graph),
                        child,
                        Arc::clone(&environment),
                    ));
                }
            }
            Task::Finish(graph, address, environment, children) => finish(
                &graph,
                address,
                &environment,
                children,
                &mut values,
                &mut tasks,
            )?,
            Task::Call(callee, arguments, source, span) => {
                let function = callable(callee, &source, span, budget)?;
                match function.bind(arguments, span)? {
                    Bound::Partial(value) => values.push(value),
                    Bound::Ready {
                        graph,
                        address,
                        environment,
                        retained,
                    } => {
                        if budget.depth >= MAX_CALL_DEPTH {
                            return Err(diagnostic(
                                "execution call-depth limit reached",
                                &source,
                                span,
                            ));
                        }
                        budget.depth += 1;
                        tasks.push(Task::Return(retained, span.or(graph.records[address].span)));
                        tasks.push(Task::Evaluate(graph, address, Arc::new(environment)));
                    }
                }
            }
            Task::Return(retained, span) => {
                budget.depth -= 1;
                let value = values.pop().expect("function evaluation returns one value");
                values.push(match value {
                    Value::State(state) => Value::State(state.retaining(&retained).at_span(span)),
                    other => other,
                });
            }
        }
    }
    Ok(values.pop().expect("execution produces one result"))
}

fn finish(
    graph: &Arc<Graph>,
    address: usize,
    environment: &BTreeMap<String, Value>,
    children: Vec<usize>,
    values: &mut Vec<Value>,
    tasks: &mut Vec<Task>,
) -> Result<(), LanguageError> {
    let results = values.split_off(values.len() - children.len());
    let mut arguments = Vec::new();
    for (child, result) in children.into_iter().zip(results) {
        match result {
            Value::Pack(pack) if graph.records[child].kind == SPREAD => {
                arguments.extend(pack);
            }
            value => arguments.push(value),
        }
    }
    let record = &graph.records[address];
    match record.kind {
        CALL => {
            let callee = arguments.remove(0);
            tasks.push(Task::Call(
                callee,
                arguments,
                graph.source.clone(),
                record.span,
            ));
        }
        _ => values.push(graph.reduce(address, environment, &arguments)?),
    }

    Ok(())
}

fn callable(
    callee: Value,
    source: &str,
    span: Option<Span>,
    budget: &mut Budget,
) -> Result<FunctionValue, LanguageError> {
    match callee {
        Value::Function(function) => Ok(function),
        Value::State(state) => {
            if let Some(function) = budget.loaded.get(&state.identity()) {
                return Ok(function.clone());
            }
            let function = FunctionValue::load(&state, source).map_err(|mut error| {
                if error.0.span.is_none() {
                    error.0.span = span;
                }
                error
            })?;
            budget.loaded.insert(state.identity(), function.clone());
            Ok(function)
        }
        Value::Pack(_) => Err(diagnostic("an argument pack is not callable", source, span)),
    }
}
