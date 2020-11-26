use frunk::coproduct::CNil;
use frunk::Coproduct;
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
