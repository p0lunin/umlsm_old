use crate::ProcessResult;

pub trait ProcessEvent<E, Answer, GErr, Other> {
    fn process(&mut self, event: &E) -> ProcessResult<Answer, GErr>;
}

pub struct EmptyPCE;
impl<E, A, GErr> ProcessEvent<E, A, GErr, ()> for EmptyPCE {
    fn process(&mut self, _: &E) -> ProcessResult<A, GErr> {
        ProcessResult::NoTransitions
    }
}
