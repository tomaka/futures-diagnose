use std::{cell::RefCell, marker::PhantomData, mem};

/// Returns the context we are currently in.
pub fn current_task() -> CurrentTask {
    CURRENT.with(|v| *v.borrow())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CurrentTask {
    None,
    System,
    Task(u64),
}

pub(crate) struct EnterGuard {
    previous_value: CurrentTask,
    marker: PhantomData<std::rc::Rc<()>>,
}

pub(crate) fn enter(state: CurrentTask) -> EnterGuard {
    let previous_value = CURRENT.with(move |v| mem::replace(&mut *v.borrow_mut(), state));
    EnterGuard {
        previous_value,
        marker: PhantomData,
    }
}

impl Drop for EnterGuard {
    fn drop(&mut self) {
        // TODO: wrong because of mem::forget
        CURRENT.with(move |v| *v.borrow_mut() = self.previous_value);
    }
}

thread_local! {
    static CURRENT: RefCell<CurrentTask> = RefCell::new(CurrentTask::None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(current_task(), CurrentTask::None);
        let _guard = enter(CurrentTask::System);
        assert_eq!(current_task(), CurrentTask::System);
        drop(_guard);
        assert_eq!(current_task(), CurrentTask::None);
    }
}
