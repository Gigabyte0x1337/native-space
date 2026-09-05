// SPDX-License-Identifier: AGPL-3.0-or-later

//! Validates, rewrites, and applies native operation strands.
//!
//! Rules are source functions, not Rust optimization rules. Rewriting is one
//! ordered bottom-up pass and makes no equivalence or performance claim.
//! See language/REFLECTION.md for the finite execution and matching contract.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::core::{
    self, Diagnostic, Expr, Function, Goal, LanguageError, NativeState, OutputKind, Program, Span,
};
use crate::strand::{self, DecodedStrand};

/// A staged operation on an explicit program representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// Apply an explicit source-defined structural rewrite rule.
    Rewrite,
    /// Execute a validated graph with supplied arguments.
    Apply,
}

impl Operation {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Rewrite => "rewrite",
            Self::Apply => "apply",
        }
    }

    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "rewrite" => Some(Self::Rewrite),
            "apply" => Some(Self::Apply),
            _ => None,
        }
    }

    pub(crate) const fn accepts(self, count: usize) -> bool {
        match self {
            Self::Rewrite => count == 3,
            Self::Apply => count >= 1,
        }
    }
}

// Finite guardrails bound matching work and native recursion stack usage.
// They are implementation limits, not claims about mathematical patterns.
const MAX_NODES: usize = 20_000;
const MAX_DEPTH: usize = 128;
const MAX_MATCH_WORK: usize = 100_000;

fn error(message: impl Into<String>, source: &str, span: Option<Span>) -> LanguageError {
    LanguageError(Diagnostic {
        code: "NSR001".into(),
        message: message.into(),
        source_name: source.into(),
        span,
    })
}

fn program(graph: &DecodedStrand, source: &str, span: Option<Span>) -> Program {
    Program {
        functions: graph.functions.clone(),
        bindings: Vec::new(),
        goal: Goal::Emit,
        output_kind: OutputKind::Pattern,
        result: Expr::Trace {
            function: graph.root.clone(),
            span,
        },
        source_name: source.into(),
        span,
    }
}

fn decode(
    state: &NativeState,
    source: &str,
    span: Option<Span>,
) -> Result<DecodedStrand, LanguageError> {
    if strand::operation_length(state, source, span)? > MAX_NODES as u64 {
        return Err(error(
            "reflection graph exceeds the coordinate limit",
            source,
            span,
        ));
    }
    let graph = strand::decode_operation_strand(state, source, span)?;
    check_size(&graph.functions, source, span)?;
    core::validate(&program(&graph, source, span))?;
    Ok(graph)
}

fn check_size(
    functions: &[Function],
    source: &str,
    span: Option<Span>,
) -> Result<(), LanguageError> {
    // Function-catalog traversal is recursive in the existing trace encoder.
    // Bound it as well as expression depth before calling that encoder.
    if functions.len() > MAX_DEPTH {
        return Err(error(
            "reflection graph exceeds the function catalog limit",
            source,
            span,
        ));
    }
    let mut pending = functions.iter().map(|f| (&f.body, 0)).collect::<Vec<_>>();
    let mut count = 0;
    while let Some((expr, depth)) = pending.pop() {
        count += 1;
        if count > MAX_NODES || depth > MAX_DEPTH {
            return Err(error(
                "reflection graph exceeds the node or nesting limit",
                source,
                span,
            ));
        }
        pending.extend(children(expr).into_iter().map(|child| (child, depth + 1)));
    }
    Ok(())
}

fn root(graph: &DecodedStrand) -> &Function {
    graph
        .functions
        .iter()
        .find(|f| f.name == graph.root)
        .expect("decoded and validated strands contain their root")
}

/// Reconstruct an executable graph without changing its parameters.
///
/// Nested graph application is rejected so it cannot reset cycle detection.
pub(crate) fn application_graph(
    state: &NativeState,
    source: &str,
    span: Option<Span>,
) -> Result<(Vec<Function>, String), LanguageError> {
    let graph = decode(state, source, span)?;
    let mut pending = graph.functions.iter().map(|f| &f.body).collect::<Vec<_>>();
    while let Some(expr) = pending.pop() {
        if matches!(
            expr,
            Expr::Reflect {
                operation: Operation::Apply,
                ..
            }
        ) {
            return Err(error(
                "applied graphs cannot contain nested graph application",
                source,
                span,
            ));
        }
        pending.extend(children(expr));
    }
    // A call root, rather than a trace root, activates the existing cycle check.
    let mut checked = program(&graph, source, span);
    let definition = root(&graph);
    checked.result = Expr::Call {
        function: graph.root.clone(),
        arguments: definition
            .parameters
            .iter()
            .map(|_| Expr::Zero { span })
            .collect(),
        span,
    };
    core::validate(&checked)?;
    Ok((graph.functions, graph.root))
}

pub(crate) fn evaluate(
    operation: Operation,
    arguments: &[NativeState],
    source: &str,
    span: Option<Span>,
) -> Result<NativeState, LanguageError> {
    if !operation.accepts(arguments.len()) {
        return Err(error("invalid reflection argument count", source, span));
    }
    match operation {
        Operation::Rewrite => {
            let expr = rewrite(arguments, source, span)?;
            core::interpret(&Program {
                functions: Vec::new(),
                bindings: Vec::new(),
                result: expr,
                goal: Goal::Emit,
                output_kind: OutputKind::Pattern,
                source_name: source.into(),
                span,
            })
        }
        Operation::Apply => {
            let (functions, root) = application_graph(&arguments[0], source, span)?;
            let graph = Program {
                functions,
                bindings: Vec::new(),
                result: Expr::Trace {
                    function: root.clone(),
                    span,
                },
                goal: Goal::Emit,
                output_kind: OutputKind::Pattern,
                source_name: source.into(),
                span,
            };
            core::exact_function(&graph, &root)?.apply(&arguments[1..])
        }
    }
}

pub(crate) fn rewrite(
    arguments: &[NativeState],
    source: &str,
    span: Option<Span>,
) -> Result<Expr, LanguageError> {
    let [target, pattern, replacement] = arguments else {
        return Err(error(
            "rewrite expects graph, pattern, and replacement",
            source,
            span,
        ));
    };
    let mut target = decode(target, source, span)?;
    let pattern = decode(pattern, source, span)?;
    let replacement = decode(replacement, source, span)?;
    let from = root(&pattern);
    let to = root(&replacement);
    let parameters = rule_parameters(from, to, source, span)?;
    // Callee names remain references. Never silently reinterpret the same
    // spelling as a different source function when combining graph catalogs.
    let mut helpers = Vec::new();
    for rule in [&pattern, &replacement] {
        for function in &rule.functions {
            if function.name == rule.root {
                continue;
            }
            if function.name == pattern.root || function.name == replacement.root {
                return Err(error(
                    "rewrite rule roots cannot be recursive helpers",
                    source,
                    span,
                ));
            }
            if let Some(existing) = target
                .functions
                .iter()
                .chain(helpers.iter())
                .find(|f| f.name == function.name)
            {
                if !same_function(existing, function) {
                    return Err(error(
                        format!("conflicting reflection helper {:?}", function.name),
                        source,
                        span,
                    ));
                }
            } else {
                helpers.push(function.clone());
            }
        }
    }
    // Rule bodies that call their own root would need a separate binding
    // scope. Reject that case rather than inserting a captured function name.
    for rule in [&pattern, &replacement] {
        let mut pending = rule.functions.iter().map(|f| &f.body).collect::<Vec<_>>();
        while let Some(expr) = pending.pop() {
            if matches!(expr, Expr::Call { function, .. } | Expr::Trace { function, .. } |
                Expr::Fold { function, .. } if function == &rule.root)
            {
                return Err(error(
                    "rewrite rule roots cannot reference themselves",
                    source,
                    span,
                ));
            }
            pending.extend(children(expr));
        }
    }
    let mut work = MAX_MATCH_WORK;
    for function in &mut target.functions {
        rewrite_expr(
            &mut function.body,
            &from.body,
            &to.body,
            &parameters,
            &mut work,
        )
        .map_err(|()| error("reflection matching exceeded its work limit", source, span))?;
    }
    target.functions.extend(helpers);
    check_size(&target.functions, source, span)?;
    core::validate(&program(&target, source, span))?;
    let catalog = target
        .functions
        .iter()
        .map(|f| (f.name.clone(), f))
        .collect();
    strand::operation_strand(&target.root, &catalog, &target.encoded_source, source, span)
}

fn rule_parameters(
    from: &Function,
    to: &Function,
    source: &str,
    span: Option<Span>,
) -> Result<BTreeSet<String>, LanguageError> {
    let parameters = from.parameters.iter().cloned().collect::<BTreeSet<_>>();
    if from.variadic || to.variadic || to.parameters.iter().any(|p| !parameters.contains(p)) {
        return Err(error(
            "rewrite rule roots require fixed parameters bound by the pattern",
            source,
            span,
        ));
    }
    let mut occurring = BTreeSet::new();
    let mut pending = vec![&from.body];
    while let Some(expr) = pending.pop() {
        if let Expr::Reference { name, .. } = expr {
            occurring.insert(name.clone());
        }
        pending.extend(children(expr));
    }
    if to.parameters.iter().any(|p| !occurring.contains(p)) {
        return Err(error(
            "replacement parameter does not occur in the pattern",
            source,
            span,
        ));
    }
    Ok(parameters)
}

fn spend(work: &mut usize) -> Result<(), ()> {
    *work = work.checked_sub(1).ok_or(())?;
    Ok(())
}

fn rewrite_expr(
    expr: &mut Expr,
    pattern: &Expr,
    replacement: &Expr,
    parameters: &BTreeSet<String>,
    work: &mut usize,
) -> Result<(), ()> {
    spend(work)?;
    for child in children_mut(expr) {
        rewrite_expr(child, pattern, replacement, parameters, work)?;
    }
    let mut bindings = BTreeMap::new();
    if matches_pattern(pattern, expr, parameters, &mut bindings, work)? {
        let result = substitute(replacement, &bindings, work, expr.span())?;
        check_expression_size(&result)?;
        *expr = result;
    }
    Ok(())
}

fn matches_pattern<'a>(
    pattern: &Expr,
    value: &'a Expr,
    parameters: &BTreeSet<String>,
    bindings: &mut BTreeMap<String, &'a Expr>,
    work: &mut usize,
) -> Result<bool, ()> {
    spend(work)?;
    if let Expr::Reference { name, .. } = pattern
        && parameters.contains(name)
    {
        return if let Some(bound) = bindings.get(name) {
            same_expr_budget(bound, value, work)
        } else {
            bindings.insert(name.clone(), value);
            Ok(true)
        };
    }
    if header(pattern) != header(value) {
        return Ok(false);
    }
    let left = children(pattern);
    let right = children(value);
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.into_iter().zip(right) {
        if !matches_pattern(left, right, parameters, bindings, work)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn substitute(
    expr: &Expr,
    bindings: &BTreeMap<String, &Expr>,
    work: &mut usize,
    location: Option<Span>,
) -> Result<Expr, ()> {
    spend(work)?;
    if let Expr::Reference { name, .. } = expr
        && let Some(value) = bindings.get(name)
    {
        // Charge for captured trees before allocating their copies.
        let mut pending = vec![*value];
        while let Some(node) = pending.pop() {
            spend(work)?;
            pending.extend(children(node));
        }
        return Ok((*value).clone());
    }
    let mut result = expr.clone();
    // Newly constructed operations belong at the match location. Captured
    // subexpressions above keep their original locations.
    *span_mut(&mut result) = location;
    for child in children_mut(&mut result) {
        *child = substitute(child, bindings, work, location)?;
    }
    Ok(result)
}

fn same_function(a: &Function, b: &Function) -> bool {
    a.name == b.name
        && a.parameters == b.parameters
        && a.variadic == b.variadic
        && same_expr(&a.body, &b.body)
}

fn same_expr(a: &Expr, b: &Expr) -> bool {
    let mut work = MAX_MATCH_WORK;
    same_expr_budget(a, b, &mut work).unwrap_or(false)
}

fn same_expr_budget(a: &Expr, b: &Expr, work: &mut usize) -> Result<bool, ()> {
    let mut pending = vec![(a, b)];
    while let Some((a, b)) = pending.pop() {
        spend(work)?;
        if header(a) != header(b) {
            return Ok(false);
        }
        let ac = children(a);
        let bc = children(b);
        if ac.len() != bc.len() {
            return Ok(false);
        }
        pending.extend(ac.into_iter().zip(bc));
    }
    Ok(true)
}

fn check_expression_size(expr: &Expr) -> Result<(), ()> {
    let mut pending = vec![(expr, 0)];
    let mut count = 0;
    while let Some((node, depth)) = pending.pop() {
        count += 1;
        if count > MAX_NODES || depth > MAX_DEPTH {
            return Err(());
        }
        pending.extend(children(node).into_iter().map(|child| (child, depth + 1)));
    }
    Ok(())
}

fn header(expr: &Expr) -> Expr {
    let mut result = expr.clone();
    *span_mut(&mut result) = None;
    for child in children_mut(&mut result) {
        *child = Expr::Zero { span: None };
    }
    result
}

fn span_mut(expr: &mut Expr) -> &mut Option<Span> {
    match expr {
        Expr::Zero { span }
        | Expr::One { span }
        | Expr::Scalar { span, .. }
        | Expr::Reference { span, .. }
        | Expr::Spread { span, .. }
        | Expr::Call { span, .. }
        | Expr::Concat { span, .. }
        | Expr::Fold { span, .. }
        | Expr::Camera { span, .. }
        | Expr::Trace { span, .. }
        | Expr::Length { span, .. }
        | Expr::Untrace { span, .. }
        | Expr::RankDescent { span, .. }
        | Expr::Apply { span, .. }
        | Expr::Reflect { span, .. }
        | Expr::Add { span, .. }
        | Expr::Multiply { span, .. }
        | Expr::Orient { span, .. }
        | Expr::Index { span, .. } => span,
    }
}

fn children(expr: &Expr) -> Vec<&Expr> {
    match expr {
        Expr::Call { arguments, .. } | Expr::Reflect { arguments, .. } => {
            arguments.iter().collect()
        }
        Expr::Add { operands, .. } | Expr::Multiply { operands, .. } => operands.iter().collect(),
        Expr::Concat { values, .. } => values.iter().collect(),
        Expr::Fold {
            initial, values, ..
        } => std::iter::once(initial.as_ref())
            .chain(values.iter())
            .collect(),
        Expr::Camera { value, .. }
        | Expr::Length { value, .. }
        | Expr::Untrace { value, .. }
        | Expr::RankDescent { value, .. }
        | Expr::Orient { value, .. }
        | Expr::Index { value, .. } => vec![value],
        Expr::Apply { pattern, .. } => vec![pattern],
        Expr::Zero { .. }
        | Expr::One { .. }
        | Expr::Scalar { .. }
        | Expr::Reference { .. }
        | Expr::Spread { .. }
        | Expr::Trace { .. } => Vec::new(),
    }
}

fn children_mut(expr: &mut Expr) -> Vec<&mut Expr> {
    match expr {
        Expr::Call { arguments, .. } | Expr::Reflect { arguments, .. } => {
            arguments.iter_mut().collect()
        }
        Expr::Add { operands, .. } | Expr::Multiply { operands, .. } => {
            operands.iter_mut().collect()
        }
        Expr::Concat { values, .. } => values.iter_mut().collect(),
        Expr::Fold {
            initial, values, ..
        } => std::iter::once(initial.as_mut())
            .chain(values.iter_mut())
            .collect(),
        Expr::Camera { value, .. }
        | Expr::Length { value, .. }
        | Expr::Untrace { value, .. }
        | Expr::RankDescent { value, .. }
        | Expr::Orient { value, .. }
        | Expr::Index { value, .. } => vec![value],
        Expr::Apply { pattern, .. } => vec![pattern],
        Expr::Zero { .. }
        | Expr::One { .. }
        | Expr::Scalar { .. }
        | Expr::Reference { .. }
        | Expr::Spread { .. }
        | Expr::Trace { .. } => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indirect_rule_root_reference_cannot_capture_a_target_function() {
        let target = traced(
            "let to = (x) => add(x, 500)\nlet f = (x) => add(multiply(x, one), to(x))\noutput trace(f)",
        );
        let pattern = traced("let from = (x) => multiply(x, one)\noutput trace(from)");
        let replacement =
            traced("let to = (x) => helper(x)\nlet helper = (x) => to(x)\noutput trace(to)");
        let failure = rewrite(&[target, pattern, replacement], "capture.ns", None).unwrap_err();
        assert!(failure.0.message.contains("reference themselves"));
    }

    fn traced(source: &str) -> NativeState {
        core::interpret(&core::parse(source, "catalog.ns").unwrap()).unwrap()
    }

    #[test]
    fn conflicting_helper_definitions_cannot_change_callee_meaning() {
        let target =
            traced("let shared = (x) => add(x, 1)\nlet f = (x) => shared(x)\noutput trace(f)");
        let pattern = traced(
            "let shared = (x) => add(x, 2)\nlet from = (x) => shared(x)\noutput trace(from)",
        );
        let replacement = traced("let to = (x) => x\noutput trace(to)");
        let failure = rewrite(&[target, pattern, replacement], "collision.ns", None).unwrap_err();
        assert!(failure.0.message.contains("conflicting reflection helper"));
    }

    #[test]
    fn malformed_graphs_and_budget_exhaustion_return_diagnostics() {
        let mut graph = traced("let f = (x) => x\noutput trace(f)");
        graph.0.values_mut().next().unwrap().real = core::rational("1/2").unwrap();
        let failure = decode(&graph, "malformed.ns", None).unwrap_err();
        assert!(!failure.0.message.is_empty());

        let mut expression = Expr::One { span: None };
        let pattern = Expr::Reference {
            name: "x".into(),
            span: None,
        };
        let parameters = BTreeSet::from(["x".into()]);
        assert!(rewrite_expr(&mut expression, &pattern, &pattern, &parameters, &mut 0).is_err());
    }

    #[test]
    fn oversized_replacement_depth_is_rejected_before_another_pass() {
        let mut expression = Expr::One { span: None };
        for _ in 0..=MAX_DEPTH {
            expression = Expr::Orient {
                turns: 1,
                value: Box::new(expression),
                span: None,
            };
        }
        assert!(check_expression_size(&expression).is_err());
    }
}
