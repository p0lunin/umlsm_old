use crate::hmap::HMapNil;
use frunk::coproduct::CNil;
use frunk::{Coproduct, HCons};
use std::marker::PhantomData;

pub trait Vertex {
    fn entry(&mut self);
    fn exit(&mut self);
}

pub trait VertexHList<C> {
    fn entry(&mut self, c: &mut C);
}

impl VertexHList<CNil> for HMapNil {
    fn entry(&mut self, c: &mut CNil) {
        match *c {}
    }
}

impl<State, Trans, Rest, CRest> VertexHList<Coproduct<PhantomData<State>, CRest>>
    for HCons<(State, Trans), Rest>
where
    State: Vertex,
    Rest: VertexHList<CRest>,
{
    fn entry(&mut self, c: &mut Coproduct<PhantomData<State>, CRest>) {
        match c {
            Coproduct::Inl(_) => self.head.0.entry(),
            Coproduct::Inr(r) => self.tail.entry(r),
        }
    }
}
