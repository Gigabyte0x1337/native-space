// SPDX-License-Identifier: AGPL-3.0-or-later
//! Small expression grammar. Named references preserve finite recursive function graphs.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

pub const BUILTINS: &[&str] = &[
    "state",
    "add",
    "multiply",
    "negate",
    "inverse",
    "split",
    "transform",
    "decode",
    "mutate",
    "program",
    "observe",
    "reflect",
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expr {
    pub line: usize,
    pub column: usize,
    pub kind: Kind,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Kind {
    Number(String),
    Text(String),
    Name(String),
    List(Vec<Arc<Expr>>),
    Record(BTreeMap<String, Arc<Expr>>),
    Call(Arc<Expr>, Vec<Arc<Expr>>),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Function {
    pub parameters: Vec<String>,
    pub body: Arc<Expr>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub functions: BTreeMap<String, Function>,
    pub bindings: Vec<(String, Arc<Expr>)>,
    pub output: Arc<Expr>,
    pub format: String,
    pub source_name: String,
}
#[derive(Clone, Debug)]
struct Token {
    text: String,
    line: usize,
    column: usize,
    string: bool,
}
struct Parser {
    tokens: Vec<Token>,
    at: usize,
    source: String,
    depth: usize,
}
impl Parser {
    fn error(&self, message: &str) -> String {
        let t = &self.tokens[self.at.min(self.tokens.len() - 1)];
        format!("{}:{}:{}: {}", self.source, t.line, t.column, message)
    }
    fn is(&self, s: &str) -> bool {
        self.tokens[self.at].text == s && !self.tokens[self.at].string
    }
    fn take(&mut self) -> Token {
        let t = self.tokens[self.at].clone();
        if self.at + 1 < self.tokens.len() {
            self.at += 1;
        }
        t
    }
    fn expect(&mut self, s: &str) -> Result<(), String> {
        if !self.is(s) {
            return Err(self.error(&format!("expected {s}")));
        }
        self.take();
        Ok(())
    }
    fn name(&mut self) -> Result<String, String> {
        let t = self.take();
        if t.string || !identifier(&t.text) {
            return Err(self.error("expected a name"));
        }
        Ok(t.text)
    }
    fn expr(&mut self) -> Result<Arc<Expr>, String> {
        if self.depth >= 128 {
            return Err(self.error("expression nesting exceeds 128"));
        }
        self.depth += 1;
        let t = self.take();
        let kind = if t.string {
            Kind::Text(t.text)
        } else {
            match t.text.as_str() {
                "(" => {
                    let e = self.expr()?;
                    self.expect(")")?;
                    e.kind.clone()
                }
                "[" => {
                    let mut xs = Vec::new();
                    if !self.is("]") {
                        loop {
                            xs.push(self.expr()?);
                            if !self.is(",") {
                                break;
                            }
                            self.take();
                        }
                    }
                    self.expect("]")?;
                    Kind::List(xs)
                }
                "{" => {
                    let mut xs = BTreeMap::new();
                    if !self.is("}") {
                        loop {
                            let key = self.take();
                            if !(key.string || identifier(&key.text)) {
                                return Err(self.error("expected record field"));
                            }
                            self.expect(":")?;
                            if xs.insert(key.text, self.expr()?).is_some() {
                                return Err(self.error("duplicate record field"));
                            }
                            if !self.is(",") {
                                break;
                            }
                            self.take();
                        }
                    }
                    self.expect("}")?;
                    Kind::Record(xs)
                }
                s if crate::algebra::rational(s).is_ok() => Kind::Number(t.text),
                s if identifier(s) => Kind::Name(t.text),
                _ => return Err(self.error("expected an expression")),
            }
        };
        let mut e = Arc::new(Expr {
            line: t.line,
            column: t.column,
            kind,
        });
        while self.is("(") {
            self.take();
            let mut args = Vec::new();
            if !self.is(")") {
                loop {
                    args.push(self.expr()?);
                    if !self.is(",") {
                        break;
                    }
                    self.take();
                }
            }
            self.expect(")")?;
            e = Arc::new(Expr {
                line: t.line,
                column: t.column,
                kind: Kind::Call(e, args),
            });
        }
        self.depth -= 1;
        Ok(e)
    }
}
fn identifier(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}
fn lex(source: &str, name: &str) -> Result<Vec<Token>, String> {
    if source.len() > 128_000 {
        return Err("source exceeds 128 KB".into());
    }
    let mut it = source.chars().peekable();
    let (mut line, mut col) = (1, 1);
    let mut out = Vec::new();
    while let Some(c) = it.next() {
        let start = col;
        col += 1;
        if c == '\n' {
            line += 1;
            col = 1;
            continue;
        }
        if c.is_whitespace() {
            continue;
        }
        if c == '#' {
            while it.peek().is_some_and(|c| *c != '\n') {
                it.next();
                col += 1;
            }
            continue;
        }
        let mut text = String::new();
        let string = c == '"';
        if string {
            let mut closed = false;
            while let Some(c) = it.next() {
                col += 1;
                if c == '"' {
                    closed = true;
                    break;
                }
                if c == '\n' {
                    return Err(format!("{name}:{line}:{start}: newline in string"));
                }
                if c == '\\' {
                    let escape = it.next().ok_or("unfinished string")?;
                    col += 1;
                    text.push(match escape {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '"' => '"',
                        '\\' => '\\',
                        _ => return Err("unsupported string escape".into()),
                    });
                } else {
                    text.push(c);
                }
            }
            if !closed {
                return Err(format!("{name}:{line}:{start}: unfinished string"));
            }
        } else if c == '=' && it.peek() == Some(&'>') {
            it.next();
            col += 1;
            text = "=>".into();
        } else if "()[]{},:=".contains(c) {
            text.push(c);
        } else {
            text.push(c);
            while it
                .peek()
                .is_some_and(|c| !c.is_whitespace() && !"()[]{},:=#".contains(*c))
            {
                text.push(it.next().expect("peeked"));
                col += 1;
            }
        }
        out.push(Token {
            text,
            line,
            column: start,
            string,
        });
    }
    out.push(Token {
        text: "<eof>".into(),
        line,
        column: col,
        string: false,
    });
    Ok(out)
}
pub fn parse(source: &str, name: &str) -> Result<Graph, String> {
    let mut p = Parser {
        tokens: lex(source, name)?,
        at: 0,
        source: name.into(),
        depth: 0,
    };
    let mut functions = BTreeMap::new();
    let mut bindings = Vec::new();
    let mut names = std::collections::BTreeSet::new();
    while p.is("let") {
        p.take();
        let name = p.name()?;
        if BUILTINS.contains(&name.as_str())
            || ["let", "output", "as", "phase", "index"].contains(&name.as_str())
            || !names.insert(name.clone())
        {
            return Err(p.error("duplicate or reserved definition"));
        }
        p.expect("=")?;
        let saved = p.at;
        let mut parameters = Vec::new();
        let mut function = false;
        if p.is("(") {
            p.take();
            while !p.is(")") && identifier(&p.tokens[p.at].text) {
                parameters.push(p.name()?);
                if !p.is(",") {
                    break;
                }
                p.take();
            }
            if p.is(")") {
                p.take();
                if p.is("=>") {
                    p.take();
                    function = true;
                }
            }
        }
        if function {
            let mut unique = std::collections::BTreeSet::new();
            if parameters
                .iter()
                .any(|n| !unique.insert(n) || BUILTINS.contains(&n.as_str()))
            {
                return Err(p.error("duplicate or reserved parameter"));
            }
            functions.insert(
                name,
                Function {
                    parameters,
                    body: p.expr()?,
                },
            );
        } else {
            p.at = saved;
            bindings.push((name, p.expr()?));
        }
    }
    p.expect("output")?;
    let output = p.expr()?;
    let format = if p.is("as") {
        p.take();
        p.name()?
    } else {
        "state".into()
    };
    if !["state", "number", "vector", "program"].contains(&format.as_str()) {
        return Err(p.error("output format must be state, number, vector, or program"));
    }
    p.expect("<eof>")?;
    let graph = Graph {
        functions,
        bindings,
        output,
        format,
        source_name: name.into(),
    };
    validate(&graph)?;
    Ok(graph)
}
/// Validate reloaded or reflectively rebuilt graphs without recompiling source.
pub fn validate(g: &Graph) -> Result<(), String> {
    // Source and reflected/reloaded graphs obey the same naming contract.
    let valid_name = |name: &str| {
        identifier(name)
            && !BUILTINS.contains(&name)
            && !["let", "output", "as", "phase", "index"].contains(&name)
    };
    let mut names = std::collections::BTreeSet::new();
    for (name, function) in &g.functions {
        if !valid_name(name) || !names.insert(name) {
            return Err("invalid or duplicate graph definition".into());
        }
        let mut parameters = std::collections::BTreeSet::new();
        for parameter in &function.parameters {
            if !valid_name(parameter) || !parameters.insert(parameter) {
                return Err("invalid or duplicate graph parameter".into());
            }
        }
    }
    for (name, _) in &g.bindings {
        if !valid_name(name) || !names.insert(name) {
            return Err("invalid or duplicate graph definition".into());
        }
    }
    if !["state", "number", "vector", "program"].contains(&g.format.as_str()) {
        return Err("invalid graph output format".into());
    }
    fn expr(e: &Expr, depth: usize, n: &mut usize) -> Result<(), String> {
        *n += 1;
        if depth > 128 || *n > 100_000 {
            return Err("graph exceeds structural budget".into());
        }
        match &e.kind {
            Kind::Number(v) => {
                crate::algebra::rational(v)?;
            }
            Kind::Name(s) if s == "phase" || s == "index" => {
                return Err("PHASE and arithmetic INDEX are not NS2 operations".into());
            }
            Kind::List(xs) => {
                for x in xs {
                    expr(x, depth + 1, n)?;
                }
            }
            Kind::Record(xs) => {
                for x in xs.values() {
                    expr(x, depth + 1, n)?;
                }
            }
            Kind::Call(f, xs) => {
                expr(f, depth + 1, n)?;
                for x in xs {
                    expr(x, depth + 1, n)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let mut n = 0;
    for f in g.functions.values() {
        expr(&f.body, 0, &mut n)?;
    }
    for (_, e) in &g.bindings {
        expr(e, 0, &mut n)?;
    }
    expr(&g.output, 0, &mut n)
}
