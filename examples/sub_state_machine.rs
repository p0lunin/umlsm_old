use umlsm::{
    CurrentStateIs, EntryVertex, ExitVertex, Guard, InitialPseudoState, ProcessEvent,
    ProcessResult, TerminationPseudoState,
    vertex::StateMachineVertex,
};

// Vertexes

struct State1;
impl EntryVertex for State1 {}
impl ExitVertex for State1 {}

// Events
struct MyEvent;
struct MyEvent2;

fn main() {
    struct InnerSmIdx;

    #[rustfmt::skip]
    let inner_sm: StateMachineVertex<InnerSmIdx, _, _, _> = StateMachineVertex::empty(umlsm::state_machine!(
        state = (), err = (),
        [],
        @InitialPseudoState + MyEvent2 => TerminationPseudoState,
    ));

    #[rustfmt::skip]
    let mut sm = umlsm::state_machine!(
        state = (), err = (),
        [@Sub inner_sm],

        @InitialPseudoState                         + ()       => StateMachineVertex<InnerSmIdx, _, _, _>,
        @(StateMachineVertex<InnerSmIdx, _, _, _>)  + MyEvent  => TerminationPseudoState,
    );
    assert!(sm.is::<InitialPseudoState>());

    sm.process(&()).unwrap();
    assert!(sm.is::<StateMachineVertex<InnerSmIdx, _, _, _>>());

    sm.process(&());
    assert!(sm.is::<StateMachineVertex<InnerSmIdx, _, _, _>>());

    sm.process(&MyEvent2).unwrap();
    assert!(sm.is::<StateMachineVertex<InnerSmIdx, _, _, _>>());

    sm.process(&());
    assert!(sm.is::<StateMachineVertex<InnerSmIdx, _, _, _>>());

    sm.process(&MyEvent).unwrap();
    assert!(sm.is::<TerminationPseudoState>());
}
