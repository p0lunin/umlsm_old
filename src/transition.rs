use crate::hmap::HMapNil;
use crate::Guard;
use frunk::coproduct::{CNil, CoproductEmbedder};
use frunk::{Coproduct, HCons, HNil};
use std::any::TypeId;
use std::marker::PhantomData;

pub trait Action<Source, Ctx, Event> {
    fn trigger(&self, source: &mut Source, ctx: &mut Ctx, event: &Event);
}

impl<Source, Ctx, Event, F> Action<Source, Ctx, Event> for F
where
    F: Fn(&mut Source, &mut Ctx, &Event),
{
    fn trigger(&self, source: &mut Source, ctx: &mut Ctx, event: &Event) {
        self(source, ctx, event)
    }
}

pub struct Transition<Source, Ctx, Event, Action, Guard, Target> {
    action: Action,
    guard: Guard,
    phantom: PhantomData<(Source, Ctx, Event, Target)>,
}

impl<Source, Ctx, Event, ActionT, GuardT, Target>
    Transition<Source, Ctx, Event, ActionT, GuardT, Target>
where
    ActionT: Action<Source, Ctx, Event>,
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

pub trait ITransition<Source, Ctx, Event, Target, Other> {
    fn process(&mut self, source: &mut Source, ctx: &mut Ctx, event: Event) -> Result<Target, ()>;
}

impl<Source, Ctx, Event, ActionT, GuardT, Target>
    ITransition<Source, Ctx, Event, Coproduct<PhantomData<Target>, CNil>, ()>
    for Transition<Source, Ctx, Event, ActionT, GuardT, PhantomData<Target>>
where
    ActionT: Action<Source, Ctx, Event>,
    GuardT: Guard<Event>,
{
    fn process(
        &mut self,
        source: &mut Source,
        ctx: &mut Ctx,
        event: Event,
    ) -> Result<Coproduct<PhantomData<Target>, CNil>, ()> {
        if self.guard.check(&event) {
            self.action.trigger(source, ctx, &event);
            Ok(Coproduct::inject(PhantomData))
        } else {
            Err(())
        }
    }
}

impl<Source, Ctx, Target, Event> ITransition<Source, Ctx, Event, Target, ()> for HNil {
    fn process(&mut self, _: &mut Source, _: &mut Ctx, _: Event) -> Result<Target, ()> {
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
    > ITransition<Source, Ctx, Event, Target, (TargetUnit, Indices, Other, OtherTrans)>
    for HCons<Transition<Source, Ctx, TransEvent, Action, Guard, PhantomData<TargetUnit>>, Rest>
where
    Transition<Source, Ctx, TransEvent, Action, Guard, PhantomData<TargetUnit>>:
        ITransition<Source, Ctx, TransEvent, Coproduct<PhantomData<TargetUnit>, CNil>, OtherTrans>,
    Coproduct<PhantomData<TargetUnit>, CNil>: CoproductEmbedder<Target, Indices>,
    Rest: ITransition<Source, Ctx, Event, Target, Other>,
    Event: 'static,
    TransEvent: 'static,
{
    fn process(&mut self, source: &mut Source, ctx: &mut Ctx, event: Event) -> Result<Target, ()> {
        if TypeId::of::<Event>() == TypeId::of::<TransEvent>() {
            self.head
                .process(source, ctx, unsafe { std::mem::transmute_copy(&event) })
                .map(|t| t.embed())
        } else {
            self.tail.process(source, ctx, event)
        }
    }
}
/*
CoprodInjector<PhantomData<Coproduct<PhantomData<Unlocked>, Coproduct<PhantomData<Locked>, CNil>>>, There<_>>` for
    `Coproduct<PhantomData<tests::Locked>, frunk::coproduct::CNil>`

Transition<Locked, (), Push, Beep, HNil, PhantomData<Unlocked>>:
    ITransition<Locked, (), Push, PhantomData<Unlocked>, _>` is not satisfied


impl<Source, Trans, Ctx, Event, Other, Idx, TargetConvert>
    ITransition<Coproduct<PhantomData<Source>, CNil>, Ctx, Event, Coproduct<PhantomData<Source>, CNil>, There<Here>, (Other, Idx, TargetConvert, (), (), ())>
    for HCons<(Source, Trans), HNil>
{
    fn process(
        &mut self,
        source: &mut Coproduct<PhantomData<Source>, CNil>,
        ctx: &mut Ctx,
        event: Event,
    ) -> Result<Event, Event> {
        ITransition::<_, _, _, Coproduct<PhantomData<Source>, CNil>, _, _>::process(&mut HNil, source, ctx, event)
    }

    fn target(&mut self, source: &mut Coproduct<PhantomData<Source>, CNil>) -> Coproduct<PhantomData<Source>, CNil> {
        ITransition::<_, Ctx, Event, _, _, _>::target(&mut HNil, &mut self.head.0)
    }
}*/
impl<Ctx, Event, Target> ITransition<CNil, Ctx, Event, Target, ()> for HMapNil {
    fn process(&mut self, _: &mut CNil, _: &mut Ctx, _: Event) -> Result<Target, ()> {
        Err(())
    }
}

impl<Source, SourceRest, Trans, Ctx, Event, Rest, OtherHM, OtherRest, Target>
    ITransition<
        Coproduct<PhantomData<Source>, SourceRest>,
        Ctx,
        Event,
        Target,
        (OtherHM, OtherRest, (Source, Trans)),
    > for HCons<(Source, Trans), Rest>
where
    Trans: ITransition<Source, Ctx, Event, Target, OtherHM>,
    Rest: ITransition<SourceRest, Ctx, Event, Target, OtherRest>,
{
    fn process(
        &mut self,
        source: &mut Coproduct<PhantomData<Source>, SourceRest>,
        ctx: &mut Ctx,
        event: Event,
    ) -> Result<Target, ()> {
        match source {
            Coproduct::Inl(_) => self.head.1.process(&mut self.head.0, ctx, event),
            Coproduct::Inr(r) => {
                let HCons { head: _, tail } = self;
                tail.process(r, ctx, event)
            }
        }
    }
}
