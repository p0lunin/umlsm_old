use frunk::coproduct::CNil;
use frunk::indices::{Here, There};
use frunk::{Coproduct, HCons, HNil};
use std::marker::PhantomData;

pub trait CoprodWithoutPhantomData {
    type WithoutPD;
}

impl CoprodWithoutPhantomData for CNil {
    type WithoutPD = CNil;
}

impl<L, R> CoprodWithoutPhantomData for Coproduct<PhantomData<L>, R>
where
    R: CoprodWithoutPhantomData,
{
    type WithoutPD = Coproduct<L, R::WithoutPD>;
}

pub trait CoprodWithRef<'a> {
    type CoprodWithRef: 'a;
}

impl CoprodWithRef<'_> for CNil {
    type CoprodWithRef = CNil;
}

impl<'a, L, R> CoprodWithRef<'a> for Coproduct<L, R>
where
    L: 'a,
    R: CoprodWithRef<'a> + 'a,
{
    type CoprodWithRef = Coproduct<&'a L, R::CoprodWithRef>;
}

pub trait GetRefsFromCoprod<'a, C> {
    type Out: 'a;
    fn get_refs(&'a self, c: &C) -> Self::Out;
}

impl<'a> GetRefsFromCoprod<'a, CNil> for HNil {
    type Out = CNil;

    fn get_refs(&'a self, c: &CNil) -> Self::Out {
        match *c {}
    }
}

impl<'a, T, CRest, Rest> GetRefsFromCoprod<'a, Coproduct<PhantomData<T>, CRest>> for HCons<T, Rest>
where
    T: 'a,
    Rest: GetRefsFromCoprod<'a, CRest>,
{
    type Out = Coproduct<&'a T, Rest::Out>;

    fn get_refs(&'a self, c: &Coproduct<PhantomData<T>, CRest>) -> Self::Out {
        match c {
            Coproduct::Inl(_) => Coproduct::Inl(&self.head),
            Coproduct::Inr(r) => Coproduct::Inr(self.tail.get_refs(r)),
        }
    }
}

pub trait SelectorPointer<T, Index> {
    fn get_prt(&self) -> *const T;
    fn get_mut_ptr(&mut self) -> *mut T;
}

impl<T, Rest> SelectorPointer<T, Here> for HCons<T, Rest> {
    fn get_prt(&self) -> *const T {
        &self.head
    }

    fn get_mut_ptr(&mut self) -> *mut T {
        &mut self.head
    }
}

impl<U, T, Rest, Idx> SelectorPointer<T, There<Idx>> for HCons<U, Rest>
where
    Rest: SelectorPointer<T, Idx>,
{
    fn get_prt(&self) -> *const T {
        self.tail.get_prt()
    }

    fn get_mut_ptr(&mut self) -> *mut T {
        self.tail.get_mut_ptr()
    }
}
