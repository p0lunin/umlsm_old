//! Main struct that encapsulates states and transitions.
//!
//! `StateMachine` store itself:
//! - Current vertex.
//! - Global mutable state.
//! - Array of vertices.
//! - Array of transitions.
//!
//! By default, `StateMachine::new` creates an empty state machine with 2 vertices:
//! - `InitialPseudoState`
//! - `TerminationPseudoState`
//!
//! For initializing the `StateMachine` we recommend use the `state_machine!` macro.

use crate::action::{Action, ActionLoop, FnIntoStruct};
use crate::guard::Guard;
use crate::hmap::{AppendInner, HMap, HMapNil};
use crate::process_result::{ProcessResult, ProcessResultInner, ProcessResultSubstate};
use crate::transition::{
    ForallTransition, ITransition, LoopTransition, ProcessByForallTransitions, Transition,
};
use crate::utils::{CoprodWithRef, CoprodWithoutPhantomData, GetRefsFromCoprod};
use crate::vert_handler::{EmptyVertexHandler, VertexHandler};
use crate::vertex::{InitialPseudoState, TerminationPseudoState};
use crate::ProcessEvent;
use frunk::coproduct::{CNil, CoproductEmbedder, CoproductSelector};
use frunk::hlist::{h_cons, HList, Selector};
use frunk::{Coproduct, HCons, HNil};
use std::any::TypeId;
use std::marker::PhantomData;

/// Main struct that encapsulates states and transitions.
///
/// For more information see `module-level documentation`.
pub struct StateMachine<
    Current,
    State,
    Vertexes,
    VertHandlers,
    Transitions,
    FAllTrans,
    Answer,
    GErr,
> {
    pub current: Current,
    pub state: State,
    pub vertexes: Vertexes,
    pub vertices_handlers: VertHandlers,
    pub transitions: Transitions,
    pub forall_transitions: FAllTrans,
    pub phantom: PhantomData<(Answer, GErr)>,
}

impl<State, Answer, GErr>
    StateMachine<
        Coproduct<
            PhantomData<InitialPseudoState>,
            Coproduct<PhantomData<TerminationPseudoState>, CNil>,
        >,
        State,
        HCons<InitialPseudoState, HCons<TerminationPseudoState, HNil>>,
        HCons<EmptyVertexHandler, HCons<EmptyVertexHandler, HNil>>,
        HMap<
            HCons<
                (PhantomData<InitialPseudoState>, HNil),
                HCons<(PhantomData<TerminationPseudoState>, HNil), HMapNil>,
            >,
        >,
        HNil,
        Answer,
        GErr,
    >
{
    /// Initialize an empty state machine with specified global state.
    pub fn new(state: State) -> Self {
        Self {
            current: Coproduct::inject(PhantomData::<InitialPseudoState>),
            state,
            vertexes: h_cons(InitialPseudoState, h_cons(TerminationPseudoState, HNil)),
            vertices_handlers: h_cons(EmptyVertexHandler, h_cons(EmptyVertexHandler, HNil)),
            transitions: HMap::new().add(PhantomData, HNil).add(PhantomData, HNil),
            forall_transitions: HNil,
            phantom: PhantomData,
        }
    }
}

impl<
        C,
        State,
        Vertexes: HList,
        VertHandlers: HList,
        Transitions: HList,
        FAllTransitions: HList,
        Answer,
        GErr,
    >
    StateMachine<C, State, Vertexes, VertHandlers, HMap<Transitions>, FAllTransitions, Answer, GErr>
{
    /// Add an `Vertex` for state machine with specified handler.
    ///
    /// More about vertices see in `umlsm::vertex` module.
    /// More about vertex handlers see in `umlsm::vert_handler` module.
    pub fn add_vertex<V, VertHandler, Inds>(
        self,
        vertex: V,
        vertex_handler: VertHandler,
    ) -> StateMachine<
        Coproduct<PhantomData<V>, C>,
        State,
        HCons<V, Vertexes>,
        HCons<VertHandler, VertHandlers>,
        HMap<HCons<(PhantomData<V>, HNil), Transitions>>,
        FAllTransitions,
        Answer,
        GErr,
    >
    where
        C: CoproductEmbedder<Coproduct<PhantomData<V>, C>, Inds>,
    {
        let StateMachine {
            current,
            state,
            vertexes,
            vertices_handlers,
            transitions,
            forall_transitions,
            phantom,
        } = self;
        StateMachine {
            current: current.embed(),
            state,
            vertexes: vertexes.prepend(vertex),
            vertices_handlers: vertices_handlers.prepend(vertex_handler),
            transitions: transitions.add(PhantomData, HNil),
            forall_transitions,
            phantom,
        }
    }
    /// Add a transition between `Source` and `Target` vertex with specified `Action` and `Guard`.
    ///
    /// More about actions see in `umlsm::action` module.
    /// More about guards see in `umlsm::guard` module.
    pub fn add_transition<AInput, A, G, S, E, Tar, AppendIdx, Idx, Out>(
        self,
        action: AInput,
        guard: G,
        _target: PhantomData<Tar>,
    ) -> StateMachine<C, State, Vertexes, VertHandlers, HMap<Out>, FAllTransitions, Answer, GErr>
    where
        Vertexes: Selector<S, Idx>,
        Transitions: AppendInner<
            PhantomData<S>,
            Transition<S, State, E, A, G, Tar, Answer, GErr>,
            AppendIdx,
            Out,
        >,
        AInput: FnIntoStruct<A>,
        A: Action<S, State, E, Tar, Answer>,
        G: Guard<E, GErr>,
        S: 'static,
        Tar: 'static,
    {
        if TypeId::of::<S>() == TypeId::of::<Tar>() {
            panic!("If you want to add loop transition, use StateMachine::add_loop instead.")
        }
        let StateMachine {
            current,
            state,
            vertexes,
            vertices_handlers,
            transitions,
            forall_transitions,
            phantom,
        } = self;
        StateMachine {
            current,
            state,
            vertexes,
            vertices_handlers,
            transitions: transitions.append_inner(Transition::new(action.into(), guard)),
            forall_transitions,
            phantom,
        }
    }
    /// Add a transition between all vertices in state machine (except `Target` vertex) and `Target`
    /// vertex with specified `Action` and `Guard`.
    ///
    /// More about actions see in `umlsm::action` module.
    /// More about guards see in `umlsm::guard` module.
    pub fn add_transition_forall<A, G, E, Tar>(
        self,
        action: A,
        guard: G,
        _target: PhantomData<Tar>,
    ) -> StateMachine<
        C,
        State,
        Vertexes,
        VertHandlers,
        HMap<Transitions>,
        HCons<ForallTransition<A, G>, FAllTransitions>,
        Answer,
        GErr,
    >
    where
        A: Clone,
        G: Guard<E, GErr> + Clone,
    {
        let StateMachine {
            current,
            state,
            vertexes,
            vertices_handlers,
            transitions,
            forall_transitions,
            phantom,
        } = self;
        StateMachine {
            current,
            state,
            vertexes,
            vertices_handlers,
            transitions,
            forall_transitions: forall_transitions.prepend(ForallTransition::new(action, guard)),
            phantom,
        }
    }
    /// Add an loop for specified `Vertex` with `Action` and `Guard`.
    pub fn add_loop<A, G, Vertex, E, AppendIdx, Out>(
        self,
        action: A,
        guard: G,
    ) -> StateMachine<C, State, Vertexes, VertHandlers, HMap<Out>, FAllTransitions, Answer, GErr>
    where
        Transitions: AppendInner<
            PhantomData<Vertex>,
            LoopTransition<Vertex, State, E, A, G, Answer, GErr>,
            AppendIdx,
            Out,
        >,
        A: ActionLoop<Vertex, State, E, Answer>,
        G: Guard<E, GErr>,
    {
        let StateMachine {
            current,
            state,
            vertexes,
            vertices_handlers,
            transitions,
            forall_transitions,
            phantom,
        } = self;
        StateMachine {
            current,
            state,
            vertexes,
            vertices_handlers,
            transitions: transitions.append_inner(LoopTransition::new(action, guard)),
            forall_transitions,
            phantom,
        }
    }
}

/// An interface for checking current vertex of machine.
pub trait CurrentStateIs<Idx, Inner> {
    /// Check that current state of state machine is `T`
    fn is<T>(&self) -> bool
    where
        Inner: CoproductSelector<PhantomData<T>, Idx>;
}

impl<C, State, Vertexes, VertHandlers, Transitions, FAllTransitions, Answer, Idx, GErr>
    CurrentStateIs<Idx, C>
    for StateMachine<C, State, Vertexes, VertHandlers, Transitions, FAllTransitions, Answer, GErr>
{
    fn is<T>(&self) -> bool
    where
        C: CoproductSelector<PhantomData<T>, Idx>,
    {
        self.current.get().is_some()
    }
}

impl<C, State, Vertexes, VertHandlers, Transitions, FAllTransitions, Answer, GErr>
    StateMachine<C, State, Vertexes, VertHandlers, Transitions, FAllTransitions, Answer, GErr>
{
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

    /// Get specified vertex.
    pub fn get_vertex<'a, T, Idx>(&'a self) -> &'a T
    where
        Vertexes: Selector<T, Idx>,
    {
        self.vertexes.get()
    }
}

impl<
        C,
        State,
        Vertexes,
        VertHandlers,
        Transitions,
        FAllTransitions,
        E,
        OtherTR,
        Answer,
        GErr,
        OtherVH,
        OtherC,
    > ProcessEvent<E, Answer, GErr, (OtherTR, OtherVH, OtherC)>
    for StateMachine<
        C,
        State,
        Vertexes,
        VertHandlers,
        HMap<Transitions>,
        FAllTransitions,
        Answer,
        GErr,
    >
where
    Transitions: ITransition<C, State, E, C, Vertexes, Answer, GErr, OtherTR>,
    VertHandlers: VertexHandler<Vertexes, C, E, Answer, GErr, OtherVH>,
    C: ProcessByForallTransitions<FAllTransitions, State, E, Vertexes, Answer, C, GErr, OtherC>,
{
    fn process(&mut self, event: &E) -> ProcessResult<Answer, GErr> {
        use ProcessResultInner::*;

        match self
            .vertices_handlers
            .process(&mut self.vertexes, &self.current, event)
        {
            ProcessResultSubstate::Handled(answer) => return ProcessResult::Handled(answer),
            ProcessResultSubstate::NoTransitions => return ProcessResult::NoTransitions,
            ProcessResultSubstate::GuardErr(ge) => return ProcessResult::GuardErr(ge),
            ProcessResultSubstate::MustLeaveState => {}
        };
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
            HandledAndProcessNext => ProcessEvent::process(self, event),
            GuardErr(ge) => ProcessResult::GuardErr(ge),
            EventTypeNotSatisfy | NoTransitions => {
                match self.current.process_by(
                    &mut self.forall_transitions,
                    &mut self.state,
                    event,
                    &mut self.vertexes,
                ) {
                    HandledAndProcessNext => ProcessEvent::process(self, event),
                    HandledAndProcessEnd((answer, target)) => {
                        self.current = target;
                        ProcessResult::Handled(answer)
                    }
                    GuardErr(ge) => ProcessResult::GuardErr(ge),
                    EventTypeNotSatisfy | NoTransitions => ProcessResult::NoTransitions,
                }
            }
        }
    }
}
