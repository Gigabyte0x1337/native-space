// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn repeated_calls_to_reflected_data_reuse_one_decoded_graph() {
    let source = crate::core::parse("let f = (a,b) => add(a,b)\noutput f", "cache.ns").unwrap();
    let native = Graph::compile(&source)
        .unwrap()
        .run()
        .unwrap()
        .native()
        .unwrap();
    let mut budget = Budget::default();
    let call = |budget: &mut Budget| {
        machine::call(
            Value::State(native.clone()),
            vec![Value::State(State::one())],
            budget,
            "cache.ns",
            None,
        )
        .unwrap()
    };
    let Value::Function(first) = call(&mut budget) else {
        panic!("partial")
    };
    let Value::Function(second) = call(&mut budget) else {
        panic!("partial")
    };
    assert!(first.shares_graph(&second));
    assert_eq!(budget.loaded.len(), 1);
}

#[test]
fn edited_records_are_validated_before_execution() {
    let program = crate::core::parse(
        "let f = (x) => reflect(x, index(7,v),v)\noutput f(1)",
        "edited.ns",
    )
    .unwrap();
    let graph = Graph::compile(&program).unwrap();
    for mutation in 0..5 {
        let mut records = graph.records.clone();
        match mutation {
            0 => {
                records
                    .iter_mut()
                    .find(|record| record.kind == REFERENCE)
                    .unwrap()
                    .name = Some("missing".into());
            }
            1 => {
                records
                    .iter_mut()
                    .find(|record| record.kind == REFLECT)
                    .unwrap()
                    .number_a = Some("0".into());
            }
            2 => {
                records
                    .iter_mut()
                    .find(|record| record.kind == PARAMETER)
                    .unwrap()
                    .number_a = Some("99".into());
            }
            3 => {
                records
                    .iter_mut()
                    .find(|record| record.kind == INDEX)
                    .unwrap()
                    .number_a = Some("0".into());
            }
            _ => {
                records
                    .iter_mut()
                    .find(|record| record.kind == INDEX)
                    .unwrap()
                    .opcode_turn = Some(0);
            }
        }
        let data = literal(&nest(records)).unwrap();
        Graph::load(data, "edited.ns").unwrap_err();
    }
}

#[test]
fn supplied_record_coordinates_have_logarithmic_sum_depth() {
    let coordinates = NativeState::from_terms((1..=1024).map(|position| {
        (
            crate::core::MultiIndex::from_depths([(7, position)]).unwrap(),
            NativeScalar::one(),
        )
    }));
    let state = State::from_projection(&coordinates);
    let mut pending = vec![(&state, 0)];
    let mut deepest = 0;
    while let Some((state, depth)) = pending.pop() {
        deepest = deepest.max(depth);
        pending.extend(state.inputs().iter().map(|input| (input, depth + 1)));
    }
    assert!(
        deepest < 40,
        "record lifting must not create a linear prefix chain: {deepest}"
    );
    assert_eq!(
        State::from_data(&state.native_data()).unwrap().project(),
        &coordinates
    );
}
