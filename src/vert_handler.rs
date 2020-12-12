use crate::vertex::StateMachineVertex;
use crate::{CurrentStateIs, ProcessEvent, TerminationPseudoState};
use frunk::coproduct::{CNil, CoproductSelector};
use frunk::{Coproduct, HCons, HNil};
use std::marker::PhantomData;

pub trait VertexHandler<Vertex, Idx, Event, Answer, GErr, Other> {
    fn process(
        &mut self,
        vertex: &mut Vertex,
        idx: &Idx,
        event: &Event,
    ) -> ProcessResultSubstate<Answer, GErr>;
}

pub enum ProcessResultSubstate<Answer, GErr> {
    Handled(Answer),
    NoTransitions,
    GuardErr(GErr),
    MustLeaveState,
}

pub struct EmptyVertexHandler;
impl<Vertex, Event, Answer, GErr> VertexHandler<Vertex, (), Event, Answer, GErr, ()>
    for EmptyVertexHandler
{
    fn process(
        &mut self,
        _: &mut Vertex,
        _: &(),
        _: &Event,
    ) -> ProcessResultSubstate<Answer, GErr> {
        ProcessResultSubstate::MustLeaveState
    }
}

pub struct SubStateMachineVertexHandler;
impl<C, IDX, SM, Entry, Exit, Event, Answer, GErr, Idx, Other>
    VertexHandler<
        StateMachineVertex<IDX, SM, Entry, Exit>,
        (),
        Event,
        Answer,
        GErr,
        (Idx, Other, C),
    > for SubStateMachineVertexHandler
where
    C: CoproductSelector<PhantomData<TerminationPseudoState>, Idx>,
    SM: CurrentStateIs<Idx, C> + ProcessEvent<Event, Answer, GErr, Other>,
{
    fn process(
        &mut self,
        sub: &mut StateMachineVertex<IDX, SM, Entry, Exit>,
        _: &(),
        event: &Event,
    ) -> ProcessResultSubstate<Answer, GErr> {
        if sub.sm.is::<TerminationPseudoState>() {
            ProcessResultSubstate::MustLeaveState
        } else {
            use crate::process_result::ProcessResult::*;

            match sub.sm.process(event) {
                Handled(answer) => ProcessResultSubstate::Handled(answer),
                NoTransitions => ProcessResultSubstate::NoTransitions,
                GuardErr(g) => ProcessResultSubstate::GuardErr(g),
            }
        }
    }
}

impl<Vertex, Event, Answer, GErr> VertexHandler<Vertex, CNil, Event, Answer, GErr, ()> for HNil {
    fn process(
        &mut self,
        _: &mut Vertex,
        idx: &CNil,
        _: &Event,
    ) -> ProcessResultSubstate<Answer, GErr> {
        match *idx {}
    }
}

impl<
        Vertex,
        IdxRest,
        Vertices,
        Event,
        Answer,
        GErr,
        VertHandler,
        VertHandlers,
        Other,
        OtherRest,
    >
    VertexHandler<
        HCons<Vertex, Vertices>,
        Coproduct<PhantomData<Vertex>, IdxRest>,
        Event,
        Answer,
        GErr,
        (Other, OtherRest),
    > for HCons<VertHandler, VertHandlers>
where
    VertHandler: VertexHandler<Vertex, (), Event, Answer, GErr, Other>,
    VertHandlers: VertexHandler<Vertices, IdxRest, Event, Answer, GErr, OtherRest>,
{
    fn process(
        &mut self,
        vertices: &mut HCons<Vertex, Vertices>,
        idx: &Coproduct<PhantomData<Vertex>, IdxRest>,
        event: &Event,
    ) -> ProcessResultSubstate<Answer, GErr> {
        match idx {
            Coproduct::Inl(_) => self.head.process(&mut vertices.head, &(), event),
            Coproduct::Inr(right) => self.tail.process(&mut vertices.tail, right, event),
        }
    }
}
