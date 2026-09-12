// SPDX-License-Identifier: AGPL-3.0-or-later
use native_space_language::{
    algebra::{State, Transform, rational},
    runtime::{Session, Value},
};
fn state(l: &str, a: &str, m: &str) -> State {
    State::new(
        rational(l).unwrap(),
        rational(a).unwrap(),
        rational(m).unwrap(),
    )
}
#[test]
fn singular_frames_and_forged_inverses_are_rejected() {
    let matrix = [["1", "2", "3"], ["2", "4", "6"], ["0", "1", "2"]]
        .map(|row| row.map(|v| rational(v).unwrap()));
    assert!(Transform::new(matrix).is_err());
    let original = Transform::identity();
    let mut wire = serde_json::to_value(&original).unwrap();
    wire["inverse"] = serde_json::to_value(
        [["2", "0", "0"], ["0", "1", "0"], ["0", "0", "1"]]
            .map(|row| row.map(|v| rational(v).unwrap())),
    )
    .unwrap();
    assert!(serde_json::from_value::<Transform>(wire).is_err());
}
#[test]
fn local_inverse_checks_canonical_domain_not_local_load() {
    // Swaps L and A. Canonical finite input has local L=0 but remains invertible.
    let prefix = "let t = transform([[0,1,0],[1,0,0],[0,0,1]]) ";
    let output = Session::new(
        &format!("{prefix}output inverse(transform(state(1,0,2),t))"),
        "inverse.ns",
    )
    .unwrap()
    .output;
    assert_eq!(
        output.scalar().unwrap().state(),
        &state("1", "0", "2").inverse().unwrap()
    );
    let Value::Framed { .. } = output else {
        panic!("frame lost")
    };
    // Canonical boundary input has local L=1, which cannot legitimize an inverse.
    assert!(
        Session::new(
            &format!("{prefix}output inverse(transform(state(0,1,2),t))"),
            "boundary.ns"
        )
        .is_err()
    );
}
#[test]
fn projective_boundary_operations_can_leave_the_projective_domain() {
    let p = state("0", "1", "0");
    assert!(p.is_projective_point());
    let result = p.add(&p);
    assert_eq!(result, state("0", "0", "0"));
    assert!(!result.is_projective_point());
    assert!(result.decode().is_err());
    let t = Transform::new(
        [["1", "2", "0"], ["0", "1", "1"], ["1", "0", "1"]]
            .map(|row| row.map(|v| rational(v).unwrap())),
    )
    .unwrap();
    let u = t.encode(&p);
    assert_eq!(t.decode(&t.operations().add(&u, &u)), result);
}
