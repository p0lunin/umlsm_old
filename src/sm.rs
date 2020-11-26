use crate::guard::Guard;
use crate::hmap::{AppendInner, HMap, HMapGetKeyByCoprod, HMapNil};
use crate::transition::{Action, ITransition, Transition};
use crate::utils::{CoprodWithRef, CoprodWithoutPhantomData};
use crate::vertex::VertexHList;
use frunk::coproduct::{CNil, CoproductEmbedder, CoproductSelector};
use frunk::hlist::HList;
use frunk::{Coproduct, HCons, HNil};
use std::marker::PhantomData;

pub struct StateMachine<Current, State, Transitions, Answer> {
    pub current: Current,
    pub state: State,
    pub transitions: Transitions,
    pub phantom: PhantomData<Answer>,
}

impl<V, State, Answer>
    StateMachine<Coproduct<PhantomData<V>, CNil>, State, HMap<HCons<(V, HNil), HMapNil>>, Answer>
{
    pub fn new(v: V, state: State) -> Self {
        Self {
            current: Coproduct::inject(PhantomData),
            state,
            transitions: HMap::new().add(v, HNil),
            phantom: PhantomData,
        }
    }
}

impl<C, State, Transitions: HList, Answer> StateMachine<C, State, HMap<Transitions>, Answer> {
    pub fn add_vertex<V, Inds>(
        self,
        vertex: V,
    ) -> StateMachine<
        Coproduct<PhantomData<V>, C>,
        State,
        HMap<HCons<(V, HNil), Transitions>>,
        Answer,
    >
    where
        C: CoproductEmbedder<Coproduct<PhantomData<V>, C>, Inds>,
    {
        let StateMachine {
            current,
            state,
            transitions,
            phantom,
        } = self;
        StateMachine {
            current: current.embed(),
            state,
            transitions: transitions.add(vertex, HNil),
            phantom,
        }
    }
    pub fn add_transition<A, G, S, E, Tar, AppendIdx, Out>(
        self,
        action: A,
        guard: G,
        _target: PhantomData<Tar>,
    ) -> StateMachine<C, State, HMap<Out>, Answer>
    where
        Transitions:
            AppendInner<S, Transition<S, State, E, A, G, PhantomData<Tar>, Answer>, AppendIdx, Out>,
        A: Action<S, State, E, Answer>,
        G: Guard<E>,
    {
        let StateMachine {
            current,
            state,
            transitions,
            phantom,
        } = self;
        StateMachine {
            current,
            state,
            transitions: transitions.append_inner(Transition::new(action, guard)),
            phantom,
        }
    }
}

impl<C, State, Transitions, Answer> StateMachine<C, State, Transitions, Answer> {
    pub fn is<T, Idx>(&self) -> bool
    where
        C: CoproductSelector<PhantomData<T>, Idx>,
    {
        self.current.get().is_some()
    }

    pub fn get_current<'a>(
        &'a self,
    ) -> <<C as CoprodWithoutPhantomData>::WithoutPD as CoprodWithRef<'a>>::CoprodWithRef
    where
        C: CoprodWithoutPhantomData,
        C::WithoutPD: CoprodWithRef<'a>,
        Transitions: HMapGetKeyByCoprod<
            'a,
            C,
            <<C as CoprodWithoutPhantomData>::WithoutPD as CoprodWithRef<'a>>::CoprodWithRef,
        >,
    {
        self.transitions.get_by_coprod(&self.current)
    }

    pub fn get_current_as<'a, T, Idx>(&'a self) -> Option<&'a T>
    where
        C: CoprodWithoutPhantomData,
        C::WithoutPD: CoprodWithRef<'a>,
        Transitions: HMapGetKeyByCoprod<
            'a,
            C,
            <<C as CoprodWithoutPhantomData>::WithoutPD as CoprodWithRef<'a>>::CoprodWithRef,
        >,
        <<C as CoprodWithoutPhantomData>::WithoutPD as CoprodWithRef<'a>>::CoprodWithRef:
            CoproductSelector<&'a T, Idx>,
    {
        self.get_current().get().map(|&u| u)
    }
}

pub trait ProcessEvent<E, Answer, Other> {
    fn process(&mut self, event: E) -> Result<Answer, ()>;
}

impl<C, State, Transitions, E, OtherTR, Answer> ProcessEvent<E, Answer, OtherTR>
    for StateMachine<C, State, HMap<Transitions>, Answer>
where
    Transitions: ITransition<C, State, E, C, Answer, OtherTR> + VertexHList<C>,
{
    fn process(&mut self, event: E) -> Result<Answer, ()> {
        let result = self
            .transitions
            .hlist
            .process(&mut self.current, &mut self.state, event)
            .map_err(|_| ());
        match result {
            Ok((answer, mut target)) => {
                self.transitions.hlist.entry(&mut target);
                self.current = target;
                Ok(answer)
            }
            Err(e) => Err(e),
        }
    }
}
