mod executor;
mod ordering;
mod outcome;
mod report;
mod result;
mod test;
mod test_tree;

pub mod provider;
pub mod settings;

pub use self::{
    executor::{Executor, TestedResult},
    ordering::{Ordering, OrderingCondition},
    outcome::{FailureReason, Outcome},
    report::{Progress, Report},
    result::TestResult,
    settings::Settings,
    test::IsTest,
    test_tree::TestTree,
};
pub use scaffolding_macro::scaffold;
use std::{
    borrow::Cow,
    mem::MaybeUninit,
    sync::{Arc, Mutex, Once},
};

pub type TestName = Cow<'static, str>;

/// create a group of tests and unit tests
#[macro_export]
macro_rules! group {
    (
        $test_name:expr,
        $ordering:expr,
        [
            $($test:expr),+ $(,)?
        ]
    ) => {{
        $crate::TestTree::Group {
            name: $crate::TestName::from($test_name),
            ordering: $ordering,
            tests: ::std::vec![$($test),+]
        }
    }};
    (
        $test_name:expr,
        [
            $($test:expr),+ $(,)?
        ]
    ) => {
        $crate::group! {
            $test_name,
            $crate::Ordering::Any,
            [$($test),+]
        }
    };
}

#[macro_export]
macro_rules! setting {
    ($setting:expr) => {{
        $crate::TestTree::SetSetting {
            value: ::std::sync::Arc::new($setting),
        }
    }};
}

/// create a single test case
///
#[macro_export]
macro_rules! single {
    ($test:expr $(,)?) => {{
        $crate::single!(::std::stringify!($test), $test)
    }};
    ($test_name:expr, $test:expr $(,)?) => {{
        $crate::TestTree::Single {
            name: $crate::TestName::from($test_name),
            test: ::std::boxed::Box::new($test),
        }
    }};
}

#[cfg(feature = "with-quickcheck")]
#[macro_export]
macro_rules! quickcheck {
    ($test:expr => $($sig:tt)* $(,)?) => {{
        $crate::quickcheck!(::std::stringify!($test), $test => $($sig)*)
    }};
    ($test_name:expr, $test:expr => $($sig:tt)* $(,)?) => {{
        $crate::single!(
            $test_name,
            $crate::provider::quickcheck::QC::new(
                $test as $($sig)*
            )
        )
    }};
}

#[cfg(feature = "with-smoke")]
#[macro_export]
macro_rules! smoke {
    ($test:expr $(,)?) => {{
        $crate::smoke!(::std::stringify!($test), $test)
    }};
    ($test_name:expr, $test:expr $(,)?) => {{
        $crate::single!($test_name, $crate::provider::smoke::Smoke::new($test))
    }};
}

static INIT_SCAFFOLDING: Once = Once::new();
static mut SCAFFOLDING: MaybeUninit<Arc<Mutex<Progress<std::io::Stdout>>>> = MaybeUninit::uninit();

pub fn scaffold(tests: TestTree) {
    let scaffolding = unsafe {
        INIT_SCAFFOLDING.call_once(|| {
            SCAFFOLDING = MaybeUninit::new(Arc::new(Mutex::new(Progress::stdout())));
        });

        // we called the `call_once` so it is guaranteed the scaffold is built already
        Arc::clone(SCAFFOLDING.as_ptr().as_ref().unwrap())
    };

    let mut scaffold = scaffolding.lock().unwrap();

    let mut tested = 0usize;
    let mut succeed = 0usize;
    for result in Executor::new(tests) {
        if let TestedResult::Single { result, .. } = &result {
            if !result.is_skipped() {
                tested += 1;
                if result.is_success() {
                    succeed += 1;
                }
            }
        }

        if let Err(error) = scaffold.handle(&result) {
            std::mem::drop(scaffold);

            panic!("Failed to report test's result: {error}", error = error)
        }
    }

    // make sure to drop the scaffold before we
    // get any further with the handling (i.e. don't want to panic with the
    // mutex still locked)
    std::mem::drop(scaffold);

    let fails = tested.saturating_sub(succeed);
    if fails > 0 {
        panic!(
            "failed {fails} out of {tests}",
            fails = fails,
            tests = tested
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[allow(clippy::eq_op, clippy::nonminimal_bool, unreachable_code)]
    #[should_panic]
    #[test]
    fn simpler_test() {
        let tests: TestTree = group!(
            "Tests",
            Ordering::Sequence {
                on_condition: OrderingCondition::Finish
            },
            [
                setting!(settings::Timeout(Duration::from_millis(500))),
                group!(
                    "unit tests",
                    Ordering::Sequence {
                        on_condition: OrderingCondition::Success
                    },
                    [
                        single!(|| { "string".is_ascii() }),
                        single!(|| { true }),
                        single!(|| { (false || true) && true }),
                        single!("return a Result::Ok(())", || {
                            Result::<_, String>::Ok(())
                        }),
                        single!("returning a Result::Err", || {
                            Result::<(), _>::Err("A very complex error occurred and is documented here, maybe now is the right time to wonder about the consequences of it all, the source of the origin of the error that has occurred and reflect on the technical choices that have been made")
                        }),
                        single!("addition", || { 1 + 1 == 2 }),
                    ]
                ),
                group!(
                    "test panics",
                    [
                        single!("sleeping on the job", || {
                            // sleeping on the job
                            std::thread::sleep(std::time::Duration::from_millis(876));
                            // the expected output otherwise
                            true
                        }),
                        setting!(settings::Timeout(Duration::from_millis(1_800))),
                        single!("timing out...", || {
                            // sleeping on the job
                            std::thread::sleep(std::time::Duration::from_secs(3));
                            // the expected output otherwise
                            true
                        }),
                        single!("panicking on error", || {
                            // an error occurred
                            panic!("a panic");
                            // the expected output otherwise
                            true
                        })
                    ]
                ),
            ]
        );

        scaffold(tests);
    }
}
