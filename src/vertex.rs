use std::marker::PhantomData;

pub trait EntryVertex<Event> {
    fn entry(&mut self, event: &Event) {}
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

impl<E, T> EntryVertex<E> for EmptyVertex<T> {
    fn entry(&mut self, _: &E) {}
}
impl<T> ExitVertex for EmptyVertex<T> { }

pub struct InitialPseudoState;
impl ExitVertex for InitialPseudoState {
    fn exit(&mut self) {}
}

pub struct TerminationPseudoState;
impl<E> EntryVertex<E> for TerminationPseudoState {
    fn entry(&mut self, _: &E) {}
}
