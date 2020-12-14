use crate::action::ActionLoop;
use std::marker::PhantomData;

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
