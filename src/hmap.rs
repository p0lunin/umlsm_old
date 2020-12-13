use crate::utils::{CoprodWithRef, CoprodWithoutPhantomData};
use crate::TerminationPseudoState;
use frunk::coproduct::CNil;
use frunk::hlist::{h_cons, HList, Selector};
use frunk::indices::{Here, There};
use frunk::{Coproduct, HCons, HNil};
use std::marker::PhantomData;

pub struct HMap<H> {
    pub hlist: H,
}
impl HMap<HMapNil> {
    pub fn new() -> Self {
        Self { hlist: HMapNil }
    }
}

pub trait HMapGet<'a, Key, Value, Idx> {
    fn get(&'a self) -> &'a Value;
}

impl<'a, Key, Value, Idx, H> HMapGet<'a, Key, Value, Idx> for HMap<H>
where
    Key: 'a,
    H: Selector<(Key, Value), Idx>,
{
    fn get(&'a self) -> &'a Value {
        let (_, v) = self.hlist.get();
        v
    }
}

pub trait HMapGetKeyByCoprod<'a, C, CRes> {
    fn get_by_coprod(&'a self, c: &C) -> CRes;
}

impl<'a, C, H>
    HMapGetKeyByCoprod<
        'a,
        C,
        <<C as CoprodWithoutPhantomData>::WithoutPD as CoprodWithRef<'a>>::CoprodWithRef,
    > for HMap<H>
where
    C: CoprodWithoutPhantomData,
    C::WithoutPD: CoprodWithRef<'a>,
    <<C as CoprodWithoutPhantomData>::WithoutPD as CoprodWithRef<'a>>::CoprodWithRef: 'a,
    H: HMapGetKeyByCoprod<
        'a,
        C,
        <<C as CoprodWithoutPhantomData>::WithoutPD as CoprodWithRef<'a>>::CoprodWithRef,
    >,
{
    fn get_by_coprod<'b>(
        &'a self,
        c: &'b C,
    ) -> <<C as CoprodWithoutPhantomData>::WithoutPD as CoprodWithRef<'a>>::CoprodWithRef {
        self.hlist.get_by_coprod(c)
    }
}

impl<'a> HMapGetKeyByCoprod<'a, CNil, CNil> for HMapNil {
    fn get_by_coprod(&'a self, c: &CNil) -> CNil {
        match *c {}
    }
}

impl<'a, C, V, CRest, CResRest, HRest>
    HMapGetKeyByCoprod<'a, Coproduct<PhantomData<C>, CRest>, Coproduct<&'a C, CResRest>>
    for HCons<(C, V), HRest>
where
    HRest: HMapGetKeyByCoprod<'a, CRest, CResRest>,
{
    fn get_by_coprod(&'a self, c: &Coproduct<PhantomData<C>, CRest>) -> Coproduct<&'a C, CResRest> {
        match c {
            Coproduct::Inl(_) => Coproduct::Inl(&self.head.0),
            Coproduct::Inr(r) => Coproduct::Inr(self.tail.get_by_coprod(r)),
        }
    }
}

impl<H> HMap<H> {
    pub fn add<Key, Value>(self, key: Key, value: Value) -> HMap<HCons<(Key, Value), H>>
    where
        H: HList,
    {
        let HMap { hlist } = self;
        HMap {
            hlist: h_cons((key, value), hlist),
        }
    }

    pub fn get_key<'a, Key, Value, Idx>(&'a self) -> &'a Key
    where
        Value: 'a,
        H: Selector<(Key, Value), Idx>,
    {
        let (k, _) = self.hlist.get();
        k
    }
    pub fn get<'a, Key, Value, Idx>(&'a self) -> &'a Value
    where
        Key: 'a,
        H: Selector<(Key, Value), Idx>,
    {
        let (_, v) = self.hlist.get();
        v
    }
    pub fn get_pair<Key, Value, Idx>(&self) -> &(Key, Value)
    where
        H: Selector<(Key, Value), Idx>,
    {
        self.hlist.get()
    }
}

impl<H> HMap<H> {
    pub fn append_inner<Key, Value, Idx, Out>(self, value: Value) -> HMap<Out>
    where
        H: AppendInner<Key, Value, Idx, Out>,
    {
        let HMap { hlist } = self;
        HMap {
            hlist: hlist.add(value),
        }
    }
}

pub trait AppendInner<Key, Value, Idx, Out> {
    fn add(self, value: Value) -> Out;
}

impl<Key, Value, ValueOld: HList, Rest: HList>
    AppendInner<Key, Value, Here, HCons<(Key, HCons<Value, ValueOld>), Rest>>
    for HCons<(Key, ValueOld), Rest>
{
    fn add(self, value: Value) -> HCons<(Key, HCons<Value, ValueOld>), Rest> {
        let HCons { head: (k, v), tail } = self;
        tail.prepend((k, v.prepend(value)))
    }
}

impl<Key, Value, Cur, Idx, Rest, RestOut> AppendInner<Key, Value, There<Idx>, HCons<Cur, RestOut>>
    for HCons<Cur, Rest>
where
    Rest: AppendInner<Key, Value, Idx, RestOut>,
    RestOut: HList,
{
    fn add(self, value: Value) -> HCons<Cur, RestOut> {
        let HCons { head, tail } = self;
        let out_tail = tail.add(value);
        out_tail.prepend(head)
    }
}

pub trait KeySelector<T, Idx> {
    fn select_by_key(&mut self) -> &mut T;
}

impl<K, V, Rest> KeySelector<K, Here> for HCons<(K, V), Rest> {
    fn select_by_key(&mut self) -> &mut K {
        &mut self.head.0
    }
}

impl<T, This, Idx, Rest> KeySelector<T, There<Idx>> for HCons<This, Rest>
where
    Rest: KeySelector<T, Idx>,
{
    fn select_by_key(&mut self) -> &mut T {
        self.tail.select_by_key()
    }
}

pub struct HMapNil;
impl HList for HMapNil {
    const LEN: usize = 0;

    fn static_len() -> usize {
        0
    }
}
