mod guard;
mod sm;
mod transition;
mod vertex;

pub use {
    guard::Guard,
    sm::StateMachine,
    transition::Transition,
    vertex::{EntryVertex, ExitVertex, InitialPseudostate, TerminatePseudostate},
};

#[cfg(test)]
mod tests {
    use crate::sm::{ProcessEvent, StateMachine};
    use crate::transition::Transition;
    use crate::vertex::{EntryVertex, ExitVertex};
    use crate::Guard;
    use frunk::hlist::Selector;
    use std::marker::PhantomData;

    struct Locked;
    impl<E> EntryVertex<E> for Locked {
        fn entry(&mut self, _: E) {}
    }
    impl ExitVertex for Locked {
        fn exit(&mut self) {}
    }
    struct Unlocked;
    impl<E> EntryVertex<E> for Unlocked {
        fn entry(&mut self, _: E) {}
    }
    impl ExitVertex for Unlocked {
        fn exit(&mut self) {}
    }

    struct Push;
    struct Coin;

    struct BeepTransition;
    impl<Ctx> Transition<Locked, Ctx, Push, Locked> for BeepTransition {
        fn make_transition(&self, _: &mut Locked, _: &mut Ctx, event: Push) -> Push {
            println!("beep!");
            event
        }
    }

    impl<G, CIdx, TransIdx, /**/ State, Vertexes, Transitions, TransWithGuards>
        ProcessEvent<Push, (BeepTransition, Locked, G, CIdx, TransIdx)>
        for StateMachine<Locked, State, Vertexes, Transitions, TransWithGuards>
    where
        Vertexes: Selector<Locked, CIdx>,
        Transitions: Selector<BeepTransition, TransIdx>,
        TransWithGuards: Selector<(BeepTransition, G), TransIdx>,
        G: Guard<Push>,
    {
        type ResultOk = StateMachine<Locked, State, Vertexes, Transitions, TransWithGuards>;

        fn process(mut self, event: Push) -> Result<Self::ResultOk, Self> {
            let current: &mut Locked = self.vertexes.get_mut();
            let (trans, guard) = self.trans_with_guards.get_mut();

            if guard.check(&event) {
                current.exit();
                let event = trans.make_transition(current, &mut self.state, event);

                let target: &mut Locked = self.vertexes.get_mut();
                target.entry(event);

                let StateMachine {
                    state,
                    vertexes,
                    trans_with_guards,
                    ..
                } = self;
                Ok(StateMachine {
                    state,
                    vertexes,
                    trans_with_guards,
                    phantom: PhantomData,
                })
            } else {
                Err(self)
            }
        }
    }

    struct CoinTransition<Source>(PhantomData<Source>);
    impl<Source, Ctx> Transition<Source, Ctx, Coin, Unlocked> for CoinTransition<Source> {
        fn make_transition(&self, _: &mut Source, _: &mut Ctx, event: Coin) -> Coin {
            println!("blink! blink! blink!");
            event
        }
    }

    impl<G, CIdx, TIdx, TransIdx, /**/ C, State, Vertexes, Transitions, TransWithGuards>
        ProcessEvent<Coin, (CoinTransition<C>, Unlocked, G, CIdx, TIdx, TransIdx)>
        for StateMachine<C, State, Vertexes, Transitions, TransWithGuards>
    where
        Vertexes: Selector<C, CIdx> + Selector<Unlocked, TIdx>,
        Transitions: Selector<CoinTransition<C>, TransIdx>,
        TransWithGuards: Selector<(CoinTransition<C>, G), TransIdx>,
        C: ExitVertex,
        G: Guard<Coin>,
    {
        type ResultOk = StateMachine<Unlocked, State, Vertexes, Transitions, TransWithGuards>;

        fn process(mut self, event: Coin) -> Result<Self::ResultOk, Self> {
            let current: &mut C = self.vertexes.get_mut();
            let (trans, guard) = self.trans_with_guards.get_mut();

            if guard.check(&event) {
                current.exit();
                let event = trans.make_transition(current, &mut self.state, event);

                let target: &mut Unlocked = self.vertexes.get_mut();
                target.entry(event);

                let StateMachine {
                    state,
                    vertexes,
                    trans_with_guards,
                    ..
                } = self;
                Ok(StateMachine {
                    state,
                    vertexes,
                    trans_with_guards,
                    phantom: PhantomData,
                })
            } else {
                Err(self)
            }
        }
    }

    struct LockTransition;
    impl<Ctx> Transition<Unlocked, Ctx, Push, Locked> for LockTransition {
        fn make_transition(&self, _: &mut Unlocked, _: &mut Ctx, event: Push) -> Push {
            println!("beep!");
            event
        }
    }

    impl<T, G, CIdx, TIdx, TransIdx, /**/ State, Vertexes, Transitions, TransWithGuards>
        ProcessEvent<Push, (LockTransition, T, G, CIdx, TIdx, TransIdx)>
        for StateMachine<Unlocked, State, Vertexes, Transitions, TransWithGuards>
    where
        Vertexes: Selector<Unlocked, CIdx> + Selector<T, TIdx>,
        Transitions: Selector<LockTransition, TransIdx>,
        TransWithGuards: Selector<(LockTransition, G), TransIdx>,
        T: EntryVertex<Push>,
        G: Guard<Push>,
    {
        type ResultOk = StateMachine<T, State, Vertexes, Transitions, TransWithGuards>;

        fn process(mut self, event: Push) -> Result<Self::ResultOk, Self> {
            let current: &mut Unlocked = self.vertexes.get_mut();
            let (trans, guard) = self.trans_with_guards.get_mut();

            if guard.check(&event) {
                current.exit();
                let event = trans.make_transition(current, &mut self.state, event);

                let target: &mut T = self.vertexes.get_mut();
                target.entry(event);

                let StateMachine {
                    state,
                    vertexes,
                    trans_with_guards,
                    ..
                } = self;
                Ok(StateMachine {
                    state,
                    vertexes,
                    trans_with_guards,
                    phantom: PhantomData,
                })
            } else {
                Err(self)
            }
        }
    }

    #[test]
    fn test() {
        let sm = StateMachine::new(())
            .add_initial(Locked)
            .add_vertex(Unlocked)
            .add_transition(BeepTransition, frunk::hlist![])
            .add_transition(LockTransition, frunk::hlist![])
            .add_transition::<_, _, Locked, _, _, _, _>(
                CoinTransition(PhantomData),
                frunk::hlist![],
            )
            .add_transition::<_, _, Unlocked, _, _, _, _>(
                CoinTransition(PhantomData),
                frunk::hlist![],
            );

        let sm: StateMachine<Locked, _, _, _, _> = sm;

        let sm: StateMachine<Unlocked, _, _, _, _> = sm.process(Coin).ok().unwrap();

        let _: StateMachine<Locked, _, _, _, _> = sm.process(Push).ok().unwrap();
    }
}
