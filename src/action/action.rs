/// An interface for actions that must be called when running `ITransition`.
///
/// See module-level documentation for more information.
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
