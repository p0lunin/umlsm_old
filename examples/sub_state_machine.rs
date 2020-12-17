use umlsm::vertex::{InitialPseudoState, TerminationPseudoState};
use umlsm::{vertex::StateMachineVertex, CurrentStateIs, ProcessEvent};

struct MyEvent;
struct MyEvent2;

fn main() {
    struct InnerSmIdx;

    #[rustfmt::skip]
    let inner_sm: StateMachineVertex<InnerSmIdx, _, _, _> = StateMachineVertex::empty(umlsm::state_machine!(
        state = (), err = (),
        [],

        InitialPseudoState + MyEvent2 => TerminationPseudoState;
    ));

    #[rustfmt::skip]
    let mut sm = umlsm::state_machine!(
        state = (), err = (),
        [@Sub inner_sm],

        InitialPseudoState                         + ()       => StateMachineVertex<InnerSmIdx, _, _, _>,
        (StateMachineVertex<InnerSmIdx, _, _, _>)  + MyEvent  => TerminationPseudoState;
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
