mod guard;
mod hmap;
mod sm;
mod transition;
mod utils;
mod vertex;

pub use {
    guard::Guard,
    sm::StateMachine,
    transition::Action,
    vertex::{EntryVertex, ExitVertex},
};

#[cfg(test)]
mod tests {
    use crate::sm::{ProcessEvent, StateMachine};
    use crate::transition::Action;
    use crate::vertex::{EntryVertex, ExitVertex};
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

    struct Beep;
    impl Action<Locked, (), Push> for Beep {
        fn trigger(&self, _: &mut Locked, _: &mut (), _: &Push) {
            println!("beep!");
        }
    }

    #[test]
    fn test() {
        let sm = StateMachine::new(Locked {}, ())
            .add_vertex(Unlocked {})
            .add_transition(Beep, frunk::hlist![], PhantomData::<Unlocked>);

        let mut sm = sm;
        ProcessEvent::process(&mut sm, Push).ok().unwrap();
        assert!(sm.is::<Unlocked, _>());
        let _current = sm.get_current_as::<Unlocked, _>().unwrap();
    }
}
