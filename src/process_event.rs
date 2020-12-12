use crate::ProcessResult;

pub trait ProcessEvent<E, Answer, GErr, Other> {
    fn process(&mut self, event: &E) -> ProcessResult<Answer, GErr>;
}
