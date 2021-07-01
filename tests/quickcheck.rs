#![cfg(feature = "with-quickcheck")]

use scaffolding::{
    group,
    provider::quickcheck::{MaxRetry, StopAfter, WithAtLeast},
    quickcheck, scaffold, setting, TestTree,
};

fn always_true(a: u64, b: u64) -> bool {
    a.saturating_add(b) >= a
}

fn always_false(a: u64) -> bool {
    a.saturating_add(1) < a
}

#[scaffold]
fn default_settings() -> TestTree {
    quickcheck!(
        always_true
        => fn(_, _) -> _
    )
}

#[scaffold]
fn change_retry() -> TestTree {
    group!(
        "test change of retry",
        [
            setting!(MaxRetry(1)),
            quickcheck!(
                always_true
                => fn(_, _) -> _
            ),
            setting!(MaxRetry(0)),
            quickcheck!(
                always_false
                => fn(_) -> _
            )
        ]
    )
}

#[scaffold]
fn false_but_no_least() -> TestTree {
    group!(
        "Max retry 10 with at least 0 success",
        [
            setting!(MaxRetry(10)),
            setting!(WithAtLeast(0)),
            quickcheck!(
                always_false
                => fn(_) -> _
            ),
        ]
    )
}

#[should_panic]
#[scaffold]
fn not_enough_retry() -> TestTree {
    group!(
        "Max retry 1 with at least 2 success",
        [
            setting!(MaxRetry(1)),
            setting!(WithAtLeast(2)),
            quickcheck!(
                always_true
                => fn(_, _) -> _
            ),
        ]
    )
}

#[should_panic]
#[scaffold]
fn stop_after_one() -> TestTree {
    group!(
        "stop after 1 success but needs at least 2 success",
        [
            setting!(WithAtLeast(2)),
            setting!(StopAfter(1)),
            quickcheck!(
                always_true
                => fn(_, _) -> _
            ),
        ]
    )
}
