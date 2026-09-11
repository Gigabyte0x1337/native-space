// SPDX-License-Identifier: AGPL-3.0-or-later
use native_space_language::{
    core,
    pattern::{Observation, Pattern},
    retained::State,
    strand::execution::{Graph, Value},
};
use num_bigint::BigUint;

fn pattern(source: &str) -> Pattern {
    let graph = Graph::compile(&core::parse(source, "native-pattern.ns").unwrap()).unwrap();
    Pattern::new(
        graph.run().unwrap().native().unwrap(),
        graph.function("step").unwrap(),
    )
    .unwrap()
}

fn transform(source: &str, inputs: Vec<State>) -> State {
    let graph = Graph::compile(&core::parse(source, "transform.ns").unwrap()).unwrap();
    graph
        .function("transform")
        .unwrap()
        .call(inputs.into_iter().map(Value::State).collect())
        .unwrap()
        .native()
        .unwrap()
}

#[test]
fn only_native_records_survive_cache_destruction() {
    let (serialized, expected_seed, expected_step_bindings, expected_result) = {
        let graph = Graph::compile(
            &core::parse(
                "let advance = (offset,x) => add(offset,phase(1,x))\noutput multiply(7,0)",
                "curried.ns",
            )
            .unwrap(),
        )
        .unwrap();
        let Value::Function(step) = graph
            .function("advance")
            .unwrap()
            .call(vec![Value::State(State::zero())])
            .unwrap()
        else {
            panic!("curried")
        };
        let p = Pattern::new(graph.run().unwrap().native().unwrap(), step).unwrap();
        let selected = p.observe(BigUint::from(2_u32).pow(128));
        (
            serde_json::to_string(&selected.to_data()).unwrap(),
            p.seed().native_data(),
            p.step().bindings().project().clone(),
            p.observe(2_u32.into()).evaluate(2).unwrap().native_data(),
        )
    };
    // No Pattern, graph, environment, or State from the original survives here.
    let data = serde_json::from_str(&serialized).unwrap();
    let restored = Observation::from_data(&data).unwrap();
    assert_eq!(restored.index(), &BigUint::from(2_u32).pow(128));
    assert_eq!(restored.pattern().seed().native_data(), expected_seed);
    assert_eq!(
        restored.pattern().step().bindings().project(),
        &expected_step_bindings
    );
    assert_eq!(restored.to_data(), data);
    let seed_records = transform(
        "let transform = (o) => reflect(o,index(104,index(101,v)),v)\noutput 0",
        vec![restored.native().clone()],
    );
    assert_eq!(
        native_space_language::strand::execution::state_data::encode(restored.pattern().seed())
            .unwrap()
            .project(),
        seed_records.project(),
        "decoding also preserves retained source locations"
    );
    assert!(
        restored
            .pattern()
            .observe(2_u32.into())
            .evaluate(2)
            .unwrap()
            .native_data()
            == expected_result,
        "reloaded callable must preserve retained evaluation structure"
    );
    // Huge selection is exact; huge replay is still explicitly resource-bounded.
    restored.evaluate(2).unwrap_err();
}

#[test]
fn reflect_selects_pattern_fields_and_native_successor_matches_convenience() {
    let p = pattern("let step = (x) => phase(1,x)\noutput multiply(7,0)");
    let a = p.observe(BigUint::from(2_u32).pow(128));
    let fields = "let transform = (o) => reflect(o,index(104,v),v)\noutput 0";
    let extracted = transform(fields, vec![a.native().clone()]);
    assert_eq!(extracted.project(), p.native().project());
    let reloaded = Pattern::from_native(&extracted).unwrap();
    assert_eq!(reloaded.seed().native_data(), p.seed().native_data());
    let seed_records = transform(
        "let transform = (o) => reflect(o,index(104,index(101,v)),v)\noutput 0",
        vec![a.native().clone()],
    );
    assert_eq!(
        native_space_language::strand::execution::state_data::decode(seed_records.project())
            .unwrap()
            .native_data(),
        p.seed().native_data()
    );
    let step_records = transform(
        "let transform = (o) => reflect(o,index(104,index(102,v)),v)\noutput 0",
        vec![a.native().clone()],
    );
    let called = Value::State(step_records)
        .call(vec![Value::State(State::one())])
        .unwrap()
        .native()
        .unwrap();
    assert_eq!(
        called.project(),
        &core::NativeState::scalar(core::NativeScalar::from_text("0", "1").unwrap())
    );
    let next = transform(
        "let transform = (o) => add(o,index(103,index(6,1)))\noutput 0",
        vec![a.native().clone()],
    );
    let b = Observation::from_native(&next).unwrap();
    assert!(b.same_selection(&a.successor()));
    assert_eq!(
        b.pattern().native().project(),
        a.pattern().native().project()
    );
    let index = transform(
        "let transform = (o) => reflect(o,index(103,index(6,v)),v)\noutput 0",
        vec![b.native().clone()],
    );
    assert_eq!(
        index.project().to_data(),
        core::NativeState::scalar(
            core::NativeScalar::from_text(&b.index().to_string(), "0").unwrap()
        )
        .to_data()
    );
}

#[test]
fn reflected_step_replacement_executes_without_recompiling_either_generator() {
    let a = pattern("let step = (x) => phase(1,x)\noutput 1").observe(1_u32.into());
    let replacement = pattern("let step = (x) => multiply(3,x)\noutput 1");
    let edited = transform(
        "let transform = (o,p) => add(o,phase(2,reflect(o,index(104,index(102,v)),index(104,index(102,v)))),index(104,reflect(p,index(102,v),index(102,v))))\noutput 0",
        vec![a.native().clone(), replacement.native().clone()],
    );
    let b = Observation::from_native(&edited).unwrap();
    assert_eq!(
        b.evaluate(1).unwrap().project(),
        &core::NativeState::scalar(core::NativeScalar::from_text("3", "0").unwrap())
    );
    assert_eq!(
        b.pattern().seed().native_data(),
        a.pattern().seed().native_data()
    );
}

#[test]
fn payload_schema_addresses_and_power_depth_cannot_alias_observation_index() {
    let source = "let step = (x) => x\noutput 1";
    let graph = Graph::compile(&core::parse(source, "collision.ns").unwrap()).unwrap();
    let mut seed = State::one();
    for direction in [1, 6, 100, 101, 102, 103, 104, 105, 1024, u64::MAX] {
        seed = seed.index_power(direction, 7).unwrap();
    }
    seed = seed.add(&State::one().multiply(&State::zero()));
    let p = Pattern::new(seed.clone(), graph.function("step").unwrap()).unwrap();
    let a = p.observe(0_u32.into());
    let restored = Observation::from_data(&a.to_data()).unwrap();
    assert_eq!(restored.index(), &BigUint::from(0_u32));
    assert_eq!(restored.pattern().seed().native_data(), seed.native_data());
    assert_eq!(restored.project(0).unwrap(), *seed.project());
    assert!(restored.same_selection(&a));
    assert!(!restored.same_selection(&a.successor()));
}

#[test]
fn malformed_native_metadata_is_rejected_instead_of_ignored() {
    let a = pattern("let step = (x) => phase(1,x)\noutput 1").observe(0_u32.into());
    for source in [
        "let transform = (o) => add(o,index(103,index(6,-1)))\noutput 0",
        "let transform = (o) => add(o,index(103,index(6,1/2)))\noutput 0",
        "let transform = (o) => add(o,index(103,index(6,phase(1,1))))\noutput 0",
        "let transform = (o) => add(o,index(99,1))\noutput 0",
        "let transform = (o) => add(o,index(105,1))\noutput 0",
        "let transform = (o) => index(104,o)\noutput 0",
        "let transform = (o) => add(o,phase(2,index(103,index(1,1))))\noutput 0",
    ] {
        Observation::from_native(&transform(source, vec![a.native().clone()])).unwrap_err();
    }
}
#[test]
fn native_library_and_successors_preserve_a_fixed_generator() {
    let p = pattern("let step = (x) => phase(1,x)\noutput 1");
    let mut selected = p.observe(1_u32.into());
    let node_count = selected.to_data()["nodes"].as_array().unwrap().len();
    for _ in 0..256 {
        selected = selected.successor();
        assert!(selected.pattern().shares_generator(&p));
    }
    assert_eq!(
        selected.to_data()["nodes"].as_array().unwrap().len(),
        node_count
    );
    let source = concat!(
        include_str!("../../pattern.ns"),
        "\nlet transform = (o) => observation_successor(o)\noutput 0"
    );
    let next =
        Observation::from_native(&transform(source, vec![selected.native().clone()])).unwrap();
    assert!(next.same_selection(&selected.successor()));
    let restored = Pattern::from_data(&p.to_data()).unwrap();
    assert_eq!(restored.native().project(), p.native().project());
    let a = restored.observe(1_u32.into());
    let b = restored.observe(5_u32.into());
    assert_eq!(a.project(5).unwrap(), b.project(5).unwrap());
    assert!(!a.same_selection(&b));
}
