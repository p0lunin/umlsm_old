use std::marker::PhantomData;

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
impl<T> ExitVertex for EmptyVertex<T> { }

pub struct InitialPseudoState;
impl ExitVertex for InitialPseudoState {
    fn exit(&mut self) {}
}

pub struct TerminationPseudoState;
impl EntryVertex for TerminationPseudoState {
    fn entry(&mut self) {}
}
