// SPDX-License-Identifier: AGPL-3.0-or-later
use native_space_language::{
    algebra::{Split, State, Transform, rational},
    camera, optimize,
    runtime::{Program, Session, Value},
};
use std::sync::Arc;
fn s(l: i64, a: i64, m: i64) -> State {
    State::new(
        rational(&l.to_string()).unwrap(),
        rational(&a.to_string()).unwrap(),
        rational(&m.to_string()).unwrap(),
    )
}
fn number(source: &str) -> String {
    Session::new(source, "test.ns")
        .unwrap()
        .output
        .number_readout()
        .unwrap()
        .to_string()
}
#[test]
fn exact_arithmetic_over_signed_coordinates() {
    for l in -3..=3 {
        if l == 0 {
            continue;
        }
        for a in -3..=3 {
            for m in -3..=3 {
                for b in -2..=2 {
                    let p = s(l, a, m);
                    let q = s(2, b, 3);
                    let (r, x) = p.decode().unwrap();
                    let (t, y) = q.decode().unwrap();
                    assert_eq!(p.add(&q).decode().unwrap(), (&r + &t, &x + &y));
                    assert_eq!(p.multiply(&q).decode().unwrap(), (&r * &t, &x * &y));
                    assert_eq!(p.negate().decode().unwrap(), (-&r, -&x));
                    if m != 0 && a + m != 0 {
                        let inverse = p.inverse().unwrap();
                        assert_eq!(inverse.decode().unwrap(), (r.recip(), x.recip()));
                        assert!(p.multiply(&inverse).equivalent(&State::one()).unwrap());
                    }
                }
            }
        }
    }
}
#[test]
fn identities_scale_and_projective_distributivity() {
    let p = s(2, 2, 2);
    let q = s(1, 1, 1);
    assert_eq!(p.add(&State::zero()), p);
    assert_eq!(p.multiply(&State::one()), p);
    let lhs = p.multiply(&q.add(&q));
    let rhs = p.multiply(&q).add(&p.multiply(&q));
    assert_ne!(lhs, rhs);
    assert!(lhs.equivalent(&rhs).unwrap());
    assert_eq!(rhs, lhs.rescale(&rational("2").unwrap()).unwrap());
    let big = rational(&(num_bigint::BigInt::from(1) << 1000usize).to_string()).unwrap();
    for c in [big.clone(), big.recip(), rational("-7").unwrap()] {
        let scaled = p.rescale(&c).unwrap();
        assert_ne!(p, scaled);
        assert!(p.equivalent(&scaled).unwrap());
        assert_eq!(scaled.rescale(&c.recip()).unwrap(), p);
    }
}
#[test]
fn half_split_preserves_total_not_scale_doubling() {
    for l in 1..=4 {
        for a in -3..=3 {
            for m in -3..=3 {
                let p = s(l, a, m);
                let (r, x) = p.decode().unwrap();
                for (route, offset) in [(Split::Add, 0), (Split::Multiply, 1)] {
                    let q = p.split(route);
                    assert_eq!(p.total(), q.total());
                    assert_eq!(
                        q.decode().unwrap(),
                        (
                            r.clone() * rational("2").unwrap() + rational("1").unwrap(),
                            x.clone() * rational("2").unwrap()
                                + rational(&offset.to_string()).unwrap()
                        )
                    );
                }
            }
        }
    }
}
#[test]
fn singularities_are_camera_domains() {
    assert!(s(0, 1, 1).decode().is_err());
    assert!(s(1, 0, 0).inverse().is_err());
    assert!(s(1, -2, 2).inverse().is_err());
    let p = s(1, -2, 1);
    assert!(p.simplex().is_err());
    assert_eq!(p.decode().unwrap().1, rational("1").unwrap());
    assert!(camera::log_ratio(&p).is_err());
    assert!(camera::raw(&p).is_ok());
}
#[test]
fn reversible_cameras_and_balance() {
    let p = s(2, 3, 7);
    let raw = camera::raw(&p).unwrap();
    for inverse in [
        camera::orthogonal_inverse(camera::orthogonal(&p).unwrap()).unwrap(),
        camera::log_ratio_inverse(camera::log_ratio(&p).unwrap()).unwrap(),
    ] {
        for (a, b) in raw.iter().zip(inverse) {
            assert!((a - b).abs() < 1e-12);
        }
    }
    let center = camera::log_ratio(&s(1, 1, 1)).unwrap();
    assert_eq!(center[0], 0.0);
    assert_eq!(center[1], 0.0);
    let swapped = camera::log_ratio(&s(2, 7, 3)).unwrap();
    let original = camera::log_ratio(&p).unwrap();
    assert!((swapped[0] + original[0]).abs() < 1e-12);
    assert_eq!(swapped[1], original[1]);
    let t = optimize::balanced_frame(&p).unwrap();
    assert_eq!(t.decode(&t.encode(&p)), p);
    let zero = rational("0").unwrap();
    assert!(
        Transform::new(std::array::from_fn(|_| std::array::from_fn(
            |_| zero.clone()
        )))
        .is_err()
    );
}
#[test]
fn calls_currying_zero_and_recursion_share_graph() {
    assert_eq!(
        number("let f = (a,b) => add(multiply(a,2),b) output f(3)(4)"),
        "10"
    );
    assert_eq!(number("let f = (a,b) => add(a,b) output f(0)(4)"), "4");
    assert_eq!(
        number("let f = (a,b) => add(a,b) output add(f(1)(2),f(3)(4))"),
        "10"
    );
    let session = Session::new(
        "let step = (s) => split(s,\"add\") output state(1,1,1)",
        "cycle.ns",
    )
    .unwrap();
    let f = session.function("step").unwrap();
    assert!(Arc::ptr_eq(&f.graph, &session.graph));
    let recursion = Session::new("let f = (x) => f(x) output f(1)", "recursive.ns")
        .unwrap_err()
        .to_string();
    assert!(recursion.contains("call depth"));
}
#[test]
fn observation_is_lazy_portable_and_index_is_not_coordinate() {
    let session = Session::new(
        "let f = (route,s) => split(s,route) output program(state(1,1,1), f(\"add\"))",
        "p.ns",
    )
    .unwrap();
    let Value::Program(p) = session.output else {
        panic!("program")
    };
    let large = num_bigint::BigUint::from(u64::MAX) + num_bigint::BigUint::from(17u8);
    let o = p.observe(large);
    let next = o.successor();
    assert!(Arc::ptr_eq(&o.program, &next.program));
    assert!(o.evaluate(10).is_err());
    let serialized = Value::Observation(o.clone()).to_json().unwrap();
    drop(session.graph);
    drop(p);
    let Value::Observation(restored) = Value::from_json(&serialized).unwrap() else {
        panic!("observation")
    };
    assert_eq!(restored.index, o.index);
    assert_eq!(
        restored
            .program
            .observe(4u32.into())
            .evaluate(4)
            .unwrap()
            .scalar()
            .unwrap()
            .state(),
        o.program
            .observe(4u32.into())
            .evaluate(4)
            .unwrap()
            .scalar()
            .unwrap()
            .state()
    );
    let p2 = Program::new(Value::state(s(1, 1, 1)), restored.program.step.clone()).unwrap();
    let first = p2.observe(0u32.into());
    assert_eq!(
        first.evaluate(0).unwrap().scalar().unwrap().state(),
        &s(1, 1, 1)
    );
}
#[test]
fn reflection_reads_state_program_and_transforms_functions() {
    assert_eq!(number("output reflect(state(2,3,7),state(l,a,m),m)"), "7");
    assert_eq!(
        number("let f = (s) => s output reflect(observe(program(1,f),17),observe(p,k),k)"),
        "17"
    );
    assert_eq!(
        number(
            "let f = (a,b) => add(a,b) let copy = reflect(f,{graph:g,name:n,bindings:b,scope:s},{graph:g,name:n,bindings:b,scope:s}) output copy(3)(4)"
        ),
        "7"
    );
    assert_eq!(
        number(
            "let f = (x) => add(x,1) let g = (x) => multiply(x,2) let copy = reflect(f,{graph:graph,bindings:b,scope:s},{graph:graph,name:\"g\",bindings:b,scope:s}) output copy(5)"
        ),
        "10"
    );
    assert!(Session::new("output phase(1,1)", "old.ns").is_err());
    assert!(Session::new("output index(1,1)", "old.ns").is_err());
}
#[test]
fn camera_change_is_not_state_mutation() {
    let code = "let t = transform([[2,0,0],[0,3,0],[0,0,4]]) ";
    let session = Session::new(
        &format!("{code}output transform(state(1,2,3),t)"),
        "frame.ns",
    )
    .unwrap();
    assert_eq!(session.output.scalar().unwrap().state(), &s(1, 2, 3));
    let transport = Session::new(
        &format!("{code}output add(transform(state(1,2,3),t),state(2,4,5))"),
        "frame.ns",
    )
    .unwrap();
    assert_eq!(
        transport.output.scalar().unwrap().state(),
        &s(1, 2, 3).add(&s(2, 4, 5))
    );
    let mutated =
        Session::new(&format!("{code}output mutate(state(1,2,3),t)"), "mutate.ns").unwrap();
    assert_eq!(mutated.output.scalar().unwrap().state(), &s(2, 6, 12));
}
#[test]
fn history_survives_zero_and_reloading() {
    let a = Session::new("output multiply(7,0)", "a.ns").unwrap().output;
    let b = Session::new("output multiply(100,0)", "b.ns")
        .unwrap()
        .output;
    assert_eq!(a.scalar().unwrap().state(), b.scalar().unwrap().state());
    assert_ne!(a.to_json().unwrap(), b.to_json().unwrap());
    assert_eq!(
        a.to_json().unwrap(),
        Value::from_json(&a.to_json().unwrap())
            .unwrap()
            .to_json()
            .unwrap()
    );
}
#[test]
fn scale_inference_requires_unseen_prediction() {
    let good = optimize::probe([2.0, 1.5, 1.25, 1.125]).unwrap();
    assert_eq!(good.limit, 1.0);
    assert_eq!(good.alpha, 1.0);
    assert_eq!(good.error, 0.0);
    let bad = optimize::probe([2.0, 1.5, 1.25, 99.0]).unwrap();
    assert!(bad.error > 90.0);
    assert!(optimize::probe([1.0, 2.0, 4.0, 8.0]).is_err());
}

#[test]
fn reloaded_graph_obeys_source_binding_rules() {
    let session = Session::new("let f = (x,y) => add(x,y) output f", "graph.ns").unwrap();
    let mut graph = (*session.graph).clone();
    graph.functions.get_mut("f").unwrap().parameters = vec!["x".into(), "x".into()];
    assert!(native_space_language::syntax::validate(&graph).is_err());
    graph.functions.get_mut("f").unwrap().parameters = vec!["phase".into()];
    assert!(native_space_language::syntax::validate(&graph).is_err());
}

#[test]
fn unused_scalar_argument_remains_in_retained_history() {
    let output = Session::new("let f = (x) => 0 output f(7)", "unused.ns")
        .unwrap()
        .output;
    let scalar = output.scalar().unwrap();
    assert_eq!(scalar.state(), &State::zero());
    assert!(
        scalar
            .records()
            .unwrap()
            .iter()
            .any(|r| r.state == s(1, 0, 7))
    );
}
