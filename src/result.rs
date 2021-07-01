use crate::outcome::{FailureReason, Outcome};
use std::{borrow::Cow, fmt, time::Duration};

/// the result of a test
///
/// it contains all the details associated to running the test
/// such as the outcome, the duration and eventually some details
#[derive(Debug)]
pub struct TestResult {
    pub outcome: Outcome,
    pub short: Cow<'static, str>,
    pub details: Cow<'static, str>,
    pub duration: Duration,
}

impl TestResult {
    /// returns `true` if the `outcome` is [`Outcome::Success`]
    ///
    /// this is similar to testing:
    ///
    /// ```
    /// # use scaffolding::{TestResult, Outcome};
    /// # let result = TestResult::passed("");
    ///
    /// # let is_success =
    /// matches!(result.outcome, Outcome::Success);
    /// # assert!(is_success, "test that are _passed_ should have a `Outcome::Success`")
    /// ```
    #[inline]
    pub fn is_success(&self) -> bool {
        self.outcome.is_success()
    }

    #[inline]
    pub fn is_timeout(&self) -> bool {
        self.outcome.is_timeout()
    }

    #[inline]
    pub fn is_skipped(&self) -> bool {
        self.outcome.is_skipped()
    }

    #[inline]
    pub fn is_failure(&self) -> bool {
        self.outcome.is_failure()
    }

    /// create a standard [`TestResult`] for passed test. You can add
    /// some _details_ (a description of what happened), it is okay to
    /// leave it empty though.
    #[inline]
    pub fn passed<D>(details: D) -> Self
    where
        D: Into<Cow<'static, str>>,
    {
        Self {
            outcome: Outcome::Success,
            short: Cow::Borrowed("Success"),
            details: details.into(),
            duration: Duration::from_secs(0),
        }
    }

    /// create a standard [`TestResult`] for failed test. You can add
    /// some _details_ (a description of what happened), it is okay to
    /// leave it empty though.
    #[inline]
    pub fn failed<D>(details: D) -> Self
    where
        D: Into<Cow<'static, str>>,
    {
        Self {
            outcome: Outcome::Failure {
                reason: FailureReason::Failed,
            },
            short: Cow::Borrowed("Failure"),
            details: details.into(),
            duration: Duration::from_secs(0),
        }
    }

    #[inline]
    pub fn skip() -> Self {
        Self {
            outcome: Outcome::Skipped,
            short: Cow::Borrowed("Skipped"),
            details: Cow::Borrowed(""),
            duration: Duration::from_secs(0),
        }
    }

    /// create a standard [`TestResult`] for failed test. You can add
    /// some _details_ (a description of what happened), it is okay to
    /// leave it empty though.
    #[inline]
    pub fn timedout<D>(details: D) -> Self
    where
        D: Into<Cow<'static, str>>,
    {
        Self {
            outcome: Outcome::Failure {
                reason: FailureReason::Timedout,
            },
            short: Cow::Borrowed("Timeout"),
            details: details.into(),
            duration: Duration::from_secs(0),
        }
    }
}

impl From<bool> for TestResult {
    #[inline]
    fn from(condition: bool) -> Self {
        if condition {
            Self::passed("")
        } else {
            Self::failed("")
        }
    }
}

impl<E> From<Result<(), E>> for TestResult
where
    E: fmt::Display,
{
    #[inline]
    fn from(result: Result<(), E>) -> Self {
        if let Err(error) = result {
            Self::failed(error.to_string())
        } else {
            Self::passed("")
        }
    }
}
