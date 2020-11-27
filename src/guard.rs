use frunk::{HCons, HNil};

pub trait Guard<Input, Err> {
    fn check(&self, input: &Input) -> Result<(), Err>;
}

impl<Input, F, Err> Guard<Input, Err> for F
where
    F: Fn(&Input) -> Result<(), Err>,
{
    fn check(&self, input: &Input) -> Result<(), Err> {
        self(input)
    }
}

impl<Input, Err> Guard<Input, Err> for HNil {
    fn check(&self, _: &Input) -> Result<(), Err> {
        Ok(())
    }
}

impl<Input, F, Rest, Err> Guard<Input, Err> for HCons<F, Rest>
where
    F: Guard<Input, Err>,
    Rest: Guard<Input, Err>,
{
    fn check(&self, input: &Input) -> Result<(), Err> {
        self.head.check(input).and_then(|_| self.tail.check(input))
    }
}
