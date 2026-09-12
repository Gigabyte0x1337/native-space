// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic generative properties. Fixed seeds and case numbers make failures replayable.
use native_space_language::{
    algebra::{Rational, Split, State, Transform, pow2},
    runtime::{Program, Session, Value},
};
use num_bigint::{BigInt, BigUint};
use num_traits::{One, Zero};
use std::sync::Arc;

// SplitMix64 is test data generation only, never a discovery/production primitive.
struct Cases(u64);
impl Cases {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    fn rational(&mut self) -> Rational {
        Rational::new(
            BigInt::from((self.next() % 35) as i64 - 17),
            BigInt::from(self.next() % 7 + 1),
        )
    }
    fn state(&mut self) -> State {
        State::new(self.rational(), self.rational(), self.rational())
    }
    fn nonzero(&mut self) -> Rational {
        let r = self.rational();
        if r.is_zero() { Rational::one() } else { r }
    }
    fn frame(&mut self) -> Transform {
        let zero = Rational::zero();
        let mut rows = [
            [self.nonzero(), self.rational(), self.rational()],
            [zero.clone(), self.nonzero(), self.rational()],
            [zero.clone(), zero, self.nonzero()],
        ];
        rows.rotate_left((self.next() % 3) as usize);
        Transform::new(rows).unwrap()
    }
}

#[test]
fn rational_algebra_properties_6144_cases() {
    for seed in [0x4c414d, 0x62726f6164, 0xdeadbeef] {
        let mut cases = Cases(seed);
        for case in 0..2048 {
            let (p, q, s) = (cases.state(), cases.state(), cases.state());
            assert_eq!(p.add(&q), q.add(&p), "ADD seed {seed} case {case}");
            assert_eq!(
                p.multiply(&q),
                q.multiply(&p),
                "MULTIPLY seed {seed} case {case}"
            );
            assert_eq!(
                p.add(&q).add(&s),
                p.add(&q.add(&s)),
                "ADD associativity {seed}/{case}"
            );
            assert_eq!(
                p.multiply(&q).multiply(&s),
                p.multiply(&q.multiply(&s)),
                "MULTIPLY associativity {seed}/{case}"
            );
            let c = cases.nonzero();
            let scaled = p.rescale(&c).unwrap();
            assert_eq!(
                scaled.rescale(&c.recip()).unwrap(),
                p,
                "scale {seed}/{case}"
            );
            assert_eq!(p.projective_equivalent(&scaled), p.is_projective_point());
            for route in [Split::Add, Split::Multiply] {
                assert_eq!(
                    p.split(route).total(),
                    p.total(),
                    "split total {seed}/{case}"
                );
            }
            if let Ok((r, x)) = p.decode() {
                assert_eq!(scaled.decode().unwrap(), (r.clone(), x.clone()));
                assert_eq!(p.negate().decode().unwrap(), (-&r, -&x));
                for (route, offset) in [(Split::Add, 0), (Split::Multiply, 1)] {
                    assert_eq!(
                        p.split(route).decode().unwrap(),
                        (
                            &r * Rational::from_integer(2.into()) + Rational::one(),
                            &x * Rational::from_integer(2.into())
                                + Rational::from_integer(offset.into())
                        )
                    );
                }
                if !r.is_zero() && !x.is_zero() {
                    assert_eq!(
                        p.inverse().unwrap().decode().unwrap(),
                        (r.recip(), x.recip())
                    );
                } else {
                    assert!(p.inverse().is_err());
                }
                if let Ok((t, y)) = q.decode() {
                    let (r, x) = p.decode().unwrap();
                    assert_eq!(p.add(&q).decode().unwrap(), (&r + &t, &x + &y));
                    assert_eq!(p.multiply(&q).decode().unwrap(), (&r * &t, &x * &y));
                    if s.is_finite() {
                        let a = p.multiply(&q.add(&s));
                        let b = p.multiply(&q).add(&p.multiply(&s));
                        assert!(
                            a.projective_equivalent(&b),
                            "projective distributivity {seed}/{case}"
                        );
                        assert_eq!(a.decode().unwrap(), b.decode().unwrap());
                    }
                }
            }
        }
    }
}

#[test]
fn transported_and_mixed_frame_properties_2048_cases() {
    let mut cases = Cases(0x6672616d65);
    for case in 0..2048 {
        let (p, q) = (cases.state(), cases.state());
        let (t, s) = (cases.frame(), cases.frame());
        let u = t.encode(&p);
        let v = t.encode(&q);
        let from_s = t.reframe(&s.encode(&q), &s);
        assert_eq!(from_s, v, "mixed basis case {case}");
        assert_eq!(t.decode(&u), p);
        assert_eq!(t.inverse().encode(&u), p);
        assert_eq!(s.compose(&t).encode(&p), s.encode(&t.encode(&p)));
        assert_eq!(s.compose(&t).inverse(), t.inverse().compose(&s.inverse()));
        assert_eq!(Transform::identity().encode(&p), p);
        let ops = t.operations();
        assert!(std::ptr::eq(ops, t.operations()), "cached coefficients");
        assert!(
            std::ptr::eq(ops, t.clone().operations()),
            "clone shares coefficients"
        );
        assert_eq!(ops.add(&u, &v), t.add_reference(&u, &v));
        assert_eq!(ops.multiply(&u, &v), t.multiply_reference(&u, &v));
        assert_eq!(t.decode(&ops.add(&u, &from_s)), p.add(&q));
        assert_eq!(t.decode(&ops.multiply(&u, &from_s)), p.multiply(&q));
        assert_eq!(t.decode(&ops.negate(&u)), p.negate());
        for route in [Split::Add, Split::Multiply] {
            assert_eq!(t.decode(&ops.split(&u, route)), p.split(route));
        }
        match p.inverse() {
            Ok(inverse) => assert_eq!(t.decode(&t.inverse_state(&u).unwrap()), inverse),
            Err(_) => assert!(t.inverse_state(&u).is_err()),
        }
        if case % 64 == 0 {
            let wire = serde_json::to_string(&t).unwrap();
            let restored: Transform = serde_json::from_str(&wire).unwrap();
            assert_eq!(restored, t, "cache not part of equality");
            assert_eq!(restored.operations().add(&u, &v), ops.add(&u, &v));
        }
    }
}

fn source(p: &State) -> String {
    format!("state({},{},{})", p.l, p.a, p.m)
}
fn matrix(t: &Transform) -> String {
    format!(
        "[{}]",
        t.matrix()
            .iter()
            .map(|r| format!("[{},{},{}]", r[0], r[1], r[2]))
            .collect::<Vec<_>>()
            .join(",")
    )
}
#[test]
fn runtime_uses_transported_arithmetic_128_cases() {
    let mut cases = Cases(0x72756e);
    for case in 0..128 {
        let (p, q) = (cases.state(), cases.state());
        let (t, s) = (cases.frame(), cases.frame());
        let prefix = format!(
            "let t = transform({}) let s = transform({}) let p = {} let q = {} ",
            matrix(&t),
            matrix(&s),
            source(&p),
            source(&q)
        );
        for (expression, expected) in [
            ("add(transform(p,t),transform(q,s))", p.add(&q)),
            ("multiply(transform(p,t),transform(q,s))", p.multiply(&q)),
            ("add(p,transform(q,t))", p.add(&q)),
            ("multiply(transform(p,t),q)", p.multiply(&q)),
            ("negate(transform(p,t))", p.negate()),
            ("split(transform(p,t),\"add\")", p.split(Split::Add)),
            (
                "split(transform(p,t),\"multiply\")",
                p.split(Split::Multiply),
            ),
        ] {
            let output = Session::new(&format!("{prefix}output {expression}"), "property.ns")
                .unwrap()
                .output;
            assert_eq!(
                output.scalar().unwrap().state(),
                &expected,
                "case {case}: {expression}"
            );
            let Value::Framed { frame, local } = output else {
                panic!("frame lost")
            };
            assert_eq!(frame, t);
            assert_eq!(local.state(), &t.encode(&expected));
            assert_ne!(local.0.operation, "encode", "arithmetic runs locally");
        }
    }
}

#[test]
fn program_generator_is_shared_for_arbitrary_observations_1024_cases() {
    let session = Session::new("let step = (p) => negate(p) output step", "program.ns").unwrap();
    let step = session.function("step").unwrap();
    let mut cases = Cases(0x6f627365727665);
    for _ in 0..1024 {
        let seed = cases.state();
        let p = Program::new(Value::state(seed.clone()), step.clone()).unwrap();
        let k = cases.next() % 16;
        let o = p.observe(k.into());
        let expected = if k % 2 == 0 {
            seed.clone()
        } else {
            seed.negate()
        };
        assert_eq!(o.evaluate(k).unwrap().scalar().unwrap().state(), &expected);
        assert!(Arc::ptr_eq(&o.program.step.graph, &session.graph));
        let huge = (BigUint::one() << 100usize) + BigUint::from(cases.next());
        let selected = p.observe(huge.clone());
        let next = selected.successor();
        assert_eq!(next.index, &huge + BigUint::one());
        assert!(Arc::ptr_eq(&selected.program, &next.program));
        assert!(Arc::ptr_eq(&selected.program.step.graph, &session.graph));
        assert_eq!(selected.program.seed.scalar().unwrap().state(), &seed);
    }
}

#[test]
fn exact_extreme_scale_and_projective_boundary() {
    let r = |v: i64| Rational::from_integer(v.into());
    use native_space_language::{
        camera,
        optimize::{Balance, balanced_frame_with},
    };
    let p = State::new(r(1), r(2), r(3));
    for k in [10_000, -10_000, 0, 1, -1] {
        let huge = p.rescale_pow2(k);
        assert_eq!(huge.rescale_pow2(-k), p);
        assert!(p.projective_equivalent(&huge));
        assert_eq!(huge.decode().unwrap(), p.decode().unwrap());
        let t = balanced_frame_with(&huge, Balance::PowerOfTwo).unwrap();
        let local = t.encode(&huge);
        assert_eq!(t.decode(&local), huge);
        assert!(local.coordinates().iter().all(|v| v <= &Rational::one()));
        assert!(local.m > pow2(-1));
    }
    let boundary = State::new(r(0), r(2), r(-3));
    assert!(boundary.is_boundary());
    assert!(boundary.is_projective_point());
    assert!(!boundary.is_finite());
    assert!(boundary.decode().is_err());
    assert!(boundary.projective_equivalent(&boundary.rescale(&r(-2)).unwrap()));
    assert!(!boundary.projective_equivalent(&State::new(r(0), r(2), r(3))));
    let raw_zero = State::new(r(0), r(0), r(0));
    assert!(!raw_zero.is_projective_point());
    assert!(!raw_zero.is_boundary());
    assert!(!raw_zero.projective_equivalent(&raw_zero));
    assert!(!raw_zero.projective_equivalent(&boundary));
    let serialized = Value::state(raw_zero.clone()).to_json().unwrap();
    assert_eq!(
        Value::from_json(&serialized)
            .unwrap()
            .scalar()
            .unwrap()
            .state(),
        &raw_zero
    );
    let total_zero = State::new(r(1), r(-2), r(1));
    assert!(total_zero.simplex().is_err());
    assert!(total_zero.decode().is_ok());
    assert!(camera::raw(&total_zero).is_ok());
    for divisor in [State::new(r(1), r(2), r(0)), State::new(r(1), r(-2), r(2))] {
        assert!(divisor.decode().is_ok());
        assert!(divisor.inverse().is_err());
        assert!(divisor.is_projective_point());
    }
    // Both factors represent nonzero elements of Q x Q; their product is zero.
    let a = State::new(r(1), r(1), r(0));
    let b = State::new(r(1), r(-1), r(1));
    assert_eq!(a.multiply(&b), State::zero());
}
