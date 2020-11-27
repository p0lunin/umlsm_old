use crate::action::Action;
use crate::guard::Guard;
use crate::hmap::{AppendInner, HMap, HMapNil};
use crate::process_result::{ProcessResult, ProcessResultInner};
use crate::transition::{ITransition, Transition};
use crate::utils::{CoprodWithRef, CoprodWithoutPhantomData, GetRefsFromCoprod};
use crate::vertex::{InitialPseudoState, TerminationPseudoState};
use frunk::coproduct::{CNil, CoproductEmbedder, CoproductSelector};
use frunk::hlist::{h_cons, HList};
use frunk::{Coproduct, HCons, HNil};
use std::marker::PhantomData;

pub struct StateMachine<Current, State, Vertexes, Transitions, Answer> {
    pub current: Current,
    pub state: State,
    pub vertexes: Vertexes,
    pub transitions: Transitions,
    pub phantom: PhantomData<Answer>,
}

impl<State, Answer>
    StateMachine<
        Coproduct<
            PhantomData<InitialPseudoState>,
            Coproduct<PhantomData<TerminationPseudoState>, CNil>,
        >,
        State,
        HCons<InitialPseudoState, HCons<TerminationPseudoState, HNil>>,
        HMap<
            HCons<
                (PhantomData<InitialPseudoState>, HNil),
                HCons<(PhantomData<TerminationPseudoState>, HNil), HMapNil>,
            >,
        >,
        Answer,
    >
{
    pub fn new(state: State) -> Self {
        Self {
            current: Coproduct::inject(PhantomData::<InitialPseudoState>),
            state,
            vertexes: h_cons(InitialPseudoState, h_cons(TerminationPseudoState, HNil)),
            transitions: HMap::new().add(PhantomData, HNil).add(PhantomData, HNil),
            phantom: PhantomData,
        }
    }
}

impl<C, State, Vertexes: HList, Transitions: HList, Answer>
    StateMachine<C, State, Vertexes, HMap<Transitions>, Answer>
{
    pub fn add_vertex<V, Inds>(
        self,
        vertex: V,
    ) -> StateMachine<
        Coproduct<PhantomData<V>, C>,
        State,
        HCons<V, Vertexes>,
        HMap<HCons<(PhantomData<V>, HNil), Transitions>>,
        Answer,
    >
    where
        C: CoproductEmbedder<Coproduct<PhantomData<V>, C>, Inds>,
    {
        let StateMachine {
            current,
            state,
            vertexes,
            transitions,
            phantom,
        } = self;
        StateMachine {
            current: current.embed(),
            state,
            vertexes: vertexes.prepend(vertex),
            transitions: transitions.add(PhantomData, HNil),
            phantom,
        }
    }
    pub fn add_transition<A, G, S, E, Tar, AppendIdx, Out>(
        self,
        action: A,
        guard: G,
        _target: PhantomData<Tar>,
    ) -> StateMachine<C, State, Vertexes, HMap<Out>, Answer>
    where
        Transitions: AppendInner<
            PhantomData<S>,
            Transition<S, State, E, A, G, PhantomData<Tar>, Answer>,
            AppendIdx,
            Out,
        >,
        A: Action<S, State, E, Answer>,
        G: Guard<E>,
    {
        let StateMachine {
            current,
            state,
            vertexes,
            transitions,
            phantom,
        } = self;
        StateMachine {
            current,
            state,
            vertexes,
            transitions: transitions.append_inner(Transition::new(action, guard)),
            phantom,
        }
    }
}

impl<C, State, Vertexes, Transitions, Answer>
    StateMachine<C, State, Vertexes, Transitions, Answer>
{
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
        Vertexes: GetRefsFromCoprod<
            'a,
            C,
            Out = <<C as CoprodWithoutPhantomData>::WithoutPD as CoprodWithRef<'a>>::CoprodWithRef,
        >,
        C: CoprodWithoutPhantomData,
        C::WithoutPD: CoprodWithRef<'a>,
    {
        self.vertexes.get_refs(&self.current)
    }

    pub fn get_current_as<'a, T, Idx>(&'a self) -> Option<&'a T>
    where
        C: CoprodWithoutPhantomData,
        C::WithoutPD: CoprodWithRef<'a>,
        Vertexes: GetRefsFromCoprod<
            'a,
            C,
            Out = <<C as CoprodWithoutPhantomData>::WithoutPD as CoprodWithRef<'a>>::CoprodWithRef,
        >,
        <<C as CoprodWithoutPhantomData>::WithoutPD as CoprodWithRef<'a>>::CoprodWithRef:
            CoproductSelector<&'a T, Idx>,
    {
        self.get_current().get().map(|&u| u)
    }
}

pub trait ProcessEvent<E, Answer, Other> {
    fn process(&mut self, event: &E) -> ProcessResult<Answer>;
}

impl<C, State, Vertexes, Transitions, E, OtherTR, Answer> ProcessEvent<E, Answer, (OtherTR,)>
    for StateMachine<C, State, Vertexes, HMap<Transitions>, Answer>
where
    Transitions: ITransition<C, State, E, C, Vertexes, Answer, OtherTR>,
{
    fn process(&mut self, event: &E) -> ProcessResult<Answer> {
        use ProcessResultInner::*;

        let result = self.transitions.hlist.process(
            &mut self.current,
            &mut self.state,
            &event,
            &mut self.vertexes,
        );
        match result {
            HandledAndProcessEnd((answer, target)) => {
                self.current = target;
                ProcessResult::Handled(answer)
            }
            HandledAndProcessNext => ProcessEvent::process(self, &event),
            NoTransitions => ProcessResult::NoTransitions,
            GuardReturnFalse => ProcessResult::GuardReturnFalse,
        }
    }
}
