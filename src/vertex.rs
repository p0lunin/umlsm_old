use std::marker::PhantomData;
use crate::StateMachine;

pub trait EntryVertex {
    fn entry(&mut self) {}
}

pub trait ExitVertex {
    fn exit(&mut self) {}
}

pub struct EmptyVertex<T>(PhantomData<T>);
impl<T> EmptyVertex<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> EntryVertex for EmptyVertex<T> {
    fn entry(&mut self) {}
}
impl<T> ExitVertex for EmptyVertex<T> {}

pub struct InitialPseudoState;
impl ExitVertex for InitialPseudoState {
    fn exit(&mut self) {}
}

pub struct TerminationPseudoState;
impl EntryVertex for TerminationPseudoState {
    fn entry(&mut self) {}
}

pub struct StateMachineVertex<IDX, SM, Entry, Exit> {
    pub (crate) sm: SM,
    pub (crate) entry: Entry,
    pub (crate) exit: Exit,
    pub (crate) phantom: PhantomData<IDX>,
}

impl<IDX, Current, State, Vertexes, VertHandlers, Transitions, Answer, GErr, Entry, Exit>
StateMachineVertex<IDX, StateMachine<Current, State, Vertexes, VertHandlers, Transitions, Answer, GErr>, Entry, Exit>
{
    pub fn new(sm: StateMachine<Current, State, Vertexes, VertHandlers, Transitions, Answer, GErr>, entry: Entry, exit: Exit) -> Self {
        StateMachineVertex { sm, entry, exit, phantom: PhantomData }
    }
}

impl<IDX, Current, State, Vertexes, VertHandlers, Transitions, Answer, GErr>
StateMachineVertex<IDX, StateMachine<Current, State, Vertexes, VertHandlers, Transitions, Answer, GErr>, EmptyVertex<()>, EmptyVertex<()>>
{
    pub fn empty(sm: StateMachine<Current, State, Vertexes, VertHandlers, Transitions, Answer, GErr>) -> Self {
        StateMachineVertex { sm, entry: EmptyVertex::new(), exit: EmptyVertex::new(), phantom: PhantomData }
    }
}

impl<IDX, Current, State, Vertexes, VertHandlers, Transitions, Answer, GErr, Entry, Exit>
EntryVertex for
StateMachineVertex<IDX, StateMachine<Current, State, Vertexes, VertHandlers, Transitions, Answer, GErr>, Entry, Exit>
where
    Entry: EntryVertex,
{
    fn entry(&mut self) {
        self.entry.entry()
    }
}

impl<IDX, Current, State, Vertexes, VertHandlers, Transitions, Answer, GErr, Entry, Exit>
ExitVertex for
StateMachineVertex<IDX, StateMachine<Current, State, Vertexes, VertHandlers, Transitions, Answer, GErr>, Entry, Exit>
where
    Exit: ExitVertex
{
    fn exit(&mut self) {
        self.exit.exit()
    }
}
