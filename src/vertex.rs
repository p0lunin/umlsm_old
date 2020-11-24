pub trait EntryVertex<Event> {
    fn entry(&mut self, event: Event);
}

pub trait ExitVertex {
    fn exit(&mut self);
}

pub struct InitialPseudostate;

impl EntryVertex<()> for InitialPseudostate {
    fn entry(&mut self, _: ()) {

    }
}

impl ExitVertex for InitialPseudostate {
    fn exit(&mut self) {

    }
}

pub struct TerminatePseudostate;

impl EntryVertex<()> for TerminatePseudostate {
    fn entry(&mut self, _: ()) {

    }
}