use std::marker::PhantomData;

pub trait Action<Source, Ctx, Event, Target, Answer> {
    fn trigger(
        &self,
        source: &mut Source,
        ctx: &mut Ctx,
        event: &Event,
        target: &mut Target,
    ) -> Answer;
}

impl<Source, Ctx, Event, Target, F, Answer> Action<Source, Ctx, Event, Target, Answer> for F
where
    F: Fn(&mut Source, &mut Ctx, &Event, &mut Target) -> Answer,
{
    fn trigger(
        &self,
        source: &mut Source,
        ctx: &mut Ctx,
        event: &Event,
        target: &mut Target,
    ) -> Answer {
        self(source, ctx, event, target)
    }
}

pub struct EmptyAction<Source, Event>(PhantomData<(Source, Event)>);

impl<Source, Event> EmptyAction<Source, Event> {
    pub fn new() -> Self {
        EmptyAction(PhantomData)
    }
}

impl<Source, Ctx, Event, Target> Action<Source, Ctx, Event, Target, ()>
    for EmptyAction<Source, Event>
{
    fn trigger(&self, _: &mut Source, _: &mut Ctx, _: &Event, _: &mut Target) -> () {
        ()
    }
}

pub trait ActionLoop<Source, Ctx, Event, Answer> {
    fn trigger(&self, source: &mut Source, ctx: &mut Ctx, event: &Event) -> Answer;
}

impl<Source, Ctx, Event, F, Answer> ActionLoop<Source, Ctx, Event, Answer> for F
where
    F: Fn(&mut Source, &mut Ctx, &Event) -> Answer,
{
    fn trigger(&self, source: &mut Source, ctx: &mut Ctx, event: &Event) -> Answer {
        self(source, ctx, event)
    }
}

pub struct EmptyActionLoop<Source, Event>(PhantomData<(Source, Event)>);

impl<Source, Event> EmptyActionLoop<Source, Event> {
    pub fn new() -> Self {
        EmptyActionLoop(PhantomData)
    }
}

impl<Source, Ctx, Event> ActionLoop<Source, Ctx, Event, ()> for EmptyActionLoop<Source, Event> {
    fn trigger(&self, _: &mut Source, _: &mut Ctx, _: &Event) -> () {
        ()
    }
}

pub struct EmptyForallAction<Event>(PhantomData<Event>);

impl<Event> EmptyForallAction<Event> {
    pub fn new() -> Self {
        EmptyForallAction(PhantomData)
    }
}

impl<Source, Ctx, Event> ActionLoop<Source, Ctx, Event, ()> for EmptyForallAction<Event> {
    fn trigger(&self, _: &mut Source, _: &mut Ctx, _: &Event) -> () {
        ()
    }
}
