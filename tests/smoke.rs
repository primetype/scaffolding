#![cfg(feature = "with-smoke")]

use scaffolding::{scaffold, smoke, TestTree};
use smoke::{forall, generator::num, property::greater};

#[scaffold]
fn default_settings() -> TestTree {
    smoke!(forall(num::<u32>()).ensure(|n| greater(*n + 1, *n)))
}
