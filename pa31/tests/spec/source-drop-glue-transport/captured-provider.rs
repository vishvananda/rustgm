pub trait Tagged {}
impl Tagged for u8 {}

#[repr(C)]
pub struct Bounded<'a, 'b: 'a, T: Tagged, const N: usize> {
    input: &'a T,
    other: &'b u8,
    marks: [u8; N],
    pub state: u8,
}

impl<'later: 'earlier, 'earlier, const M: usize, U: Tagged> Drop
    for Bounded<'earlier, 'later, U, M>
{
    fn drop(&mut self) {
        self.state = 37;
    }
}

#[repr(C)]
pub struct Conditional<'a> {
    input: &'a u8,
    pub state: u8,
}

const impl<'b> Drop for Conditional<'b> {
    fn drop(&mut self) {
        self.state = 61;
    }
}
