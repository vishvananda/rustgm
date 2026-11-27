pub trait Rebind<U> {
    type Output;
}

impl<'a, T: 'a, U: 'a> Rebind<U> for &'a T {
    type Output = &'a U;
}

impl<'a, T: 'a, U: 'a> Rebind<U> for &'a mut T {
    type Output = &'a mut U;
}
