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

impl<Source, Event> FnIntoStruct<EmptyAction<Source, Event>> for EmptyAction<Source, Event> {
    fn into(self) -> EmptyAction<Source, Event> {
        self
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

pub trait FnIntoStruct<T> {
    fn into(self) -> T;
}

pub struct StaticOutput<F, Answer>(F, PhantomData<Answer>);

impl<F, Answer> StaticOutput<F, Answer>
where
    F: Fn() -> Answer,
{
    pub fn new(field0: F) -> Self {
        StaticOutput(field0, PhantomData)
    }
}
impl<F, Source, Ctx, Event, Target, Answer> Action<Source, Ctx, Event, Target, Answer>
    for StaticOutput<F, Answer>
where
    F: Fn() -> Answer,
{
    fn trigger(&self, _: &mut Source, _: &mut Ctx, _: &Event, _: &mut Target) -> Answer {
        (self.0)()
    }
}
impl<F, Answer> FnIntoStruct<StaticOutput<F, Answer>> for F
where
    F: Fn() -> Answer,
{
    fn into(self) -> StaticOutput<F, Answer> {
        StaticOutput(self, PhantomData)
    }
}

pub struct FuncActionAllArgs<F, Other>(F, PhantomData<Other>);
impl<Source, Ctx, Event, Target, F, Answer> Action<Source, Ctx, Event, Target, Answer>
    for FuncActionAllArgs<F, (Source, Ctx, Event, Target, Answer)>
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
        (self.0)(source, ctx, event, target)
    }
}

impl<F, Source, Ctx, Event, Target, Answer>
    FnIntoStruct<FuncActionAllArgs<F, (Source, Ctx, Event, Target, Answer)>> for F
where
    F: Fn(&mut Source, &mut Ctx, &Event, &mut Target) -> Answer,
{
    fn into(self) -> FuncActionAllArgs<F, (Source, Ctx, Event, Target, Answer)> {
        FuncActionAllArgs(self, PhantomData)
    }
}
