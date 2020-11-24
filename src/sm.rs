use frunk::{HNil, HCons};
use frunk::hlist::{HList, Selector};
use std::marker::PhantomData;
use crate::vertex::{InitialPseudostate, ExitVertex, EntryVertex};
use crate::transition::Transition;
use crate::guard::Guard;

pub struct StateMachine<Current, State, Vertexes, Transitions> {
    current: PhantomData<Current>,
    state: State,
    vertexes: Vertexes,
    transitions: Transitions,
}

impl<State> StateMachine<InitialPseudostate, State, HNil, HNil> {
    pub fn new(state: State) -> Self {
        Self {
            current: PhantomData,
            state,
            vertexes: HNil,
            transitions: HNil
        }
    }
}
impl<C, State, Vertexes: HList, Transitions> StateMachine<C, State, Vertexes, Transitions> {
    pub fn add_initial<V>(self, vertex: V) -> StateMachine<V, State, HCons<V, Vertexes>, Transitions> {
        let StateMachine { vertexes, transitions, state, .. } = self;
        StateMachine {
            current: PhantomData,
            state,
            vertexes: vertexes.prepend(vertex),
            transitions
        }
    }
    pub fn add_vertex<V>(self, vertex: V) -> StateMachine<C, State, HCons<V, Vertexes>, Transitions> {
        let StateMachine { vertexes, transitions, state, .. } = self;
        StateMachine {
            current: PhantomData,
            state,
            vertexes: vertexes.prepend(vertex),
            transitions
        }
    }
}
impl<C, State, Vertexes, Transitions: HList> StateMachine<C, State, Vertexes, Transitions> {
    pub fn add_transition<T, G, S, E, Tar, SIdx, TarIdx>(self, transition: T, guard: G) -> StateMachine<C, State, Vertexes, HCons<(T, G), Transitions>>
    where
        Vertexes: Selector<S, SIdx> + Selector<Tar, TarIdx>,
        T: Transition<S, State, E, Tar>,
        G: Guard<E>,
    {
        let StateMachine { vertexes, transitions, state, .. } = self;
        StateMachine {
            current: PhantomData,
            state,
            vertexes,
            transitions: transitions.prepend((transition, guard))
        }
    }
}

impl<C, State, Vertexes, Transitions> StateMachine<C, State, Vertexes, Transitions> {
    pub fn transition<E, T, CIdx, TIdx, Trans, G, TransIdx>(mut self, event: E) -> Result<StateMachine<T, State, Vertexes, Transitions>, Self>
    where
        Vertexes: Selector<C, CIdx> + Selector<T, TIdx>,
        Transitions: Selector<(Trans, G), TransIdx>,
        Trans: Transition<C, State, E, T>,
        C: ExitVertex,
        T: EntryVertex<E>,
        G: Guard<E>,
    {
        let current: &mut C = self.vertexes.get_mut();
        let (trans, guard) = self.transitions.get_mut();

        if guard.check(&event) {
            current.exit();
            let event = trans.make_transition(current, &mut self.state, event);

            let target: &mut T = self.vertexes.get_mut();
            target.entry(event);

            let StateMachine { state, vertexes, transitions, .. } = self;
            Ok(StateMachine {
                current: PhantomData,
                state,
                vertexes,
                transitions,
            })
        }
        else {
            Err(self)
        }
    }
}
