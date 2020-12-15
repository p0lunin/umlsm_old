//! The guards for transitions.
//!
//! Guards - objects which will be called before transition will started. If `Guard` return an `Ok`,
//! transition will started.
//!
//! Example:
//! ```
//! use umlsm::Guard;
//! fn guard_is_odd(num: &i32) -> Result<(), &'static str> {
//!     if num % 2 == 1 { Ok(()) }
//!     else { Err("is not odd!") }
//! }
//!
//! guard_is_odd.check(&1).unwrap();
//! assert_eq!(guard_is_odd.check(&2), Err("is not odd!"));
//! ```
use frunk::{HCons, HNil};

/// An interface for transition guards.
///
/// `Guard` - object which takes `Event` as argument and return `Result<(), Error>`. It will be called
/// when user request `StateMachine::process`. If it return `Ok(())`, transition will start and
/// `StateMachine::process` will return `Error` if `Guard` returns `Err(Error)`.
///
/// More about guards: https://en.wikipedia.org/wiki/UML_state_machine#Guard_conditions
pub trait Guard<Input, Err> {
    fn check(&self, input: &Input) -> Result<(), Err>;
}

impl<Input, F, Err> Guard<Input, Err> for F
where
    F: Fn(&Input) -> Result<(), Err>,
{
    fn check(&self, input: &Input) -> Result<(), Err> {
        self(input)
    }
}

impl<Input, Err> Guard<Input, Err> for HNil {
    fn check(&self, _: &Input) -> Result<(), Err> {
        Ok(())
    }
}

impl<Input, F, Rest, Err> Guard<Input, Err> for HCons<F, Rest>
where
    F: Guard<Input, Err>,
    Rest: Guard<Input, Err>,
{
    fn check(&self, input: &Input) -> Result<(), Err> {
        self.head.check(input).and_then(|_| self.tail.check(input))
    }
}
