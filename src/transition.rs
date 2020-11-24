use std::marker::PhantomData;

pub trait Transition<Source, Ctx, Event, Target> {
    fn make_transition(&self, source: &mut Source, ctx: &mut Ctx, event: Event) -> Event;
}

impl<Source, Ctx, Event, Target, F> Transition<Source, Ctx, Event, Target> for F
where
    F: Fn(&mut Source, &mut Ctx, Event, PhantomData<Target>) -> Event
{
    fn make_transition(&self, source: &mut Source, ctx: &mut Ctx, event: Event) -> Event {
        self(source, ctx, event, PhantomData)
    }
}
