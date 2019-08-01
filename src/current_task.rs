use std::marker::PhantomData;

/// Returns the context we are currently in.
pub fn current_task() -> CurrentTask {
    CURRENT.with(|v| *v)
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
    debug_assert_ne!(state, CurrentTask::None);
    unimplemented!()
}

thread_local! {
    static CURRENT: CurrentTask = CurrentTask::None;
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
