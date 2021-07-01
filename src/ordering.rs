/// ordering condition
///
/// when executing a group of test, let say in sequence for example,
/// the system will require the test to execute sequentially if
/// the condition succeed:
///
/// * success: means the next test will start only if the test succeed
/// * finished: means the next test will start only upon completion
///
#[derive(Debug, Clone, Copy)]
pub enum OrderingCondition {
    /// set the condition that the previous test needs to have succeed
    /// before the next test starts
    Success,
    /// set the condition that the previous test needs only to complete
    /// (succeed or or not) until the next test starts
    Finish,
}

/// the ordering to execute a group of tests
///
#[derive(Debug, Clone, Copy)]
pub enum Ordering {
    /// Any ordering as appropriate
    ///
    /// The test will be started in non deterministic order,
    /// spawning in threads as appropriate
    Any,
    /// execute only in sequence, based on the ordering condition
    Sequence { on_condition: OrderingCondition },
}
