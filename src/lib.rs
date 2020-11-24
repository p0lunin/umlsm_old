mod vertex;
mod guard;
mod transition;
mod sm;

pub use {
    sm::StateMachine,
    vertex::{InitialPseudostate, TerminatePseudostate, EntryVertex, ExitVertex},
    transition::Transition,
    guard::Guard
};

#[cfg(test)]
mod tests {
    use crate::vertex::{EntryVertex, ExitVertex};
    use crate::sm::StateMachine;
    use std::marker::PhantomData;
    use crate::transition::Transition;

    struct Locked;
    impl<E> EntryVertex<E> for Locked {
        fn entry(&mut self, _: E) {

        }
    }
    impl ExitVertex for Locked {
        fn exit(&mut self) {

        }
    }
    struct Unlocked;
    impl<E> EntryVertex<E> for Unlocked {
        fn entry(&mut self, _: E) {

        }
    }
    impl ExitVertex for Unlocked {
        fn exit(&mut self) {

        }
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

    struct CoinTransition<Source>(PhantomData<Source>);
    impl<Source, Ctx> Transition<Source, Ctx, Coin, Unlocked> for CoinTransition<Source> {
        fn make_transition(&self, _: &mut Source, _: &mut Ctx, event: Coin) -> Coin {
            println!("blink! blink! blink!");
            event
        }
    }

    struct LockTransition;
    impl<Ctx> Transition<Unlocked, Ctx, Push, Locked> for LockTransition {
        fn make_transition(&self, _: &mut Unlocked, _: &mut Ctx, event: Push) -> Push {
            println!("beep!");
            event
        }
    }

    #[test]
    fn test() {
        let sm = StateMachine::new(())
            .add_initial(Locked)
            .add_vertex(Unlocked)
            .add_transition(BeepTransition, frunk::hlist![])
            .add_transition(LockTransition, frunk::hlist![])
            .add_transition::<_, _, Locked, _, _, _, _>(CoinTransition(PhantomData), frunk::hlist![])
            .add_transition::<_, _, Unlocked, _, _, _, _>(CoinTransition(PhantomData), frunk::hlist![]);

        let sm: StateMachine<Locked, _, _, _> = sm;

        let sm: StateMachine<Unlocked, _, _, _> =
            sm.transition::<_, _, _, _, CoinTransition<Locked>, _, _>(Coin).ok().unwrap();

        let _: StateMachine<Locked, _, _, _> =
            sm.transition::<_, _, _, _, LockTransition, _, _>(Push).ok().unwrap();
    }
}
