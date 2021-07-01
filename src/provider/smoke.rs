use crate::{settings::Setting, IsTest, Settings, TestResult};
use smoke::{
    ux::{TestResults, TestRunStatus},
    Context, Testable,
};

pub use smoke::Seed;

pub struct Smoke {
    test: Box<dyn Testable + Send + 'static>,
}

impl Smoke {
    pub fn new<T>(smoke: T) -> Self
    where
        T: Testable + Send + 'static,
    {
        Self {
            test: Box::new(smoke),
        }
    }
}

impl Setting for Seed {}
impl IsTest for Smoke {
    fn run(self: Box<Self>, settings: Settings) -> TestResult {
        let mut context = Context::new();

        if let Some(seed) = settings.get::<Seed>() {
            context.set_seed(*seed);
        }

        let results = self.test.test(&context);
        let status = results.to_status();
        let TestResults {
            nb_tests,
            nb_success,
            nb_failed: _,
            nb_skipped: _,
            failures,
            duration,
        } = results;

        let result = match status {
            TestRunStatus::Passed => TestResult::passed(format!("{} tests completed", nb_success)),
            TestRunStatus::Skipped => TestResult::skip(),
            TestRunStatus::Failed => {
                let mut details = String::new();

                details.push_str(&format!(
                    "Only {succeed} successful tests completed out of {total}.\n",
                    succeed = nb_success,
                    total = nb_tests
                ));
                for failure in failures.iter() {
                    details.push_str(failure);
                    details.push('\n');
                }
                TestResult::failed(details)
            }
        };

        TestResult { duration, ..result }
    }
}
