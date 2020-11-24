use crate::guard::Guard;
use crate::transition::Transition;
use crate::vertex::InitialPseudostate;
use frunk::hlist::{HList, Selector};
use frunk::{HCons, HNil};
use std::marker::PhantomData;

pub struct StateMachine<Current, State, Vertexes, Transitions, TransWithGuards> {
    pub state: State,
    pub vertexes: Vertexes,
    pub trans_with_guards: TransWithGuards,
    pub phantom: PhantomData<(Current, Transitions)>,
}

impl<State> StateMachine<InitialPseudostate, State, HNil, HNil, HNil> {
    pub fn new(state: State) -> Self {
        Self {
            state,
            vertexes: HNil,
            trans_with_guards: HNil,
            phantom: PhantomData,
        }
    }
}
impl<C, State, Vertexes: HList, Transitions, TransWithGuards>
    StateMachine<C, State, Vertexes, Transitions, TransWithGuards>
{
    pub fn add_initial<V>(
        self,
        vertex: V,
    ) -> StateMachine<V, State, HCons<V, Vertexes>, Transitions, TransWithGuards> {
        let StateMachine {
            vertexes,
            trans_with_guards,
            state,
            ..
        } = self;
        StateMachine {
            state,
            vertexes: vertexes.prepend(vertex),
            trans_with_guards,
            phantom: PhantomData,
        }
    }
    pub fn add_vertex<V>(
        self,
        vertex: V,
    ) -> StateMachine<C, State, HCons<V, Vertexes>, Transitions, TransWithGuards> {
        let StateMachine {
            vertexes,
            trans_with_guards,
            state,
            ..
        } = self;
        StateMachine {
            state,
            vertexes: vertexes.prepend(vertex),
            trans_with_guards,
            phantom: PhantomData,
        }
    }
}
impl<C, State, Vertexes, Transitions, TransWithGuards: HList>
    StateMachine<C, State, Vertexes, Transitions, TransWithGuards>
{
    pub fn add_transition<T, G, S, E, Tar, SIdx, TarIdx>(
        self,
        transition: T,
        guard: G,
    ) -> StateMachine<C, State, Vertexes, HCons<T, Transitions>, HCons<(T, G), TransWithGuards>>
    where
        Vertexes: Selector<S, SIdx> + Selector<Tar, TarIdx>,
        T: Transition<S, State, E, Tar>,
        G: Guard<E>,
    {
        let StateMachine {
            vertexes,
            trans_with_guards,
            state,
            ..
        } = self;
        StateMachine {
            state,
            vertexes,
            trans_with_guards: trans_with_guards.prepend((transition, guard)),
            phantom: PhantomData,
        }
    }
}

pub trait ProcessEvent<E, Other>: Sized {
    type ResultOk;
    fn process(self, event: E) -> Result<Self::ResultOk, Self>;
}
