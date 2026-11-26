pub struct Reading(pub u8);

const _: u8 = {
    struct Probe(u8);
    const fn hidden(input: Probe) -> u8 {
        input.0
    }
    impl Reading {
        pub fn first(&self) -> u8 {
            hidden(Probe(self.0)) ^ 3
        }
    }
    hidden(Probe(7))
};

const _: () = {
    struct Probe(u8);
    const fn hidden(input: Probe) -> u8 {
        input.0 ^ 5
    }
    impl Reading {
        pub fn second(&self) -> u8 {
            hidden(Probe(self.0))
        }
    }
};

pub const NAMED: u8 = {
    const fn hidden() -> u8 {
        17
    }
    hidden()
};

pub static STORED: u8 = {
    const fn hidden() -> u8 {
        29
    }
    hidden()
};
