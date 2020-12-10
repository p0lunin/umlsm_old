use crate::action::ActionLoop;
use crate::hmap::HMapNil;
use crate::process_result::ProcessResultInner;
use crate::utils::SelectorPointer;
use crate::{Action, EntryVertex, ExitVertex, Guard};
use frunk::coproduct::{CNil, CoprodInjector};
use frunk::hlist::Selector;
use frunk::{Coproduct, HCons, HNil};
use std::any::TypeId;
use std::marker::PhantomData;

pub struct Transition<Source, Ctx, Event, Action, Guard, Target, Answer, GErr> {
    action: Action,
    guard: Guard,
    phantom: PhantomData<(Source, Ctx, Event, Target, Answer, GErr)>,
}

impl<Source, Ctx, Event, ActionT, GuardT, GErr, Target, Answer>
    Transition<Source, Ctx, Event, ActionT, GuardT, Target, Answer, GErr>
where
    ActionT: Action<Source, Ctx, Event, Target, Answer>,
    GuardT: Guard<Event, GErr>,
{
    pub fn new(action: ActionT, guard: GuardT) -> Self {
        Transition {
            action,
            guard,
            phantom: PhantomData,
        }
    }
}

pub trait ITransition<Source, Ctx, Event, Target, Vertexes, Answer, GErr, Other> {
    fn process(
        &mut self,
        source: &mut Source,
        ctx: &mut Ctx,
        event: &Event,
        vertexes: &mut Vertexes,
    ) -> ProcessResultInner<(Answer, Target), GErr>;
}

impl<
        Source,
        Ctx,
        TransEvent,
        Event,
        ActionT,
        GuardT,
        Target,
        Vertexes,
        Answer,
        GErr,
        Idx1,
        Idx2,
    >
    ITransition<
        PhantomData<Source>,
        Ctx,
        Event,
        PhantomData<Target>,
        Vertexes,
        Answer,
        GErr,
        (Idx1, Idx2),
    > for Transition<Source, Ctx, TransEvent, ActionT, GuardT, Target, Answer, GErr>
where
    Vertexes: SelectorPointer<Source, Idx1> + SelectorPointer<Target, Idx2>,
    Source: ExitVertex,
    Target: EntryVertex,
    ActionT: Action<Source, Ctx, TransEvent, Target, Answer>,
    GuardT: Guard<TransEvent, GErr>,
    Source: 'static,
    Target: 'static,
    Event: 'static,
    TransEvent: 'static,
{
    fn process(
        &mut self,
        _: &mut PhantomData<Source>,
        ctx: &mut Ctx,
        event: &Event,
        vertexes: &mut Vertexes,
    ) -> ProcessResultInner<(Answer, PhantomData<Target>), GErr> {
        use ProcessResultInner::*;
        if TypeId::of::<Event>() == TypeId::of::<TransEvent>() {
            let event = unsafe { &*(event as *const Event as *const TransEvent) };

            match self.guard.check(event) {
                Ok(_) => {
                    if TypeId::of::<Source>() == TypeId::of::<Target>() {
                        panic!("Transition must not have the same Source and Target vertices.");
                    }
                    let source =
                        unsafe { &mut *(SelectorPointer::<Source, _>::get_mut_ptr(vertexes)) };
                    let target =
                        unsafe { &mut *(SelectorPointer::<Target, _>::get_mut_ptr(vertexes)) };
                    let answer = self.action.trigger(source, ctx, &event, target);

                    source.exit();
                    target.entry();
                    HandledAndProcessEnd((answer, PhantomData))
                }
                Err(e) => GuardErr(e),
            }
        } else {
            ProcessResultInner::EventTypeNotSatisfy
        }
    }
}

impl<Source, Ctx, Event, Vertexes, Target, Answer, GErr>
    ITransition<Source, Ctx, Event, Target, Vertexes, Answer, GErr, ()> for HNil
{
    fn process(
        &mut self,
        _: &mut Source,
        _: &mut Ctx,
        _: &Event,
        _: &mut Vertexes,
    ) -> ProcessResultInner<(Answer, Target), GErr> {
        ProcessResultInner::NoTransitions
    }
}

impl<
        Source,
        Ctx,
        Event,
        TargetUnit,
        Target,
        Vertexes,
        Rest,
        Indices,
        Other,
        OtherTrans,
        Answer,
        GErr,
        Trans,
    >
    ITransition<
        PhantomData<Source>,
        Ctx,
        Event,
        Target,
        Vertexes,
        Answer,
        GErr,
        (TargetUnit, Indices, Other, OtherTrans),
    > for HCons<Trans, Rest>
where
    Trans: ITransition<
        PhantomData<Source>,
        Ctx,
        Event,
        PhantomData<TargetUnit>,
        Vertexes,
        Answer,
        GErr,
        OtherTrans,
    >,
    Target: CoprodInjector<PhantomData<TargetUnit>, Indices>,
    Rest: ITransition<PhantomData<Source>, Ctx, Event, Target, Vertexes, Answer, GErr, Other>,
{
    fn process(
        &mut self,
        source: &mut PhantomData<Source>,
        ctx: &mut Ctx,
        event: &Event,
        vertexes: &mut Vertexes,
    ) -> ProcessResultInner<(Answer, Target), GErr> {
        let res = self
            .head
            .process(source, ctx, unsafe { std::mem::transmute(event) }, vertexes)
            .map(|(a, t)| (a, Target::inject(t)));

        match res {
            ProcessResultInner::EventTypeNotSatisfy => {
                self.tail.process(source, ctx, event, vertexes)
            }
            _ => res,
        }
    }
}

impl<Ctx, Event, Target, Vertexes, Answer, GErr>
    ITransition<CNil, Ctx, Event, Target, Vertexes, Answer, GErr, ()> for HMapNil
{
    fn process(
        &mut self,
        source: &mut CNil,
        _: &mut Ctx,
        _: &Event,
        _: &mut Vertexes,
    ) -> ProcessResultInner<(Answer, Target), GErr> {
        match *source {}
    }
}

impl<
        Source,
        SourceRest,
        Trans,
        Ctx,
        Event,
        Rest,
        OtherHM,
        OtherRest,
        Target,
        Vertexes,
        Answer,
        GErr,
    >
    ITransition<
        Coproduct<PhantomData<Source>, SourceRest>,
        Ctx,
        Event,
        Target,
        Vertexes,
        Answer,
        GErr,
        (OtherHM, OtherRest, (Source, Trans)),
    > for HCons<(PhantomData<Source>, Trans), Rest>
where
    Trans: ITransition<PhantomData<Source>, Ctx, Event, Target, Vertexes, Answer, GErr, OtherHM>,
    Rest: ITransition<SourceRest, Ctx, Event, Target, Vertexes, Answer, GErr, OtherRest>,
{
    fn process(
        &mut self,
        source: &mut Coproduct<PhantomData<Source>, SourceRest>,
        ctx: &mut Ctx,
        event: &Event,
        vertexes: &mut Vertexes,
    ) -> ProcessResultInner<(Answer, Target), GErr> {
        match source {
            Coproduct::Inl(l) => self.head.1.process(l, ctx, event, vertexes),
            Coproduct::Inr(r) => {
                let HCons { head: _, tail } = self;
                tail.process(r, ctx, event, vertexes)
            }
        }
    }
}

pub struct LoopTransition<Vertex, Ctx, Event, Action, Guard, Answer, GErr> {
    action: Action,
    guard: Guard,
    phantom: PhantomData<(Vertex, Ctx, Event, Answer, GErr)>,
}

impl<Vertex, Ctx, Event, ActionT, GuardT, GErr, Answer>
    LoopTransition<Vertex, Ctx, Event, ActionT, GuardT, Answer, GErr>
where
    ActionT: ActionLoop<Vertex, Ctx, Event, Answer>,
    GuardT: Guard<Event, GErr>,
{
    pub fn new(action: ActionT, guard: GuardT) -> Self {
        LoopTransition {
            action,
            guard,
            phantom: PhantomData,
        }
    }
}

impl<Vertex, Ctx, TransEvent, Event, ActionT, GuardT, Vertexes, Answer, GErr, Idx1>
    ITransition<
        PhantomData<Vertex>,
        Ctx,
        Event,
        PhantomData<Vertex>,
        Vertexes,
        Answer,
        GErr,
        (Idx1),
    > for LoopTransition<Vertex, Ctx, TransEvent, ActionT, GuardT, Answer, GErr>
where
    Vertexes: Selector<Vertex, Idx1>,
    Vertex: ExitVertex + EntryVertex,
    ActionT: ActionLoop<Vertex, Ctx, TransEvent, Answer>,
    GuardT: Guard<TransEvent, GErr>,
    Event: 'static,
    TransEvent: 'static,
{
    fn process(
        &mut self,
        _: &mut PhantomData<Vertex>,
        ctx: &mut Ctx,
        event: &Event,
        vertexes: &mut Vertexes,
    ) -> ProcessResultInner<(Answer, PhantomData<Vertex>), GErr> {
        use ProcessResultInner::*;
        if TypeId::of::<Event>() == TypeId::of::<TransEvent>() {
            let event = unsafe { &*(event as *const Event as *const TransEvent) };

            match self.guard.check(event) {
                Ok(_) => {
                    let vertex = Selector::<Vertex, Idx1>::get_mut(vertexes);
                    let answer = self.action.trigger(vertex, ctx, &event);

                    vertex.exit();
                    vertex.entry();
                    HandledAndProcessEnd((answer, PhantomData))
                }
                Err(e) => GuardErr(e),
            }
        } else {
            ProcessResultInner::EventTypeNotSatisfy
        }
    }
}

/*
pub struct StateMachineTransition<Current, > {
    sm: StateMachine<>
}
*/
