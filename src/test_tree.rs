use crate::{report::Progress, settings::Setting, Executor, IsTest, Ordering, Report, TestName};
use std::{collections::LinkedList, sync::Arc};

pub enum TestTree {
    Single {
        name: TestName,
        test: Box<dyn IsTest + Send>,
    },
    SetSetting {
        value: Arc<dyn Setting>,
    },
    Group {
        name: TestName,
        ordering: Ordering,
        tests: Vec<TestTree>,
    },
}

pub enum TestItem {
    Single {
        name: TestName,
        test: Box<dyn IsTest + Send>,
    },
    SetSetting {
        value: Arc<dyn Setting>,
    },
    GroupStart {
        name: TestName,
        ordering: Ordering,
    },
    GroupEnd {
        name: TestName,
        ordering: Ordering,
    },
}

enum Either<A, B> {
    A(A),
    B(B),
}

pub struct TreeIterator {
    set: LinkedList<Either<TestTree, TestItem>>,
}

impl TestTree {
    #[inline]
    pub fn into_iterator(self) -> TreeIterator {
        let mut set = LinkedList::new();
        set.push_front(Either::A(self));
        TreeIterator { set }
    }

    /// handy function to compile a report from the given [`TestTree`]
    ///
    /// This function will block until all the tests are executed.
    ///
    #[inline]
    pub fn run(self) -> Report {
        let mut reports = vec![];
        let mut progress = Progress::stdout();
        for result in Executor::new(self) {
            progress.handle(&result).unwrap();
            reports.push(result);
        }

        reports.into_iter().collect()
    }
}

impl Iterator for TreeIterator {
    type Item = TestItem;
    fn next(&mut self) -> Option<Self::Item> {
        match self.set.pop_front()? {
            Either::B(item) => Some(item),
            Either::A(TestTree::Single { name, test }) => Some(TestItem::Single { name, test }),
            Either::A(TestTree::SetSetting { value }) => Some(TestItem::SetSetting { value }),
            Either::A(TestTree::Group {
                name,
                ordering,
                tests,
            }) => {
                self.set.push_front(Either::B(TestItem::GroupEnd {
                    name: name.clone(),
                    ordering,
                }));

                let mut tests: LinkedList<_> = tests.into_iter().map(Either::A).collect();
                tests.append(&mut self.set);
                self.set = tests;

                Some(TestItem::GroupStart { name, ordering })
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.set.len(), None)
    }
}

impl IntoIterator for TestTree {
    type IntoIter = TreeIterator;
    type Item = TestItem;
    fn into_iter(self) -> Self::IntoIter {
        Self::into_iterator(self)
    }
}
