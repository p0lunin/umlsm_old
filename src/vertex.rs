use std::marker::PhantomData;

pub trait EntryVertex<Event> {
    fn entry(&mut self, event: &Event);
}

pub trait ExitVertex<Event> {
    fn exit(&mut self, event: &Event);
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
impl<E, T> ExitVertex<E> for EmptyVertex<T> {
    fn exit(&mut self, _: &E) {}
}
