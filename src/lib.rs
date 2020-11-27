pub mod action;
mod guard;
mod hmap;
mod process_result;
mod sm;
mod transition;
mod utils;
pub mod vertex;

pub use {
    action::Action,
    guard::Guard,
    sm::{ProcessEvent, StateMachine},
    vertex::{EntryVertex, ExitVertex},
};

#[cfg(test)]
mod tests {
    use crate::action::EmptyAction;
    use crate::sm::{CurrentStateIs, ProcessEvent, StateMachine};
    use crate::vertex::{InitialPseudoState, TerminationPseudoState};
    use crate::{EntryVertex, ExitVertex};
    use std::marker::PhantomData;

    struct Locked;
    impl<Event> EntryVertex<Event> for Locked {
        fn entry(&mut self, _: &Event) {
            println!("entry Locked!");
        }
    }
    impl<Event> ExitVertex<Event> for Locked {
        fn exit(&mut self, _: &Event) {
            println!("exit Locked!");
        }
    }
    struct Unlocked;
    impl<Event> EntryVertex<Event> for Unlocked {
        fn entry(&mut self, _: &Event) {
            println!("entry Unlocked!");
        }
    }
    impl<Event> ExitVertex<Event> for Unlocked {
        fn exit(&mut self, _: &Event) {
            println!("exit Unlocked!");
        }
    }

    struct Push;

    fn beep(_: &mut Locked, _: &mut (), _: &Push) {
        println!("beep!");
    }

    #[test]
    fn test() {
        let sm = StateMachine::new(())
            .add_vertex(Locked {})
            .add_vertex(Unlocked {})
            .add_transition(
                EmptyAction::<InitialPseudoState>::new(),
                frunk::hlist![],
                PhantomData::<Locked>,
            )
            .add_transition(beep, frunk::hlist![], PhantomData::<Unlocked>)
            .add_transition(
                EmptyAction::<Unlocked>::new(),
                frunk::hlist![],
                PhantomData::<TerminationPseudoState>,
            );

        let mut sm = sm;
        assert!(sm.is::<InitialPseudoState>());

        sm.process(&()).unwrap();
        assert!(sm.is::<Locked>());

        sm.process(&Push).unwrap();
        assert!(sm.is::<Unlocked>());

        sm.process(&()).unwrap();
        assert!(sm.is::<TerminationPseudoState>());

        assert!(!ProcessEvent::process(&mut sm, &()).is_handled());
    }
}
