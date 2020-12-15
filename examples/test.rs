use umlsm::{EntryVertex, ExitVertex, InitialPseudoState, ProcessEvent};

// Vertexes

struct WaitForHello;
impl ExitVertex for WaitForHello {}
impl EntryVertex for WaitForHello {}
struct WaitForName;
impl ExitVertex for WaitForName {}
impl EntryVertex for WaitForName {}

fn main() {
    #[rustfmt::skip]
    let mut sm = umlsm::state_machine!(
        state = (), err = String,
        [WaitForHello, WaitForName],

        InitialPseudoState + i32  => WaitForHello,
        WaitForHello       + ()   => WaitForName;
    );
    sm.process(&()).unwrap();
}
