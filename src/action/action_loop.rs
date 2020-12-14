/// An interface for actions that must be called when running `LoopTransition`.
///
/// See module-level documentation for more information.
///
/// ### Why we need just one more trait?
///
/// `Action` trait can require both `Source` and `Target` transitions at the same time by the
/// mutable reference, and when `Source` == `Target` in case of loops, we got 2 mutable references for
/// the one address of memory, which is UB (undefined behaviour).
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
