mod action;
mod guard;
mod hmap;
mod sm;
mod transition;
mod utils;
mod vertex;

pub use {
    action::Action,
    guard::Guard,
    sm::{ProcessEvent, StateMachine},
    vertex::{EntryVertex, ExitVertex},
};

#[cfg(test)]
mod tests {
    use crate::sm::{ProcessEvent, StateMachine};
    use crate::{EntryVertex, ExitVertex};
    use std::marker::PhantomData;

    struct Locked;
    impl<Event> EntryVertex<Event> for Locked {
        fn entry(&mut self, _: &Event) {
            unreachable!()
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
            unreachable!()
        }
    }

    struct Push;

    fn beep(_: &mut Locked, _: &mut (), _: &Push) {
        println!("beep!");
    }

    #[test]
    fn test() {
        let sm = StateMachine::new(Locked {}, ())
            .add_vertex(Unlocked {})
            .add_transition(beep, frunk::hlist![], PhantomData::<Unlocked>);

        let mut sm = sm;
        ProcessEvent::process(&mut sm, &Push).ok().unwrap();
        assert!(sm.is::<Unlocked, _>());
        let _current = sm.get_current_as::<Unlocked, _>().unwrap();
    }
}
