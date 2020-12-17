//! An interface for processing events.
//!
//! Any `StateMachine` can process any type of `Event`. If you got an error that `StateMachine`
//! cannot process an event, this is a bug.
//!
//! `ProcessEvent::process` returns an `ProcessResult`.

use crate::ProcessResult;

/// An interface for processing events.
///
/// For more information, see `module-level documentation`.
pub trait ProcessEvent<E, Answer, GErr, Other> {
    fn process(&mut self, event: &E) -> ProcessResult<Answer, GErr>;
}
