// SPDX-License-Identifier: AGPL-3.0-or-later
use native_space_language::{
    core::{self, NativeScalar, NativeState},
    pattern::Pattern,
    retained::State,
    strand::execution::{Graph, Value},
};
use num_bigint::BigUint;

#[test]
fn cyclic_playback_advances_observations_without_expanding_the_compiled_generator() {
    let p = generator("let step = (x) => phase(1,x)\noutput 1");
    let original = p.native().native_data();
    let mut cursor = p.cursor();
    let mut selected = cursor.observation();
    // Many complete cycles, with an independent exact four-phase oracle.
    for k in 0_u32..=256 {
        let (real, imag) = match k % 4 {
            0 => ("1", "0"),
            1 => ("0", "1"),
            2 => ("-1", "0"),
            _ => ("0", "-1"),
        };
        let expected = NativeState::scalar(NativeScalar::from_text(real, imag).unwrap());
        assert_eq!(
            cursor.seek(selected.index(), 256).unwrap().project(),
            &expected
        );
        let reached = cursor.observation();
        assert!(reached.same_selection(&selected));
        assert_eq!(reached.index(), &BigUint::from(k));
        assert!(reached.pattern().step().shares_graph(p.step()));
        let successor = reached.successor();
        assert!(!successor.same_selection(&reached));
        assert!(successor.pattern().shares_generator(&p));
        selected = successor;
    }
    assert_eq!(
        cursor.observation().pattern().native().native_data(),
        original
    );
    assert_eq!(cursor.observation().index(), &BigUint::from(256_u32));
    let reached = cursor.observation();
    cursor.seek(selected.index(), 256).unwrap_err();
    assert!(cursor.observation().same_selection(&reached));
}

#[test]
fn successor_crosses_machine_integer_limits_without_wrapping_or_materializing_steps() {
    let p = generator("let step = (x) => phase(1,x)\noutput 1");
    let original = p.native().native_data();
    for k in [BigUint::from(u64::MAX), BigUint::from(2_u32).pow(256)] {
        let a = p.observe(k.clone());
        let b = a.successor();
        assert_eq!(b.index(), &(k + 1_u32));
        assert!(!a.same_selection(&b));
        assert!(a.pattern().shares_generator(b.pattern()));
        assert!(a.pattern().step().shares_graph(b.pattern().step()));
        assert_eq!(b.pattern().native().native_data(), original);
        b.evaluate(0).unwrap_err();
    }
}

fn generator(source: &str) -> Pattern {
    let graph = Graph::compile(&core::parse(source, "pattern.ns").unwrap()).unwrap();
    Pattern::new(
        graph.run().unwrap().native().unwrap(),
        graph.function("step").unwrap(),
    )
    .unwrap()
}

fn scalar(value: &str) -> NativeState {
    NativeState::scalar(NativeScalar::from_text(value, "0").unwrap())
}

#[test]
fn observation_zero_is_seed_and_k_is_exactly_k_repetitions() {
    let p = generator("let step = (x) => multiply(2,x)\noutput 1");
    for (k, expected) in ["1", "2", "4", "8", "16"].into_iter().enumerate() {
        assert_eq!(p.observe(k.into()).project(4).unwrap(), scalar(expected));
    }
    // Random observation order cannot mutate a shared playback cursor.
    assert_eq!(p.observe(2_u32.into()).project(4).unwrap(), scalar("4"));
}

#[test]
fn same_phase_at_k_and_k_plus_four_is_not_the_same_observation() {
    let p = generator("let step = (x) => phase(1,x)\noutput 1");
    for k in 0_u32..8 {
        let a = p.observe(k.into());
        let b = p.observe((k + 4).into());
        assert_eq!(a.project(12).unwrap(), b.project(12).unwrap());
        assert!(!a.same_selection(&b));
        assert!(a.pattern().shares_generator(b.pattern()));
        assert!(a.pattern().step().shares_graph(b.pattern().step()));
        assert_ne!(a.native().project(), b.native().project());
    }
}

#[test]
fn selections_keep_one_finite_generator_without_materializing_a_prefix() {
    let p = generator("let step = (x) => phase(1,x)\noutput multiply(0,7)");
    let first = p.native().native_data();
    let seed = p.seed().native_data();
    let huge = p.observe(BigUint::from(10_u32).pow(100));
    assert_eq!(first, huge.pattern().native().native_data());
    assert!(
        huge.project(100)
            .unwrap_err()
            .to_string()
            .contains("budget")
    );
    assert_eq!(p.seed().native_data(), seed);
    assert!(p.seed().native_data()["nodes"].as_array().unwrap().len() > 1);
}

#[test]
fn repeated_projection_does_not_cache_replay_history_in_the_generator() {
    let p = generator("let step = (x) => reflect(phase(1,x),v,v)\noutput 1");
    let observation = p.observe(4096_u32.into());
    let before = observation.to_data();
    assert_eq!(observation.project(4096).unwrap(), scalar("1"));
    assert_eq!(observation.to_data(), before);
}

#[test]
fn power_depth_is_independent_of_observation_index() {
    let p = generator("let step = (x) => multiply(index(7,2),x)\noutput 1");
    let third = p.observe(3_u32.into());
    let value = third.project(3).unwrap();
    let (indices, coefficient) = value.0.first_key_value().unwrap();
    assert_eq!(indices.depth(7), BigUint::from(3_u32));
    assert_eq!(coefficient, &NativeScalar::from_text("8", "0").unwrap());
    assert_eq!(third.index(), &BigUint::from(3_u32));
    let stationary = generator("let step = (x) => x\noutput index(7,1,9)");
    assert_eq!(
        stationary
            .observe(3_u32.into())
            .project(3)
            .unwrap()
            .0
            .first_key_value()
            .unwrap()
            .0
            .depth(7),
        BigUint::from(9_u32)
    );
}

#[test]
fn curried_steps_share_the_existing_graph_and_use_independent_zero_bindings() {
    let graph = Graph::compile(
        &core::parse("let offset = (a,x) => add(a,x)\noutput 1", "curry.ns").unwrap(),
    )
    .unwrap();
    let f = graph.function("offset").unwrap();
    let Value::Function(bound) = f.call(vec![Value::State(State::zero())]).unwrap() else {
        panic!("partial function")
    };
    assert!(f.shares_graph(&bound));
    let p = Pattern::new(State::one(), bound).unwrap();
    assert_eq!(p.observe(10_u32.into()).project(10).unwrap(), scalar("1"));
    assert!(
        Pattern::new(State::one(), f)
            .unwrap_err()
            .to_string()
            .contains("one unbound")
    );
}

#[test]
fn failing_steps_do_not_invalidate_other_observations_or_mutate_the_seed() {
    let p = generator("let other = (y) => y\nlet step = (x) => other\noutput 3");
    assert!(
        p.observe(1_u32.into())
            .project(1)
            .unwrap_err()
            .to_string()
            .contains("return a Native state")
    );
    assert_eq!(p.observe(0_u32.into()).project(0).unwrap(), scalar("3"));
}

#[test]
fn reflective_generators_receive_retained_inputs_not_canonicalized_replacements() {
    let p =
        generator("let hold = (a,b) => b\nlet step = (x) => add(hold(x),0)\noutput multiply(7,0)");
    let expected = p
        .step()
        .call(vec![Value::State(p.seed().clone())])
        .unwrap()
        .native()
        .unwrap();
    let flattened = p
        .step()
        .call(vec![Value::State(State::from_projection(
            p.seed().project(),
        ))])
        .unwrap()
        .native()
        .unwrap();
    assert_ne!(expected.project(), flattened.project());
    assert_eq!(
        p.observe(1_u32.into()).project(1).unwrap(),
        *expected.project()
    );
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "unit phase landmarks and integer radii are exactly representable"
)]
fn cylindrical_camera_uses_repetition_not_payload_depth_and_keeps_initial_phase() {
    use native_space_language::retained::coordinates::{Point, pattern_projection};
    let p = generator("let step = (x) => phase(1,x)\noutput index(7,1,99)");
    let camera = |k: u32| {
        let data = pattern_projection(&p.observe(k.into()), 5).unwrap();
        Point::from_data(&data["terms"][0]["point"]).unwrap()
    };
    assert_eq!(camera(0).to_f64().unwrap(), [0.0, 1.0, 0.0]);
    assert_eq!(camera(1).to_f64().unwrap(), [0.0, 0.0, 2.0]);
    assert_eq!(camera(5).to_f64().unwrap(), [0.0, 0.0, 6.0]);
    assert_eq!(camera(1).scalar(), camera(5).scalar());
    assert_ne!(camera(1).index(), camera(5).index());
}
#[test]
fn retained_evaluation_matches_manual_steps_and_preserves_seed_provenance() {
    let p = generator("let step = (x) => phase(1,x)\noutput multiply(7,0)");
    assert_eq!(
        p.observe(0_u32.into()).evaluate(0).unwrap().native_data(),
        p.seed().native_data()
    );
    let mut manual = p.seed().clone();
    for k in 1_u32..5 {
        manual = p
            .step()
            .call(vec![Value::State(manual)])
            .unwrap()
            .native()
            .unwrap();
        let observed = p.observe(k.into()).evaluate(4).unwrap();
        assert_eq!(observed.native_data(), manual.native_data());
        assert_eq!(observed.project(), &p.observe(k.into()).project(4).unwrap());
    }
}
#[test]
fn cursor_and_independent_observations_agree_after_forward_backward_and_failed_seeks() {
    for source in [
        "let step = (x) => phase(1,x)\noutput multiply(7,0)",
        "let step = (x) => multiply(index(7,2),x)\noutput 1",
    ] {
        let p = generator(source);
        let before = p.observe(0_u32.into()).to_data();
        let mut cursor = p.cursor();
        for k in [0_u32, 1, 3, 3, 1, 0, 4, 2] {
            let expected = p.observe(k.into()).evaluate(4).unwrap();
            assert_eq!(
                cursor.seek(&k.into(), 4).unwrap().native_data(),
                expected.native_data()
            );
        }
        cursor.seek(&BigUint::from(10_u32).pow(100), 4).unwrap_err();
        assert_eq!(
            cursor.seek(&3_u32.into(), 4).unwrap().native_data(),
            p.observe(3_u32.into()).evaluate(4).unwrap().native_data()
        );
        assert_eq!(p.observe(0_u32.into()).to_data(), before);
    }
    let p = generator("let f = (x) => x\nlet step = (x) => f\noutput 7");
    let mut cursor = p.cursor();
    cursor.seek(&1_u32.into(), 1).unwrap_err();
    assert_eq!(
        cursor.seek(&0_u32.into(), 1).unwrap().native_data(),
        p.seed().native_data()
    );
}
#[test]
fn cursor_preserves_reflective_seed_after_restart() {
    // One step is sufficient to distinguish retained from canonicalized input.
    // Repeated self-encoding grows the encoded program, not the playback history.
    let p =
        generator("let hold = (a,b) => b\nlet step = (x) => add(hold(x),0)\noutput multiply(7,0)");
    let expected = p.observe(1_u32.into()).evaluate(1).unwrap();
    let mut cursor = p.cursor();
    for k in [1_u32, 0, 1] {
        let state = cursor.seek(&k.into(), 1).unwrap();
        assert_eq!(
            state.native_data(),
            if k == 0 {
                p.seed().native_data()
            } else {
                expected.native_data()
            }
        );
    }
}
