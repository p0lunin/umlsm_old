pub mod action;
mod guard;
mod hmap;
mod process_event;
pub mod process_result;
mod sm;
mod transition;
mod utils;
pub mod vert_handler;
pub mod vertex;

pub use {
    action::Action,
    guard::Guard,
    process_event::ProcessEvent,
    process_result::ProcessResult,
    sm::{CurrentStateIs, StateMachine},
    vertex::{EntryVertex, ExitVertex, InitialPseudoState, TerminationPseudoState},
};

#[doc(hidden)]
pub mod reexport {
    pub use frunk;
}

#[macro_export]
macro_rules! state_machine {
    (parse_source, _) => { _ };
    (parse_source, ($some:ty)) => { $some };
    (parse_source, $some:ty) => { $some };

    (parse_action, $source:tt, $event:ty, ) => { $crate::action::EmptyAction::<$crate::state_machine!(parse_source, $source), $event>::new() };
    (parse_action, $source:tt, $event:ty,$action:expr) => { $action };

    (parse_action_loop, $source:tt, $event:ty, ) => { $crate::action::EmptyActionLoop::<$crate::state_machine!(parse_source, $source), $event>::new() };
    (parse_action_loop, $source:tt, $event:ty,$action:expr) => { $action };

    (parse_action_forall, $event:ty, ) => { $crate::action::EmptyForallAction::<$event>::new() };
    (parse_action_forall, $event:ty,$action:expr) => { $action };

    (parse_err, ) => { () };
    (parse_err, $some:ty) => { $some };

    (parse_v_type, ) => { $crate::vert_handler::EmptyVertexHandler };
    (parse_v_type, Sub) => { $crate::vert_handler::SubStateMachineVertexHandler };

    (
        state = $state:expr
        $(, err = $err:ty)?,
        [$($(@$type:ident)?$vertex:expr),*],
        $($($source:tt + $event:ty $([$($guard:expr),*])? $(| $action:expr)? => $target:ty),*;)?
        $(forall: $(+ $event3:ty $([$($guard3:expr),*])? $(| $action3:expr)? => $target3:ty;)+)?
        $(loop: $($source2:tt + $event2:ty $([$($guard2:expr),*])? $(| $action2:expr)?),*;)?
    ) => {
        $crate::StateMachine::<_, _, _, _, _, _, _, $crate::state_machine!(parse_err, $($err)?)>::new($state)
            $(.add_vertex($vertex, $crate::state_machine!(parse_v_type, $($type)?)))*
            $($(.add_transition::<_, _, _, $crate::state_machine!(parse_source, $source), $event, $target, _, _>(
                $crate::state_machine!(parse_action, $source, $event, $($action)?),
                $crate::reexport::frunk::hlist![$($($guard),*)?],
                std::marker::PhantomData,
            ))*)?
            $($(.add_transition_forall::<_, _, $event3, $target3>(
                $crate::state_machine!(parse_action_forall, $event3, $($action3)?),
                $crate::reexport::frunk::hlist![$($($guard3),*)?],
                std::marker::PhantomData,
            ))+)?
            $($(.add_loop::<_, _, $crate::state_machine!(parse_source, $source2), $event2, _, _>(
                $crate::state_machine!(parse_action_loop, $source2, $event2, $($action2)?),
                $crate::reexport::frunk::hlist![$($($guard2),*)?],
            ))*)?
    };
}

#[cfg(test)]
mod tests {
    use crate::sm::CurrentStateIs;
    use crate::vertex::{InitialPseudoState, TerminationPseudoState};
    use crate::{EntryVertex, ExitVertex, ProcessEvent};

    struct Locked;
    impl EntryVertex for Locked {
        fn entry(&mut self) {
            println!("entry Locked!");
        }
    }
    impl ExitVertex for Locked {
        fn exit(&mut self) {
            println!("exit Locked!");
        }
    }
    struct Unlocked;
    impl EntryVertex for Unlocked {
        fn entry(&mut self) {
            println!("entry Unlocked!");
        }
    }
    impl ExitVertex for Unlocked {
        fn exit(&mut self) {
            println!("exit Unlocked!");
        }
    }

    struct Push;

    fn beep(_: &mut Locked, _: &mut (), _: &Push, _: &mut Unlocked) {
        println!("beep!");
    }

    #[test]
    fn test() {
        let sm = state_machine!(
            state = (), err = (),
            [Locked {}, Unlocked {}],

            InitialPseudoState + ()   []        => Locked,
            Locked             + Push    | beep => Unlocked,
            Unlocked           + ()             => TerminationPseudoState;
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

    struct Looped;
    impl EntryVertex for Looped {}
    impl ExitVertex for Looped {}

    struct AEvent;
    struct BEvent;

    fn a_event(_: &mut Looped, _: &mut (), _: &AEvent) {}
    fn b_event(_: &mut Looped, _: &mut (), _: &BEvent, _: &mut TerminationPseudoState) {}

    #[test]
    fn test_looped() {
        let mut sm = state_machine!(
            state = (), err = (),
            [Looped],

            InitialPseudoState + ()               => Looped,
            Looped             + BEvent | b_event => TerminationPseudoState;

            loop:
            Looped              + AEvent | a_event;
        );

        sm.process(&()).unwrap();

        sm.process(&AEvent).unwrap();
        assert!(sm.is::<Looped>());

        sm.process(&AEvent).unwrap();
        assert!(sm.is::<Looped>());

        sm.process(&BEvent).unwrap();
        assert!(sm.is::<TerminationPseudoState>());
    }
}
