//! Actions which called in `ITransition`.
//!
//! Actions - functions which will be called first when `ITransition` will be started. They can
//! mutate states, do something job, and must return `Answer` which will be returning from
//! `StateMachine` when event process will be ended.
//!
//! ### Action
//! `Action` is an interface for objects which processes `Event`. Full list of args for `Action`:
//! - `Source` - a state from which transition started.
//! - `Ctx` - a global context which stored in `StateMachine`.
//! - `Event` - the event which user give for `StateMachine::process`.
//! - `Target` - a state to which transition will be moved.
//!
//! Still `Action` must return an `Answer` which will be returned to user as an answer from
//! `StateMachine::process`.
//!
//! #### Rules
//! 1. If you want to create `ForallTransition`, you must create `struct` and specify `Source` as
//! generic parameter in `impl Action for YourStruct`.
//! 2. When `Guard` allowed to start `ITransition`, `Action` will be called first. Then will be
//! called `ExitVertex` for `Source` and `EntryVertex` for `Target`.
//! 3. `Answer` must be the same for all transitions in `StateMachine`.
//!
//! #### Implementations
//! `Action` implemented for:
//! - `Fn() -> Answer`
//! - `Fn(&mut Source, &mut Ctx, &Event, &mut Target) -> Answer`
//! - `Fn(&mut Source, &Event) -> Answer`
//! - `Fn(&Event, &mut Target) -> Answer`
//! - `Fn(&mut Source, &Event, &mut Target) -> Answer`
//!
//! #### Examples
//! Simple answer:
//! ```
//! use umlsm::*;
//!
//! fn static_answer() -> &'static str {
//!     "Hello!"
//! }
//!
//! let mut sm = umlsm::state_machine!(
//!     state = (), err = (),
//!     [],
//!     InitialPseudoState + () | static_answer => TerminationPseudoState;
//! );
//! let answer = sm.process(&()).unwrap();
//! assert_eq!(answer, "Hello!");
//! ```
//! For more complicated examples see `examples` directory.
//!
//! ### ActionLoop
//! `Action` is an interface for objects which processes `Event`. It is used only when user define
//! a loop using `loop` keyword in `state_machine!` or `StateMachine::add_loop`. For more information
//! about necessity see `ActionLoop` documentation.
//!
//! Full list of args for `ActionLoop`:
//! - `Vertex` - a state which forms a loop.
//! - `Ctx` - a global context which stored in `StateMachine`.
//! - `Event` - the event which user give for `StateMachine::process`.
//!
//! Still `ActionLoop` must return an `Answer` which will be returned to user as an answer from
//! `StateMachine::process`.
//!
//! #### Rules
//! The same as above.
//!
//! #### Implementations
//! `ActionLoop` implemented for:
//! - `Fn(&mut Vertex, &mut Ctx, &Event) -> Answer`
//!
//! #### Examples
//! Simple answer:
//! ```
//! use umlsm::*;
//!
//! struct State(u32);
//! impl EntryVertex for State {}
//! impl ExitVertex for State {}
//!
//! struct ExitEvent;
//!
//! fn start() -> &'static str { "start" }
//! fn process(state: &mut State, _: &mut (), _: &()) -> &'static str {
//!     state.0 += 1;
//!     "loop"
//! }
//! fn exit() -> &'static str { "exit" }
//!
//! let mut sm = umlsm::state_machine!(
//!     state = (), err = (),
//!     [State(0)],
//!
//!     InitialPseudoState + ()        | start   => State,
//!     State              + ExitEvent | exit    => TerminationPseudoState;
//!     loop: State        + ()        | process;
//! );
//! let answer = sm.process(&()).unwrap();
//! assert_eq!(answer, "start");
//! let answer = sm.process(&()).unwrap();
//! assert_eq!(answer, "loop");
//! let answer = sm.process(&()).unwrap();
//! assert_eq!(answer, "loop");
//! let answer = sm.process(&()).unwrap();
//! assert_eq!(answer, "loop");
//!
//! assert_eq!(sm.get_current_as::<State, _>().unwrap().0, 3);
//!
//! let answer = sm.process(&ExitEvent).unwrap();
//! assert_eq!(answer, "exit");
//! ```
//! For more complicated examples see `examples` directory.

mod action;
mod action_loop;
mod action_loop_wrappers;
mod wrappers;

pub use {
    action::Action,
    action_loop::ActionLoop,
};

#[doc(hidden)]
pub use {
    action_loop_wrappers::{EmptyActionLoop, EmptyForallAction},
    wrappers::{EmptyAction, FnIntoStruct},
};
