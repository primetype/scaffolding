use scaffolding::{
    group, scaffold, setting, settings::Timeout, single, Ordering, OrderingCondition, TestTree,
};
use std::{thread::sleep, time::Duration};

#[cfg(feature = "with-quickcheck")]
use scaffolding::{provider::quickcheck, quickcheck};

#[cfg(feature = "with-smoke")]
use ::smoke::{
    forall,
    generator::{num, product2},
    property::greater,
};
#[cfg(feature = "with-smoke")]
use scaffolding::{provider::smoke, smoke};

fn main() {
    let tests = group!(
        "Tests",
        [
            group!(
                "unit tests",
                [
                    some_unit_tests(Ordering::Any),
                    some_unit_tests(Ordering::Sequence {
                        on_condition: OrderingCondition::Finish
                    }),
                    some_unit_tests(Ordering::Sequence {
                        on_condition: OrderingCondition::Success
                    }),
                ]
            ),
            #[cfg(feature = "with-quickcheck")]
            quickcheck_tests(),
            #[cfg(feature = "with-smoke")]
            smoke_tests(),
        ]
    );

    scaffold(tests);
}

fn some_unit_tests(ordering: Ordering) -> TestTree {
    group!(
        format!("Unit tests, ordering with {:?}", ordering),
        ordering,
        [
            single!(|| {}),
            single!(|| { true }),
            single!(|| { false }),
            single!(|| { Result::<_, String>::Ok(()) }),
            single!(|| { Result::<(), _>::Err("error") }),
            single!(|| { panic!("some panics") }),
            single!(|| {
                sleep(Duration::from_millis(100));
            }),
            setting!(Timeout(Duration::from_secs(1))),
            single!(|| {
                sleep(Duration::from_millis(3_000));
            }),
        ]
    )
}

#[cfg(feature = "with-quickcheck")]
fn quickcheck_tests() -> TestTree {
    group!(
        "quickcheck",
        [
            setting!(quickcheck::MaxRetry(10)),
            setting!(quickcheck::WithAtLeast(1)),
            quickcheck!(
                |a: u32| { a.saturating_add(1) > a }
                => fn(_) -> _
            ),
            setting!(quickcheck::MaxRetry(10000)),
            quickcheck!(
                |a: u32, b: u32| { a.saturating_add(b) >= a }
                => fn(_, _) -> _
            ),
            setting!(quickcheck::MaxRetry(10)),
            setting!(quickcheck::WithAtLeast(10)),
            quickcheck!(
                |a: u32, b: u32| { a.saturating_add(b) > a }
                => fn(_, _) -> _
            )
        ]
    )
}

#[cfg(feature = "with-smoke")]
fn smoke_tests() -> TestTree {
    group!(
        "smoke",
        [
            setting!(smoke::MaxRetry(100_000)),
            smoke!(forall(num::<u32>()).ensure(|n| greater(n.saturating_add(1), *n))),
            setting!(smoke::MaxRetry(1_000)),
            smoke!(forall(product2(num::<u32>(), num::<u32>(), curry))
                .ensure(|(a, b)| greater(a.saturating_add(*b), *a))),
        ]
    )
}

fn curry<A, B>(a: A, b: B) -> (A, B) {
    (a, b)
}
