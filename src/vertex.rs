pub trait EntryVertex<Event> {
    fn entry(&mut self, event: Event);
}

pub trait ExitVertex {
    fn exit(&mut self);
}
