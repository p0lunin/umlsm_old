pub mod action;
mod guard;
mod hmap;
pub mod process_result;
mod sm;
mod transition;
mod utils;
pub mod vertex;

pub use {
    action::Action,
    guard::Guard,
    process_result::ProcessResult,
    sm::{CurrentStateIs, ProcessEvent, StateMachine},
    vertex::{EntryVertex, ExitVertex, InitialPseudoState, TerminationPseudoState},
};

pub mod reexport {
    pub use frunk;
}

#[macro_export]
macro_rules! state_machine {
    (parse_source, _) => { _ };
    (parse_source, $some:ty) => { $some };
    (parse_source, ($some:ty)) => { $some };

    (parse_action, $source:tt, ) => { $crate::action::EmptyAction::<$crate::state_machine!(parse_source, $source)>::new() };
    (parse_action, $source:tt, $action:expr) => { $action };

    (parse_err, ) => { () };
    (parse_err, $some:ty) => { $some };

    (state = $state:expr $(, err = $err:ty)?, [$($vertex:expr),*], $($source:tt + $event:ty $([$($guard:expr),*])? $(| $action:expr)? => $target:ty),*) => {
        $crate::StateMachine::<_, _, _, _, _, $crate::state_machine!(parse_err, $($err)?)>::new($state)
            $(.add_vertex($vertex))*
            $(.add_transition::<_, _, $crate::state_machine!(parse_source, $source), $event, $target, _, _>(
                $crate::state_machine!(parse_action, $source, $($action)?),
                $crate::reexport::frunk::hlist![$($($guard),*)?],
                std::marker::PhantomData,
            ))*
    };
}

#[cfg(test)]
mod tests {
    use crate::sm::{CurrentStateIs, ProcessEvent};
    use crate::vertex::{InitialPseudoState, TerminationPseudoState};
    use crate::{EntryVertex, ExitVertex};

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
        let sm = state_machine!(
            state = (),
            [Locked {}, Unlocked {}],
            InitialPseudoState + ()   []        => Locked,
            Locked             + Push    | beep => Unlocked,
            Unlocked           + ()             => TerminationPseudoState
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
