use frunk::{HCons, HNil};

pub trait Guard<Input> {
    fn check(&self, input: &Input) -> bool;
}

impl<Input, F> Guard<Input> for F
where
    F: Fn(&Input) -> bool,
{
    fn check(&self, input: &Input) -> bool {
        self(input)
    }
}

impl<Input> Guard<Input> for HNil {
    fn check(&self, _: &Input) -> bool {
        true
    }
}

impl<Input, F, Rest> Guard<Input> for HCons<F, Rest>
where
    F: Guard<Input>,
    Rest: Guard<Input>,
{
    fn check(&self, input: &Input) -> bool {
        self.head.check(input) && self.tail.check(input)
    }
}
