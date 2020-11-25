use frunk::coproduct::CNil;
use frunk::hlist::{h_cons, HList, Selector};
use frunk::indices::{Here, There};
use frunk::{Coproduct, HCons, HNil};
use std::marker::PhantomData;

pub struct HMap<H> {
    pub hlist: H,
}
impl HMap<HNil> {
    pub fn new() -> Self {
        Self { hlist: HNil }
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

impl<Key, Value, Cur, Idx, Rest, Rest_Out> AppendInner<Key, Value, There<Idx>, HCons<Cur, Rest_Out>>
    for HCons<Cur, Rest>
where
    Rest: AppendInner<Key, Value, Idx, Rest_Out>,
    Rest_Out: HList,
{
    fn add(self, value: Value) -> HCons<Cur, Rest_Out> {
        let HCons { head, tail } = self;
        let out_tail = tail.add(value);
        out_tail.prepend(head)
    }
}

pub trait FromHList<H> {
    fn from() -> Self;
}

impl<Left, V> FromHList<HCons<(Left, V), HNil>> for Coproduct<PhantomData<Left>, CNil> {
    fn from() -> Self {
        Self::Inl(PhantomData)
    }
}

impl<Left, V, Middle, VMiddle, Rest, CRest>
    FromHList<HCons<(Left, V), HCons<(Middle, VMiddle), Rest>>>
    for Coproduct<PhantomData<Left>, Coproduct<PhantomData<Middle>, CRest>>
where
    Coproduct<PhantomData<Middle>, CRest>: FromHList<HCons<(Middle, VMiddle), Rest>>,
{
    fn from() -> Self {
        Self::Inr(FromHList::from())
    }
}
