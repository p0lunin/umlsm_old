pub trait Vertex<Event> {
    fn entry(&mut self, event: &Event);
    fn exit(&mut self);
}
