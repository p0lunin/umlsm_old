use crate::hmap::HMapNil;
use crate::process_result::ProcessResultInner;
use crate::{Action, EntryVertex, ExitVertex, Guard};
use frunk::coproduct::{CNil, CoproductEmbedder, CoprodInjector};
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

impl<Source, Ctx, TransEvent, Event, ActionT, GuardT, Target, Vertexes, Answer, GErr, Idx1, Idx2>
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
    Vertexes: Selector<Source, Idx1> + Selector<Target, Idx2>,
    Source: ExitVertex,
    Target: EntryVertex<TransEvent>,
    ActionT: Action<Source, Ctx, TransEvent, Answer>,
    GuardT: Guard<TransEvent, GErr>,
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
                    let source = Selector::<Source, Idx1>::get_mut(vertexes);
                    let answer = self.action.trigger(source, ctx, &event);

                    source.exit();

                    let entry_vertex = Selector::<Target, Idx2>::get_mut(vertexes);
                    entry_vertex.entry(event);
                    HandledAndProcessEnd((answer, PhantomData))
                }
                Err(e) => GuardErr(e),
            }
        }
        else {
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
        let res = self.head
                .process(source, ctx, unsafe { std::mem::transmute(event) }, vertexes)
                .map(|(a, t)| (a, Target::inject(t)));

        match res {
            ProcessResultInner::EventTypeNotSatisfy => self.tail.process(source, ctx, event, vertexes),
            _ => res
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
