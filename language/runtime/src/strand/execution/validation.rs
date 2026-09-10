// SPDX-License-Identifier: AGPL-3.0-or-later

//! Loaded Native data receives the same scope checks before becoming executable.
use super::{
    BTreeSet, CALL, Graph, INDEX, INDEX_CAPTURE, LITERAL, LanguageError, NativeScalar, REFERENCE,
    REFLECT, SPREAD, diagnostic, required_text, required_u64,
};

impl Graph {
    pub(super) fn validate_references(&self) -> Result<(), LanguageError> {
        for signature in self.functions.values() {
            self.validate_expression(
                signature.body,
                &signature.parameters.iter().cloned().collect(),
            )?;
        }
        let mut names = BTreeSet::new();
        for (name, address) in &self.bindings {
            if self.functions.contains_key(name) {
                return Err(diagnostic(
                    "binding collides with a function",
                    &self.source,
                    self.records[*address].span,
                ));
            }
            self.validate_expression(*address, &names)?;
            names.insert(name.clone());
        }
        if let Some(entry) = self.entry {
            self.validate_expression(entry, &names)?;
        }
        Ok(())
    }

    fn validate_expression(
        &self,
        root: usize,
        names: &BTreeSet<String>,
    ) -> Result<(), LanguageError> {
        let mut pending = vec![root];
        while let Some(address) = pending.pop() {
            let record = &self.records[address];
            let children = &self.children[address];
            let error = |message| diagnostic(message, &self.source, record.span);
            match record.kind {
                REFERENCE | SPREAD => {
                    let name = required_text(record.name.as_deref(), &self.source, record.span)?;
                    if !names.contains(name) && !self.functions.contains_key(name) {
                        return Err(error("unbound graph reference"));
                    }
                    if record.kind == SPREAD && !names.contains(name) {
                        return Err(error("spread requires a bound argument pack"));
                    }
                }
                CALL => {
                    let callee = &self.records[children[0]];
                    if callee.kind == REFERENCE
                        && let Some(name) = callee.name.as_deref()
                        && !names.contains(name)
                        && let Some(signature) = self.functions.get(name)
                        && !signature.variadic
                        && !children[1..]
                            .iter()
                            .any(|child| self.records[*child].kind == SPREAD)
                        && children.len() - 1 > signature.parameters.len()
                    {
                        return Err(error("too many arguments for function"));
                    }
                }
                LITERAL => {
                    NativeScalar::from_text(
                        record.text_a.as_deref().unwrap_or("0"),
                        record.text_b.as_deref().unwrap_or("0"),
                    )
                    .map_err(|_error| error("invalid scalar coordinates"))?;
                }
                INDEX => {
                    if required_u64(record.number_a.as_deref(), &self.source, record.span)? == 0 {
                        return Err(error("index direction must be positive"));
                    }
                    if record.kind == INDEX
                        && required_u64(record.number_b.as_deref(), &self.source, record.span)? == 0
                    {
                        return Err(error("index depth must be positive"));
                    }
                }
                INDEX_CAPTURE => {
                    return Err(error(
                        "INDEX depth capture belongs inside a reflect template",
                    ));
                }
                REFLECT => match record.name.as_deref() {
                    Some("reflect") if children.len() == 3 => {}
                    _ => return Err(error("invalid reflection operation or argument count")),
                },
                _ => {}
            }
            if self.rules.contains_key(&address) {
                pending.push(children[0]);
            } else {
                pending.extend(children.iter().copied());
            }
        }
        Ok(())
    }
}
