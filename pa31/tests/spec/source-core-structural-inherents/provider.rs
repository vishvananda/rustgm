impl Clone for u32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u32 {}

impl ! {}
impl () {}
impl<T> (T,) {}
impl<Ret, T> fn(T) -> Ret {}

impl () {
    pub const UNIT_CONST: u32 = 11;

    pub fn unit_tag(&self) -> u32 {
        17
    }
}

impl<T> (T,) {
    pub fn first_ref(&self) -> &T {
        &self.0
    }
}

impl<'a, T> &'a T {
    pub fn reborrow_tag(self) -> &'a T {
        self
    }
}

impl<T: Copy> &mut T {
    pub fn write_copy(&mut self, value: T) {
        **self = value;
    }
}

impl<Ret, T> fn(T) -> Ret {
    pub fn hidden_fn_method(&self) -> u32 {
        99
    }
}
