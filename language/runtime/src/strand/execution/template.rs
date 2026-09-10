// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    ADD, Graph, INDEX, INDEX_CAPTURE, LITERAL, MULTIPLY, NativeScalar, PHASE, REFERENCE,
    i64_or_zero,
};
use crate::value_reflection::{Template, TemplateNode};

#[derive(Clone, Copy)]
pub(super) struct View<'a> {
    pub graph: &'a Graph,
    pub address: usize,
}

impl Template for View<'_> {
    fn node(self) -> Result<TemplateNode<Self>, String> {
        let record = &self.graph.records[self.address];
        let children = &self.graph.children[self.address];
        let child = |position| Self {
            graph: self.graph,
            address: children[position],
        };
        let number = |value: Option<&str>| {
            value
                .ok_or("missing template integer")?
                .parse::<u64>()
                .map_err(|_error| "invalid template integer")
        };
        let name = || {
            record
                .name
                .clone()
                .ok_or_else(|| "missing template capture".to_owned())
        };
        Ok(match record.kind {
            LITERAL => TemplateNode::Scalar(NativeScalar::from_text(
                record.text_a.as_deref().unwrap_or("0"),
                record.text_b.as_deref().unwrap_or("0"),
            )?),
            REFERENCE => TemplateNode::Reference(name()?),
            INDEX => TemplateNode::Index(
                number(record.number_a.as_deref())?,
                number(record.number_b.as_deref())?,
                child(0),
            ),
            INDEX_CAPTURE => {
                TemplateNode::Depth(number(record.number_a.as_deref())?, name()?, child(0))
            }
            PHASE => TemplateNode::Phase(
                i64_or_zero(record.number_a.as_deref(), &self.graph.source, record.span)
                    .map_err(|error| error.to_string())?,
                child(0),
            ),
            ADD => TemplateNode::Add((0..children.len()).map(child).collect()),
            MULTIPLY => TemplateNode::Multiply((0..children.len()).map(child).collect()),
            _ => {
                return Err(
                    "reflect templates use captures and ADD, MULTIPLY, PHASE, INDEX".into(),
                );
            }
        })
    }
}
