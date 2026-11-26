pub struct Wrapped<const N: usize>(pub u8);

pub trait Combine<Rhs> {
    fn combine(self, rhs: Rhs) -> Self;
}

impl<'a, const N: usize> Combine<&'a Self> for Wrapped<N> {
    fn combine(self, rhs: &'a Self) -> Self {
        Wrapped(self.0 ^ rhs.0)
    }
}

fn pass<F>(value: F) -> F {
    value
}

pub fn apply<'a, const N: usize>(left: Wrapped<N>, right: &'a Wrapped<N>) -> u8 {
    let add = pass(Combine::combine);
    add(left, right).0
}

pub fn apply_through_tuple<'a, const N: usize>(left: Wrapped<N>, right: &'a Wrapped<N>) -> u8 {
    let (combine,) = pass((Combine::combine,));
    combine(left, right).0
}
