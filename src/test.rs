use crate::{
    settings::{StackSize, Timeout},
    Outcome, Settings, TestResult,
};
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};

pub trait IsTest {
    fn run(self: Box<Self>, settings: Settings) -> TestResult;
}

impl<R, F> IsTest for F
where
    F: Fn() -> R + Send + 'static,
    R: Into<TestResult>,
{
    #[allow(clippy::mutex_atomic)]
    #[allow(clippy::unnecessary_unwrap)]
    fn run(self: Box<Self>, settings: Settings) -> TestResult {
        let timeout = settings.get_or_default::<Timeout>();
        let duration = Arc::new(Mutex::new(Duration::default()));
        let update_duration = Arc::clone(&duration);
        let cond = Arc::new((Mutex::new(false), Condvar::new()));
        let notify = Arc::clone(&cond);

        let thread = thread::Builder::new().stack_size(settings.get_or_default::<StackSize>().0);

        let backup_instant = Instant::now();
        let thread = thread
            .spawn(move || {
                let instant = Instant::now();
                let result = catch_unwind(AssertUnwindSafe(|| self().into()));
                *update_duration.lock().unwrap() = instant.elapsed();
                let (lock, cvar) = &*notify;
                let mut finished = lock.lock().unwrap();
                *finished = true;
                cvar.notify_one();
                result
            })
            .unwrap();
        let (lock, cvar) = &*cond;
        let result = cvar
            .wait_timeout_while(lock.lock().unwrap(), timeout.0, |pending| !*pending)
            .unwrap();
        let duration = *duration.lock().unwrap();
        let duration = if duration == Duration::default() {
            backup_instant.elapsed()
        } else {
            duration
        };

        let result = if result.1.timed_out() {
            TestResult::timedout(format!(
                "Test timedout, allocated duration was {:.2?}",
                timeout.0
            ))
        } else {
            let result = thread.join();

            match result {
                Err(_error) | Ok(Err(_error)) => TestResult {
                    outcome: Outcome::Failure {
                        reason: crate::FailureReason::Panicked,
                    },
                    ..TestResult::failed("test panicked")
                },
                Ok(Ok(result)) => result,
            }
        };

        TestResult { duration, ..result }
    }
}
