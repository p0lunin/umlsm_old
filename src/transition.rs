use crate::hmap::HMapNil;
use crate::process_result::ProcessResultInner;
use crate::{Action, EntryVertex, ExitVertex, Guard};
use frunk::coproduct::{CNil, CoproductEmbedder};
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
    ActionT: Action<Source, Ctx, Event, Answer>,
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

impl<Source, Ctx, Event, ActionT, GuardT, Target, Vertexes, Answer, GErr, Idx1, Idx2>
    ITransition<
        PhantomData<Source>,
        Ctx,
        Event,
        Coproduct<PhantomData<Target>, CNil>,
        Vertexes,
        Answer,
        GErr,
        (Idx1, Idx2),
    > for Transition<Source, Ctx, Event, ActionT, GuardT, PhantomData<Target>, Answer, GErr>
where
    Vertexes: Selector<Source, Idx1> + Selector<Target, Idx2>,
    Source: ExitVertex<Event>,
    Target: EntryVertex<Event>,
    ActionT: Action<Source, Ctx, Event, Answer>,
    GuardT: Guard<Event, GErr>,
{
    fn process(
        &mut self,
        _: &mut PhantomData<Source>,
        ctx: &mut Ctx,
        event: &Event,
        vertexes: &mut Vertexes,
    ) -> ProcessResultInner<(Answer, Coproduct<PhantomData<Target>, CNil>), GErr> {
        use ProcessResultInner::*;
        match self.guard.check(event) {
            Ok(_) => {
                let source = Selector::<Source, Idx1>::get_mut(vertexes);
                source.exit(&event);
                let answer = self.action.trigger(source, ctx, &event);
                Selector::<Target, Idx2>::get_mut(vertexes).entry(&event);
                HandledAndProcessEnd((answer, Coproduct::inject(PhantomData)))
            }
            Err(e) => GuardErr(e),
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
        TransEvent,
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
        (TargetUnit, Indices, Other, OtherTrans, TransEvent),
    > for HCons<Trans, Rest>
where
    Trans: ITransition<
        PhantomData<Source>,
        Ctx,
        TransEvent,
        Coproduct<PhantomData<TargetUnit>, CNil>,
        Vertexes,
        Answer,
        GErr,
        OtherTrans,
    >,
    Coproduct<PhantomData<TargetUnit>, CNil>: CoproductEmbedder<Target, Indices>,
    Rest: ITransition<PhantomData<Source>, Ctx, Event, Target, Vertexes, Answer, GErr, Other>,
    Event: 'static,
    TransEvent: 'static,
{
    fn process(
        &mut self,
        source: &mut PhantomData<Source>,
        ctx: &mut Ctx,
        event: &Event,
        vertexes: &mut Vertexes,
    ) -> ProcessResultInner<(Answer, Target), GErr> {
        if TypeId::of::<Event>() == TypeId::of::<TransEvent>() {
            self.head
                .process(source, ctx, unsafe { std::mem::transmute(event) }, vertexes)
                .map(|(a, t)| (a, t.embed()))
        } else {
            self.tail.process(source, ctx, event, vertexes)
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
/*
pub struct StateMachineTransition<Current, > {
    sm: StateMachine<>
}
*/
