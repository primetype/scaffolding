use crate::{test_tree::TestItem, Ordering, OrderingCondition, Settings, TestName, TestResult};

pub enum TestedResult {
    Single { name: TestName, result: TestResult },
    GroupStart { name: TestName, ordering: Ordering },
    GroupEnd { name: TestName, ordering: Ordering },
}

pub struct Executor<I> {
    tests: I,
    condition: Vec<(Ordering, bool)>,
    settings: Vec<Settings>,
}

impl<I> Executor<I>
where
    I: Iterator<Item = TestItem>,
{
    pub fn new<II>(tests: II) -> Self
    where
        II: IntoIterator<IntoIter = I>,
    {
        Self {
            tests: tests.into_iter(),
            condition: Vec::new(),
            settings: vec![Settings::new()],
        }
    }
}

impl<I> Iterator for Executor<I>
where
    I: Iterator<Item = TestItem>,
{
    type Item = TestedResult;
    fn next(&mut self) -> Option<Self::Item> {
        let test = self.tests.next()?;

        let result = match test {
            TestItem::Single { test, name } => {
                let skip = if let Some((_, failed)) = self.condition.last() {
                    *failed
                } else {
                    false
                };

                if !skip {
                    let settings = self.settings.last().cloned().unwrap_or_default();
                    let result = test.run(settings);

                    if let Some((Ordering::Sequence { on_condition }, failed)) =
                        self.condition.last_mut()
                    {
                        if result.is_failure() && matches!(on_condition, OrderingCondition::Success)
                        {
                            *failed = true;
                        }
                    }

                    TestedResult::Single { result, name }
                } else {
                    TestedResult::Single {
                        result: TestResult::skip(),
                        name,
                    }
                }
            }
            TestItem::GroupStart { name, ordering } => {
                let skip = if let Some((ordering, failed)) = self.condition.last() {
                    match ordering {
                        Ordering::Any => false,
                        Ordering::Sequence { on_condition } => match on_condition {
                            OrderingCondition::Finish => false,
                            OrderingCondition::Success => *failed,
                        },
                    }
                } else {
                    false
                };

                self.condition.push((ordering, skip));
                self.settings
                    .push(self.settings.last().cloned().unwrap_or_default());
                TestedResult::GroupStart { name, ordering }
            }
            TestItem::SetSetting { value } => {
                if let Some(settings) = self.settings.last_mut() {
                    settings.insert_raw(value);

                    // TODO: bad
                    self.next()?
                } else {
                    unreachable!("there should always be at least one setting")
                }
            }
            TestItem::GroupEnd { name, ordering } => {
                let _ = self.condition.pop();
                let _ = self.settings.pop();
                TestedResult::GroupEnd { name, ordering }
            }
        };

        Some(result)
    }
}
