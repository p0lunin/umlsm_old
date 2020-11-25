use crate::guard::Guard;
use crate::hmap::{AppendInner, FromHList, HMap, HMapGet};
use crate::transition::{Action, ITransition, Transition};
use frunk::coproduct::{CoprodInjector, CoproductEmbedder};
use frunk::hlist::{h_cons, HList, Selector};
use frunk::indices::Here;
use frunk::{Coprod, Hlist};
use frunk::{Coproduct, HCons, HNil};
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::NonNull;

pub struct StateMachine<Current, State, Transitions> {
    pub current: Current,
    pub state: State,
    pub transitions: Transitions,
}

impl<V, State> StateMachine<Coprod![PhantomData<V>], State, HMap<HCons<(V, HNil), HNil>>> {
    pub fn new(v: V, state: State) -> Self {
        Self {
            current: Coproduct::inject(PhantomData),
            state,
            transitions: HMap::new().add(v, HNil),
        }
    }
}

impl<C, State, Transitions: HList> StateMachine<C, State, HMap<Transitions>> {
    pub fn add_vertex<V, Inds>(
        self,
        vertex: V,
    ) -> StateMachine<Coproduct<PhantomData<V>, C>, State, HMap<HCons<(V, HNil), Transitions>>>
    where
        C: CoproductEmbedder<Coproduct<PhantomData<V>, C>, Inds>,
    {
        let StateMachine {
            current,
            state,
            transitions,
        } = self;
        StateMachine {
            current: current.embed(),
            state,
            transitions: transitions.add(vertex, HNil),
        }
    }
    pub fn add_transition<A, G, S, E, Tar, AppendIdx, Out>(
        self,
        action: A,
        guard: G,
        _target: PhantomData<Tar>,
    ) -> StateMachine<C, State, HMap<Out>>
    where
        Transitions:
            AppendInner<S, Transition<S, State, E, A, G, PhantomData<Tar>>, AppendIdx, Out>,
        A: Action<S, State, E>,
        G: Guard<E>,
    {
        let StateMachine {
            current,
            state,
            transitions,
        } = self;
        StateMachine {
            current,
            state,
            transitions: transitions.append_inner(Transition::new(action, guard)),
        }
    }
}

impl<C, State, Transitions> StateMachine<C, State, HMap<Transitions>> {
    pub fn process<E, Other>(&mut self, event: E) -> Result<(), ()>
    where
        Transitions: ITransition<C, State, E, C, Here, Other>,
    {
        let result = self
            .transitions
            .hlist
            .process(&mut self.current, &mut self.state, event)
            .map(|_| ())
            .map_err(|_| ());
        match result {
            Ok(r) => {
                self.current = self.transitions.hlist.target(&mut self.current);
                Ok(r)
            }
            Err(e) => Err(e),
        }
    }
}
