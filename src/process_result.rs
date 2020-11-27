pub enum ProcessResult<Answer> {
    Handled(Answer),
    NoTransitions,
    GuardReturnFalse,
}

impl<Answer> ProcessResult<Answer> {
    pub fn ok(self) -> Option<Answer> {
        match self {
            ProcessResult::Handled(h) => Some(h),
            ProcessResult::NoTransitions => None,
            ProcessResult::GuardReturnFalse => None,
        }
    }

    pub fn unwrap(self) -> Answer {
        use ProcessResult::*;

        match self {
            Handled(a) => a,
            NoTransitions => unreachable!("Expected handled result, found `NoTransitions`"),
            GuardReturnFalse => unreachable!("Expected handled result, found `GuardReturnFalse`"),
        }
    }
}

pub enum ProcessResultInner<Answer> {
    HandledAndProcessNext,
    HandledAndProcessEnd(Answer),
    NoTransitions,
    GuardReturnFalse,
}

impl<Answer> Into<ProcessResult<Answer>> for ProcessResultInner<Answer> {
    fn into(self) -> ProcessResult<Answer> {
        use ProcessResultInner::*;

        match self {
            HandledAndProcessNext => unreachable!(),
            HandledAndProcessEnd(a) => ProcessResult::Handled(a),
            NoTransitions => ProcessResult::NoTransitions,
            GuardReturnFalse => ProcessResult::GuardReturnFalse,
        }
    }
}

impl<A> ProcessResultInner<A> {
    pub fn map<ANew>(self, f: impl Fn(A) -> ANew) -> ProcessResultInner<ANew> {
        use ProcessResultInner::*;

        match self {
            HandledAndProcessNext => HandledAndProcessNext,
            HandledAndProcessEnd(a) => HandledAndProcessEnd(f(a)),
            NoTransitions => NoTransitions,
            GuardReturnFalse => GuardReturnFalse,
        }
    }
}
