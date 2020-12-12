pub enum ProcessResult<Answer, GErr> {
    Handled(Answer),
    NoTransitions,
    GuardErr(GErr),
}

impl<Answer, GErr> ProcessResult<Answer, GErr> {
    pub fn ok(self) -> Option<Answer> {
        match self {
            ProcessResult::Handled(h) => Some(h),
            ProcessResult::NoTransitions => None,
            ProcessResult::GuardErr(_) => None,
        }
    }

    pub fn unwrap(self) -> Answer {
        use ProcessResult::*;

        match self {
            Handled(a) => a,
            NoTransitions => unreachable!("Expected handled result, found `NoTransitions`"),
            GuardErr(_) => unreachable!("Expected handled result, found `GuardReturnFalse`"),
        }
    }

    pub fn is_handled(&self) -> bool {
        match &self {
            ProcessResult::Handled(_) => true,
            _ => false,
        }
    }
}

pub enum ProcessResultInner<Answer, GErr> {
    HandledAndProcessNext,
    EventTypeNotSatisfy,
    HandledAndProcessEnd(Answer),
    NoTransitions,

    GuardErr(GErr),
}

impl<Answer, GErr> Into<ProcessResult<Answer, GErr>> for ProcessResultInner<Answer, GErr> {
    fn into(self) -> ProcessResult<Answer, GErr> {
        use ProcessResultInner::*;

        match self {
            HandledAndProcessNext => unreachable!(),
            HandledAndProcessEnd(a) => ProcessResult::Handled(a),
            NoTransitions => ProcessResult::NoTransitions,
            GuardErr(e) => ProcessResult::GuardErr(e),
            EventTypeNotSatisfy => ProcessResult::NoTransitions,
        }
    }
}

impl<A, GErr> ProcessResultInner<A, GErr> {
    pub fn map<ANew>(self, f: impl Fn(A) -> ANew) -> ProcessResultInner<ANew, GErr> {
        use ProcessResultInner::*;

        match self {
            HandledAndProcessNext => HandledAndProcessNext,
            HandledAndProcessEnd(a) => HandledAndProcessEnd(f(a)),
            NoTransitions => NoTransitions,
            GuardErr(e) => GuardErr(e),
            EventTypeNotSatisfy => ProcessResultInner::EventTypeNotSatisfy,
        }
    }
}
