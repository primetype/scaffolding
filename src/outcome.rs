/// reason a test failed
///
/// This is the opportunity to give more details about why
/// a test may have failed.
#[derive(Debug)]
pub enum FailureReason {
    /// the test simply failed to execute.
    Failed,
    /// the test panicked while running
    Panicked,
    /// the test timedout
    Timedout,
}

/// The test outcome
#[derive(Debug)]
pub enum Outcome {
    Success,
    Skipped,
    Failure { reason: FailureReason },
}

impl Outcome {
    /// returns true if the [`Outcome`] is a success
    /// (is [`Outcome::Success`]).
    ///
    /// This is similar to testing `matches!(outcome, Outcome::Success)`
    #[inline]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// returns true if the [`Outcome`] is a skip
    /// (is [`Outcome::Skipped`]).
    ///
    /// This is similar to testing `matches!(outcome, Outcome::Skipped)`
    #[inline]
    pub fn is_skipped(&self) -> bool {
        matches!(self, Self::Skipped)
    }

    /// returns true if the [`Outcome`] is a failure
    /// (is [`Outcome::Failure`]).
    ///
    /// This is similar to testing `matches!(outcome, Outcome::Failure { .. })`
    #[inline]
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failure { .. })
    }

    /// returns true if the [`Outcome`] is a timeout
    /// (is [`Outcome::Failure`]).
    ///
    /// This is similar to testing `matches!(outcome, Outcome::Failure { reason: FailureReason::Timedout })`
    #[inline]
    pub fn is_timeout(&self) -> bool {
        matches!(
            self,
            Self::Failure {
                reason: FailureReason::Timedout
            }
        )
    }
}
