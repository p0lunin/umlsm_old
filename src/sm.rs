use crate::action::{Action, ActionLoop};
use crate::guard::Guard;
use crate::hmap::{AppendInner, HMap, HMapNil};
use crate::process_result::{ProcessResult, ProcessResultInner};
use crate::transition::{ITransition, LoopTransition, Transition};
use crate::utils::{CoprodWithRef, CoprodWithoutPhantomData, GetRefsFromCoprod};
use crate::vert_handler::{EmptyVertexHandler, ProcessResultSubstate, VertexHandler};
use crate::vertex::{InitialPseudoState, TerminationPseudoState};
use crate::ProcessEvent;
use frunk::coproduct::{CNil, CoproductEmbedder, CoproductSelector};
use frunk::hlist::{h_cons, HList};
use frunk::{Coproduct, HCons, HNil};
use std::any::TypeId;
use std::marker::PhantomData;

pub struct StateMachine<Current, State, Vertexes, VertHandlers, Transitions, Answer, GErr> {
    pub current: Current,
    pub state: State,
    pub vertexes: Vertexes,
    pub vertices_handlers: VertHandlers,
    pub transitions: Transitions,
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
        Answer,
        GErr,
    >
{
    pub fn new(state: State) -> Self {
        Self {
            current: Coproduct::inject(PhantomData::<InitialPseudoState>),
            state,
            vertexes: h_cons(InitialPseudoState, h_cons(TerminationPseudoState, HNil)),
            vertices_handlers: h_cons(EmptyVertexHandler, h_cons(EmptyVertexHandler, HNil)),
            transitions: HMap::new().add(PhantomData, HNil).add(PhantomData, HNil),
            phantom: PhantomData,
        }
    }
}

impl<C, State, Vertexes: HList, VertHandlers: HList, Transitions: HList, Answer, GErr>
    StateMachine<C, State, Vertexes, VertHandlers, HMap<Transitions>, Answer, GErr>
{
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
            phantom,
        } = self;
        StateMachine {
            current: current.embed(),
            state,
            vertexes: vertexes.prepend(vertex),
            vertices_handlers: vertices_handlers.prepend(vertex_handler),
            transitions: transitions.add(PhantomData, HNil),
            phantom,
        }
    }
    pub fn add_transition<A, G, S, E, Tar, AppendIdx, Out>(
        self,
        action: A,
        guard: G,
        _target: PhantomData<Tar>,
    ) -> StateMachine<C, State, Vertexes, VertHandlers, HMap<Out>, Answer, GErr>
    where
        Transitions: AppendInner<
            PhantomData<S>,
            Transition<S, State, E, A, G, Tar, Answer, GErr>,
            AppendIdx,
            Out,
        >,
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
            phantom,
        } = self;
        StateMachine {
            current,
            state,
            vertexes,
            vertices_handlers,
            transitions: transitions.append_inner(Transition::new(action, guard)),
            phantom,
        }
    }
    pub fn add_loop<A, G, Vertex, E, AppendIdx, Out>(
        self,
        action: A,
        guard: G,
    ) -> StateMachine<C, State, Vertexes, VertHandlers, HMap<Out>, Answer, GErr>
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
            phantom,
        } = self;
        StateMachine {
            current,
            state,
            vertexes,
            vertices_handlers,
            transitions: transitions.append_inner(LoopTransition::new(action, guard)),
            phantom,
        }
    }
}

pub trait CurrentStateIs<Idx, Inner> {
    fn is<T>(&self) -> bool
    where
        Inner: CoproductSelector<PhantomData<T>, Idx>;
}

impl<C, State, Vertexes, VertHandlers, Transitions, Answer, Idx, GErr> CurrentStateIs<Idx, C>
    for StateMachine<C, State, Vertexes, VertHandlers, Transitions, Answer, GErr>
{
    fn is<T>(&self) -> bool
    where
        C: CoproductSelector<PhantomData<T>, Idx>,
    {
        self.current.get().is_some()
    }
}

impl<C, State, Vertexes, VertHandlers, Transitions, Answer, GErr>
    StateMachine<C, State, Vertexes, VertHandlers, Transitions, Answer, GErr>
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
}

impl<C, State, Vertexes, VertHandlers, Transitions, E, OtherTR, Answer, GErr, OtherVH>
    ProcessEvent<E, Answer, GErr, (OtherTR, OtherVH)>
    for StateMachine<C, State, Vertexes, VertHandlers, HMap<Transitions>, Answer, GErr>
where
    Transitions: ITransition<C, State, E, C, Vertexes, Answer, GErr, OtherTR>,
    VertHandlers: VertexHandler<Vertexes, C, E, Answer, GErr, OtherVH>,
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
            HandledAndProcessNext => ProcessEvent::process(self, &event),
            NoTransitions => ProcessResult::NoTransitions,
            GuardErr(ge) => ProcessResult::GuardErr(ge),
            EventTypeNotSatisfy => ProcessResult::NoTransitions,
        }
    }
}
