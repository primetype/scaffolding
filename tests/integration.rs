#![allow(clippy::eq_op, clippy::nonminimal_bool, unreachable_code)]
use scaffolding::{group, scaffold, single, TestTree};

#[scaffold]
fn simpler_test() -> TestTree {
    group!(
        "Tests",
        [
            single!(|| { "string".is_ascii() }),
            single!(|| { true }),
            single!(|| { (false || true) && true }),
            single!("return a Result::Ok(())", || {
                Result::<_, String>::Ok(())
            }),
            single!("addition", || { 1 + 1 == 2 }),
        ]
    )
}
