//! Results that are returned from different interfaces.

/// An result of processing event.
///
/// - `Handled` - event handled and `Answer` is returned.
/// - `NoTransitions` - event not handled because there are no transitions from this `Source` vertex
/// that give a specified `Event` type.
/// - `GuardErr` - event not handled because `Guard` not accept it and guard error returns.
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

/// An inner result of processing event. It is need only if you implement your own `ITransition`.
///
/// - `HandledAndProcessNext` - event handled, but answer is not returned because there are required
/// at least one more step int `StateMachine`.
/// - `EventTypeNotSatisfy` - event type that received not satisfy for type of `ITransition`.
/// - `HandledAndProcessEnd` - event handled and `Answer` is returned.
/// - `NoTransitions` - event not handled because there are no transitions from this `Source` vertex
/// that give a specified `Event` type.
/// - `GuardErr` - event not handled because `Guard` not accept it and guard error returns.
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

/// An inner result of processing event by substate.
///
/// - `Handled` - event handled and `Answer` is returned.
/// - `NoTransitions` - event not handled because there are no transitions from this `Source` vertex
/// that give a specified `Event` type.
/// - `GuardErr` - event not handled because `Guard` not accept it and guard error returns.
/// - `MustLeaveState` - state machine must leave substate and make transition to another vertex.
pub enum ProcessResultSubstate<Answer, GErr> {
    Handled(Answer),
    NoTransitions,
    GuardErr(GErr),
    MustLeaveState,
}
