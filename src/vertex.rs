pub trait EntryVertex<Event> {
    fn entry(&mut self, event: &Event);
}

pub trait ExitVertex<Event> {
    fn exit(&mut self, event: &Event);
}
