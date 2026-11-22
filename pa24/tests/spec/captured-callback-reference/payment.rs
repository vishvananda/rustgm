fn fail(zero: u8) -> u8 {
    1 / zero
}

enum Outcome<T, E> {
    Okay(T),
    Error(E),
}

struct Budget {
    remaining: usize,
}

impl Budget {
    fn pay(&self, steps: usize) -> Outcome<(), &'static str> {
        if steps <= self.remaining {
            Outcome::Okay(())
        } else {
            Outcome::Error("exhausted")
        }
    }
}

fn register(steps: usize, pay: &dyn Fn(usize) -> Outcome<(), &'static str>) -> bool {
    match pay(steps) {
        Outcome::Okay(()) => true,
        Outcome::Error(_) => false,
    }
}

fn check_budget(budget: &Budget, steps: usize) -> bool {
    let pay = |steps| budget.pay(steps);
    let eligible = |steps| register(steps, &pay);
    eligible(steps)
}

fn main() {
    let budget = Budget { remaining: 42 };
    if !check_budget(&budget, 42) || check_budget(&budget, 43) {
        let _ = fail(0);
    }
}
