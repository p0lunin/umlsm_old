use crate::hmap::HMapNil;
use crate::{Action, Guard, Vertex};
use frunk::coproduct::{CNil, CoproductEmbedder};
use frunk::hlist::Selector;
use frunk::{Coproduct, HCons, HNil};
use std::any::TypeId;
use std::marker::PhantomData;

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

pub trait ITransition<Source, Ctx, Event, Target, Vertexes, Answer, Other> {
    fn process(
        &mut self,
        source: &mut Source,
        ctx: &mut Ctx,
        event: &Event,
        vertexes: &mut Vertexes,
    ) -> Result<(Answer, Target), ()>;
}

impl<Source, Ctx, Event, ActionT, GuardT, Target, Vertexes, Answer, Idx1, Idx2>
    ITransition<
        PhantomData<Source>,
        Ctx,
        Event,
        Coproduct<PhantomData<Target>, CNil>,
        Vertexes,
        Answer,
        (Idx1, Idx2),
    > for Transition<Source, Ctx, Event, ActionT, GuardT, PhantomData<Target>, Answer>
where
    Vertexes: Selector<Source, Idx1> + Selector<Target, Idx2>,
    Source: Vertex<Event>,
    Target: Vertex<Event>,
    ActionT: Action<Source, Ctx, Event, Answer>,
    GuardT: Guard<Event>,
{
    fn process(
        &mut self,
        _: &mut PhantomData<Source>,
        ctx: &mut Ctx,
        event: &Event,
        vertexes: &mut Vertexes,
    ) -> Result<(Answer, Coproduct<PhantomData<Target>, CNil>), ()> {
        if self.guard.check(event) {
            let source = Selector::<Source, Idx1>::get_mut(vertexes);
            source.exit();
            let answer = self.action.trigger(source, ctx, event);
            Selector::<Target, Idx2>::get_mut(vertexes).entry(event);
            Ok((answer, Coproduct::inject(PhantomData)))
        } else {
            Err(())
        }
    }
}

impl<Source, Ctx, Event, Vertexes, Target, Answer>
    ITransition<Source, Ctx, Event, Target, Vertexes, Answer, ()> for HNil
{
    fn process(
        &mut self,
        _: &mut Source,
        _: &mut Ctx,
        _: &Event,
        _: &mut Vertexes,
    ) -> Result<(Answer, Target), ()> {
        Err(())
    }
}

impl<
        Source,
        Ctx,
        Event,
        TargetUnit,
        Target,
        Vertexes,
        Rest,
        Indices,
        Other,
        Action,
        Guard,
        TransEvent,
        OtherTrans,
        Answer,
    >
    ITransition<
        PhantomData<Source>,
        Ctx,
        Event,
        Target,
        Vertexes,
        Answer,
        (TargetUnit, Indices, Other, OtherTrans),
    >
    for HCons<
        Transition<Source, Ctx, TransEvent, Action, Guard, PhantomData<TargetUnit>, Answer>,
        Rest,
    >
where
    Transition<Source, Ctx, TransEvent, Action, Guard, PhantomData<TargetUnit>, Answer>:
        ITransition<
            PhantomData<Source>,
            Ctx,
            TransEvent,
            Coproduct<PhantomData<TargetUnit>, CNil>,
            Vertexes,
            Answer,
            OtherTrans,
        >,
    Coproduct<PhantomData<TargetUnit>, CNil>: CoproductEmbedder<Target, Indices>,
    Rest: ITransition<PhantomData<Source>, Ctx, Event, Target, Vertexes, Answer, Other>,
    Event: 'static,
    TransEvent: 'static,
{
    fn process(
        &mut self,
        source: &mut PhantomData<Source>,
        ctx: &mut Ctx,
        event: &Event,
        vertexes: &mut Vertexes,
    ) -> Result<(Answer, Target), ()> {
        if TypeId::of::<Event>() == TypeId::of::<TransEvent>() {
            self.head
                .process(
                    source,
                    ctx,
                    unsafe { std::mem::transmute(&event) },
                    vertexes,
                )
                .map(|(a, t)| (a, t.embed()))
        } else {
            self.tail.process(source, ctx, event, vertexes)
        }
    }
}

impl<Ctx, Event, Target, Vertexes, Answer>
    ITransition<CNil, Ctx, Event, Target, Vertexes, Answer, ()> for HMapNil
{
    fn process(
        &mut self,
        source: &mut CNil,
        _: &mut Ctx,
        _: &Event,
        _: &mut Vertexes,
    ) -> Result<(Answer, Target), ()> {
        match *source {}
    }
}

impl<Source, SourceRest, Trans, Ctx, Event, Rest, OtherHM, OtherRest, Target, Vertexes, Answer>
    ITransition<
        Coproduct<PhantomData<Source>, SourceRest>,
        Ctx,
        Event,
        Target,
        Vertexes,
        Answer,
        (OtherHM, OtherRest, (Source, Trans)),
    > for HCons<(PhantomData<Source>, Trans), Rest>
where
    Trans: ITransition<PhantomData<Source>, Ctx, Event, Target, Vertexes, Answer, OtherHM>,
    Rest: ITransition<SourceRest, Ctx, Event, Target, Vertexes, Answer, OtherRest>,
{
    fn process(
        &mut self,
        source: &mut Coproduct<PhantomData<Source>, SourceRest>,
        ctx: &mut Ctx,
        event: &Event,
        vertexes: &mut Vertexes,
    ) -> Result<(Answer, Target), ()> {
        match source {
            Coproduct::Inl(l) => self.head.1.process(l, ctx, event, vertexes),
            Coproduct::Inr(r) => {
                let HCons { head: _, tail } = self;
                tail.process(r, ctx, event, vertexes)
            }
        }
    }
}
