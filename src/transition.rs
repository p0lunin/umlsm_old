use crate::hmap::HMapNil;
use crate::{Guard, Vertex};
use frunk::coproduct::{CNil, CoproductEmbedder};
use frunk::{Coproduct, HCons, HNil};
use std::any::TypeId;
use std::marker::PhantomData;

pub trait Action<Source, Ctx, Event, Answer> {
    fn trigger(&self, source: &mut Source, ctx: &mut Ctx, event: &Event) -> Answer;
}

impl<Source, Ctx, Event, F, Answer> Action<Source, Ctx, Event, Answer> for F
where
    F: Fn(&mut Source, &mut Ctx, &Event) -> Answer,
{
    fn trigger(&self, source: &mut Source, ctx: &mut Ctx, event: &Event) -> Answer {
        self(source, ctx, event)
    }
}

pub struct Transition<Source, Ctx, Event, Action, Guard, Target, Answer> {
    action: Action,
    guard: Guard,
    phantom: PhantomData<(Source, Ctx, Event, Target, Answer)>,
}

impl<Source, Ctx, Event, ActionT, GuardT, Target, Answer>
    Transition<Source, Ctx, Event, ActionT, GuardT, Target, Answer>
where
    ActionT: Action<Source, Ctx, Event, Answer>,
    GuardT: Guard<Event>,
{
    pub fn new(action: ActionT, guard: GuardT) -> Self {
        Transition {
            action,
            guard,
            phantom: PhantomData,
        }
    }
}

pub trait ITransition<Source, Ctx, Event, Target, Answer, Other> {
    fn process(
        &mut self,
        source: &mut Source,
        ctx: &mut Ctx,
        event: Event,
    ) -> Result<(Answer, Target), ()>;
}

impl<Source, Ctx, Event, ActionT, GuardT, Target, Answer>
    ITransition<Source, Ctx, Event, Coproduct<PhantomData<Target>, CNil>, Answer, ()>
    for Transition<Source, Ctx, Event, ActionT, GuardT, PhantomData<Target>, Answer>
where
    Source: Vertex,
    ActionT: Action<Source, Ctx, Event, Answer>,
    GuardT: Guard<Event>,
{
    fn process(
        &mut self,
        source: &mut Source,
        ctx: &mut Ctx,
        event: Event,
    ) -> Result<(Answer, Coproduct<PhantomData<Target>, CNil>), ()> {
        if self.guard.check(&event) {
            source.exit();
            let answer = self.action.trigger(source, ctx, &event);
            Ok((answer, Coproduct::inject(PhantomData)))
        } else {
            Err(())
        }
    }
}

impl<Source, Ctx, Target, Event, Answer> ITransition<Source, Ctx, Event, Target, Answer, ()>
    for HNil
{
    fn process(&mut self, _: &mut Source, _: &mut Ctx, _: Event) -> Result<(Answer, Target), ()> {
        Err(())
    }
}

impl<
        Source,
        Ctx,
        Event,
        TargetUnit,
        Target,
        Rest,
        Indices,
        Other,
        Action,
        Guard,
        TransEvent,
        OtherTrans,
        Answer,
    > ITransition<Source, Ctx, Event, Target, Answer, (TargetUnit, Indices, Other, OtherTrans)>
    for HCons<
        Transition<Source, Ctx, TransEvent, Action, Guard, PhantomData<TargetUnit>, Answer>,
        Rest,
    >
where
    Transition<Source, Ctx, TransEvent, Action, Guard, PhantomData<TargetUnit>, Answer>:
        ITransition<
            Source,
            Ctx,
            TransEvent,
            Coproduct<PhantomData<TargetUnit>, CNil>,
            Answer,
            OtherTrans,
        >,
    Coproduct<PhantomData<TargetUnit>, CNil>: CoproductEmbedder<Target, Indices>,
    Rest: ITransition<Source, Ctx, Event, Target, Answer, Other>,
    Event: 'static,
    TransEvent: 'static,
{
    fn process(
        &mut self,
        source: &mut Source,
        ctx: &mut Ctx,
        event: Event,
    ) -> Result<(Answer, Target), ()> {
        if TypeId::of::<Event>() == TypeId::of::<TransEvent>() {
            self.head
                .process(source, ctx, unsafe { std::mem::transmute_copy(&event) })
                .map(|(a, t)| (a, t.embed()))
        } else {
            self.tail.process(source, ctx, event)
        }
    }
}

impl<Ctx, Event, Target, Answer> ITransition<CNil, Ctx, Event, Target, Answer, ()> for HMapNil {
    fn process(&mut self, _: &mut CNil, _: &mut Ctx, _: Event) -> Result<(Answer, Target), ()> {
        Err(())
    }
}

impl<Source, SourceRest, Trans, Ctx, Event, Rest, OtherHM, OtherRest, Target, Answer>
    ITransition<
        Coproduct<PhantomData<Source>, SourceRest>,
        Ctx,
        Event,
        Target,
        Answer,
        (OtherHM, OtherRest, (Source, Trans)),
    > for HCons<(Source, Trans), Rest>
where
    Trans: ITransition<Source, Ctx, Event, Target, Answer, OtherHM>,
    Rest: ITransition<SourceRest, Ctx, Event, Target, Answer, OtherRest>,
{
    fn process(
        &mut self,
        source: &mut Coproduct<PhantomData<Source>, SourceRest>,
        ctx: &mut Ctx,
        event: Event,
    ) -> Result<(Answer, Target), ()> {
        match source {
            Coproduct::Inl(_) => self.head.1.process(&mut self.head.0, ctx, event),
            Coproduct::Inr(r) => {
                let HCons { head: _, tail } = self;
                tail.process(r, ctx, event)
            }
        }
    }
}
