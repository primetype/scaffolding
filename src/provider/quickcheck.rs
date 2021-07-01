use crate::{settings::Setting, IsTest, Settings, TestResult};
use std::time::Instant;

pub struct QC {
    test: Box<dyn Fn(MaxRetry, StopAfter) -> Result<u64, quickcheck::TestResult> + Send + 'static>,
}

#[derive(Debug, Clone, Copy)]
pub struct StopAfter(pub u64);
impl Setting for StopAfter {}
impl Default for StopAfter {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MaxRetry(pub u64);
impl Setting for MaxRetry {}
impl Default for MaxRetry {
    fn default() -> Self {
        Self(10_000)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WithAtLeast(pub u64);
impl Setting for WithAtLeast {}

impl QC {
    pub fn new<A>(f: A) -> Self
    where
        A: quickcheck::Testable + Send + 'static + Copy,
    {
        Self {
            test: Box::new(
                move |MaxRetry(max_retry): MaxRetry, StopAfter(stop_after): StopAfter| {
                    //
                    quickcheck::QuickCheck::new()
                        .max_tests(max_retry)
                        .tests(stop_after)
                        .quicktest(f)
                },
            ),
        }
    }
}

impl IsTest for QC {
    fn run(self: Box<Self>, settings: Settings) -> TestResult {
        let instant = Instant::now();

        let max_retry = *settings.get_or_default::<MaxRetry>();
        let stop_after = *settings.get_or_default::<StopAfter>();
        let WithAtLeast(with_at_least) = *settings.get_or_default::<WithAtLeast>();

        let result = (*self.test)(max_retry, stop_after);
        let duration = instant.elapsed();

        let result = match result {
            Ok(number) if number >= with_at_least => {
                TestResult::passed(format!("{} tests completed", number))
            }
            Ok(number) => TestResult::failed(format!(
                "Only {succeed} successful tests completed out of {total}",
                succeed = number,
                total = with_at_least
            )),
            Err(error) => error.into(),
        };

        TestResult { duration, ..result }
    }
}

impl From<quickcheck::TestResult> for TestResult {
    fn from(qc_result: quickcheck::TestResult) -> Self {
        if qc_result.is_error() {
            TestResult::failed(format!("{:?}", qc_result))
        } else {
            TestResult::passed("")
        }
    }
}
