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
    vertex::Vertex,
};

#[cfg(test)]
mod tests {
    use crate::sm::{ProcessEvent, StateMachine};
    use crate::Vertex;
    use std::marker::PhantomData;

    struct Locked;
    impl<Event> Vertex<Event> for Locked {
        fn entry(&mut self, _: &Event) {
            unreachable!()
        }
        fn exit(&mut self) {
            println!("exit Locked!");
        }
    }
    struct Unlocked;
    impl<Event> Vertex<Event> for Unlocked {
        fn entry(&mut self, _: &Event) {
            println!("entry Unlocked!");
        }
        fn exit(&mut self) {
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
