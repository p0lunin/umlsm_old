//! Vertex interfaces and instances.

use crate::StateMachine;
use std::marker::PhantomData;

/// An entry point for vertex.
///
/// Called after `Action` and `ExitVertex` for `Target` vertex.
pub trait EntryVertex {
    fn entry(&mut self) {}
}

/// An exit point for vertex.
///
/// Called after `Action` for `Source` vertex.
pub trait ExitVertex {
    fn exit(&mut self) {}
}

/// Action that do nothing.
pub struct EmptyVertex<T>(PhantomData<T>);
impl<T> EmptyVertex<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> EntryVertex for EmptyVertex<T> {}
impl<T> ExitVertex for EmptyVertex<T> {}

/// PseudoState from which `StateMachine` started. Not implement `EntryVertex`, so you cannot
/// transition in this state.
///
/// https://www.uml-diagrams.org/state-machine-diagrams.html#initial-pseudostate
pub struct InitialPseudoState;
impl ExitVertex for InitialPseudoState {
    fn exit(&mut self) {}
}

/// PseudoState from which `StateMachine` cannot leave because it not implement `ExitVertex`.
/// `StateMachineVertex` must transition in this state to leave substate.
///
/// https://www.uml-diagrams.org/state-machine-diagrams.html#terminate-pseudostate
pub struct TerminationPseudoState;
impl EntryVertex for TerminationPseudoState {
    fn entry(&mut self) {}
}

/// Sub state machine or composite state. Used local transitions.
///
/// https://www.uml-diagrams.org/state-machine-diagrams.html#composite-state
/// https://stackoverflow.com/questions/55545971/what-is-different-with-transitions-external-internal-and-local-in-spring-doc-1
pub struct StateMachineVertex<IDX, SM, Entry, Exit> {
    pub(crate) sm: SM,
    pub(crate) entry: Entry,
    pub(crate) exit: Exit,
    pub(crate) phantom: PhantomData<IDX>,
}

impl<
        IDX,
        Current,
        State,
        Vertexes,
        VertHandlers,
        Transitions,
        FAllTransitions,
        Answer,
        GErr,
        Entry,
        Exit,
    >
    StateMachineVertex<
        IDX,
        StateMachine<
            Current,
            State,
            Vertexes,
            VertHandlers,
            Transitions,
            FAllTransitions,
            Answer,
            GErr,
        >,
        Entry,
        Exit,
    >
{
    /// Creates an `StateMachineVertex` with entry and exit points.
    pub fn new(
        sm: StateMachine<
            Current,
            State,
            Vertexes,
            VertHandlers,
            Transitions,
            FAllTransitions,
            Answer,
            GErr,
        >,
        entry: Entry,
        exit: Exit,
    ) -> Self {
        StateMachineVertex {
            sm,
            entry,
            exit,
            phantom: PhantomData,
        }
    }
}

impl<IDX, Current, State, Vertexes, VertHandlers, Transitions, FAllTransitions, Answer, GErr>
    StateMachineVertex<
        IDX,
        StateMachine<
            Current,
            State,
            Vertexes,
            VertHandlers,
            Transitions,
            FAllTransitions,
            Answer,
            GErr,
        >,
        EmptyVertex<()>,
        EmptyVertex<()>,
    >
{
    /// Creates an `StateMachineVertex` without entry and exit points.
    pub fn empty(
        sm: StateMachine<
            Current,
            State,
            Vertexes,
            VertHandlers,
            Transitions,
            FAllTransitions,
            Answer,
            GErr,
        >,
    ) -> Self {
        StateMachineVertex {
            sm,
            entry: EmptyVertex::new(),
            exit: EmptyVertex::new(),
            phantom: PhantomData,
        }
    }
}

impl<
        IDX,
        Current,
        State,
        Vertexes,
        VertHandlers,
        Transitions,
        FAllTransitions,
        Answer,
        GErr,
        Entry,
        Exit,
    > EntryVertex
    for StateMachineVertex<
        IDX,
        StateMachine<
            Current,
            State,
            Vertexes,
            VertHandlers,
            Transitions,
            FAllTransitions,
            Answer,
            GErr,
        >,
        Entry,
        Exit,
    >
where
    Entry: EntryVertex,
{
    fn entry(&mut self) {
        self.entry.entry()
    }
}

impl<
        IDX,
        Current,
        State,
        Vertexes,
        VertHandlers,
        Transitions,
        FAllTransitions,
        Answer,
        GErr,
        Entry,
        Exit,
    > ExitVertex
    for StateMachineVertex<
        IDX,
        StateMachine<
            Current,
            State,
            Vertexes,
            VertHandlers,
            Transitions,
            FAllTransitions,
            Answer,
            GErr,
        >,
        Entry,
        Exit,
    >
where
    Exit: ExitVertex,
{
    fn exit(&mut self) {
        self.exit.exit()
    }
}
