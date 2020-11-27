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

pub struct EmptyAction<Source, Event = ()>(PhantomData<(Source, Event)>);

impl<Source, Event> EmptyAction<Source, Event> {
    pub fn new() -> Self {
        EmptyAction(PhantomData)
    }
}

impl<Source, Ctx, Event> Action<Source, Ctx, Event, ()> for EmptyAction<Source, Event> {
    fn trigger(&self, _: &mut Source, _: &mut Ctx, _: &Event) -> () {
        ()
    }
}
