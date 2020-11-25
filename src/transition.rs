use crate::Guard;
use frunk::coproduct::{CNil, CoprodInjector, CoproductEmbedder};
use frunk::indices::{Here, There};
use frunk::{Coproduct, HCons, HNil};
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

pub trait ITransition<Source, Ctx, Event, Target, Index, Other> {
    fn process(&mut self, source: &mut Source, ctx: &mut Ctx, event: Event)
        -> Result<Event, Event>;
    fn target(&mut self, source: &mut Source) -> Target;
}

impl<Source, Ctx, Event, ActionT, GuardT, Target>
    ITransition<Source, Ctx, Event, PhantomData<Target>, Here, ()>
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
    ) -> Result<Event, Event> {
        if self.guard.check(&event) {
            self.action.trigger(source, ctx, &event);
            Ok(event)
        } else {
            Err(event)
        }
    }

    #[inline]
    fn target(&mut self, _: &mut Source) -> PhantomData<Target> {
        PhantomData
    }
}

impl<Source, Ctx, Event> ITransition<Source, Ctx, Event, Source, Here, ()> for HNil {
    fn process(&mut self, _: &mut Source, _: &mut Ctx, event: Event) -> Result<Event, Event> {
        Err(event)
    }

    fn target(&mut self, _: &mut Source) -> Source {
        // Because we always return Err from process, we never called target
        unreachable!()
    }
}

impl<Source, Ctx, Event, TargetUnit, Target, CoprodRest, Trans, Rest, Other, Indices>
    ITransition<Source, Ctx, Event, Target, Here, (TargetUnit, CoprodRest, Other, Indices)>
    for HCons<Trans, Rest>
where
    Trans: ITransition<Source, Ctx, Event, PhantomData<TargetUnit>, Here, Other>,
    Target: CoprodInjector<PhantomData<TargetUnit>, Indices>,
{
    fn process(
        &mut self,
        source: &mut Source,
        ctx: &mut Ctx,
        event: Event,
    ) -> Result<Event, Event> {
        self.head.process(source, ctx, event)
    }

    fn target(&mut self, source: &mut Source) -> Target {
        Target::inject(self.head.target(source))
    }
}

impl<Source, Ctx, Event, Target, CoprodRest, Trans, InnerTarget, Rest, Other, Idx, Indices>
    ITransition<Source, Ctx, Event, Target, There<Idx>, (CoprodRest, Other, Indices, InnerTarget)>
    for HCons<Trans, Rest>
where
    Rest: ITransition<Source, Ctx, Event, PhantomData<InnerTarget>, Idx, Other>,
    Target: CoprodInjector<PhantomData<InnerTarget>, Indices>,
{
    fn process(
        &mut self,
        source: &mut Source,
        ctx: &mut Ctx,
        event: Event,
    ) -> Result<Event, Event> {
        self.tail.process(source, ctx, event)
    }

    fn target(&mut self, source: &mut Source) -> Target {
        Target::inject(self.tail.target(source))
    }
}

impl<Source, Trans, Ctx, Event, Target, Other, Idx>
    ITransition<Coproduct<PhantomData<Source>, CNil>, Ctx, Event, Target, Here, (Other, Idx)>
    for HCons<(Source, Trans), HNil>
where
    Trans: ITransition<Source, Ctx, Event, Target, Idx, Other>,
{
    fn process(
        &mut self,
        source: &mut Coproduct<PhantomData<Source>, CNil>,
        ctx: &mut Ctx,
        event: Event,
    ) -> Result<Event, Event> {
        match source {
            Coproduct::Inl(_) => self.head.1.process(&mut self.head.0, ctx, event),
            Coproduct::Inr(r) => match *r {},
        }
    }

    fn target(&mut self, source: &mut Coproduct<PhantomData<Source>, CNil>) -> Target {
        match source {
            Coproduct::Inl(_) => self.head.1.target(&mut self.head.0),
            Coproduct::Inr(r) => match *r {},
        }
    }
}

impl<
        Source,
        SourceMiddle,
        SourceRest,
        Trans,
        TransMiddle,
        Ctx,
        Event,
        Rest,
        Other,
        OtherRest,
        Target,
        Idx,
    >
    ITransition<
        Coproduct<PhantomData<Source>, Coproduct<PhantomData<SourceMiddle>, SourceRest>>,
        Ctx,
        Event,
        Target,
        Here,
        (Other, OtherRest, Idx),
    > for HCons<(Source, Trans), HCons<(SourceMiddle, TransMiddle), Rest>>
where
    Trans: ITransition<Source, Ctx, Event, Target, Idx, Other>,
    HCons<(SourceMiddle, TransMiddle), Rest>: ITransition<
        Coproduct<PhantomData<SourceMiddle>, SourceRest>,
        Ctx,
        Event,
        Target,
        Here,
        OtherRest,
    >,
{
    fn process(
        &mut self,
        source: &mut Coproduct<
            PhantomData<Source>,
            Coproduct<PhantomData<SourceMiddle>, SourceRest>,
        >,
        ctx: &mut Ctx,
        event: Event,
    ) -> Result<Event, Event> {
        match source {
            Coproduct::Inl(_) => self.head.1.process(&mut self.head.0, ctx, event),
            Coproduct::Inr(r) => {
                let HCons { head: _, tail } = self;
                tail.process(r, ctx, event)
            }
        }
    }

    fn target(
        &mut self,
        source: &mut Coproduct<
            PhantomData<Source>,
            Coproduct<PhantomData<SourceMiddle>, SourceRest>,
        >,
    ) -> Target {
        match source {
            Coproduct::Inl(_) => self.head.1.target(&mut self.head.0),
            Coproduct::Inr(r) => self.tail.target(r),
        }
    }
}
