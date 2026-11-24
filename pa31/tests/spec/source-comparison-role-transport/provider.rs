#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[lang = "eq"]
pub const trait PartialEq<Rhs: PointeeSized = Self>: PointeeSized {
    fn eq(&self, other: &Rhs) -> bool;

    fn ne(&self, other: &Rhs) -> bool {
        if self.eq(other) { false } else { true }
    }
}

pub struct Key(pub u8);
pub struct Right(pub u8);

impl PartialEq for Key {
    fn eq(&self, _: &Self) -> bool {
        match self.0 { 0 => true, _ => false }
    }
}

impl PartialEq<Right> for Key {
    fn eq(&self, other: &Right) -> bool {
        match other.0 { 0 => true, _ => false }
    }
}

#[lang = "Option"]
pub enum Option<T> {
    #[lang = "None"]
    None,
    #[lang = "Some"]
    Some(T),
}

#[lang = "Ordering"]
#[repr(i8)]
pub enum Ordering {
    Less = 255u8 as i8,
    Equal = 0,
    Greater = 1,
}

pub enum ControlFlow<B, C = ()> {
    Continue(C),
    Break(B),
}

#[lang = "partial_ord"]
pub const trait PartialOrd<Rhs: PointeeSized = Self>: [const] PartialEq<Rhs> + PointeeSized {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;
    fn lt(&self, other: &Rhs) -> bool {
        let comparison = self.partial_cmp(other);
        match comparison {
            Option::Some(Ordering::Less) => true,
            _ => false,
        }
    }
    fn le(&self, other: &Rhs) -> bool {
        let comparison = self.partial_cmp(other);
        match comparison {
            Option::Some(Ordering::Less | Ordering::Equal) => true,
            _ => false,
        }
    }
    fn gt(&self, other: &Rhs) -> bool {
        let comparison = self.partial_cmp(other);
        match comparison {
            Option::Some(Ordering::Greater) => true,
            _ => false,
        }
    }
    fn ge(&self, other: &Rhs) -> bool {
        let comparison = self.partial_cmp(other);
        match comparison {
            Option::Some(Ordering::Greater | Ordering::Equal) => true,
            _ => false,
        }
    }
    fn __chaining_lt(&self, other: &Rhs) -> ControlFlow<bool> {
        ControlFlow::Break(self.lt(other))
    }
    fn __chaining_le(&self, other: &Rhs) -> ControlFlow<bool> {
        ControlFlow::Break(self.le(other))
    }
    fn __chaining_gt(&self, other: &Rhs) -> ControlFlow<bool> {
        ControlFlow::Break(self.gt(other))
    }
    fn __chaining_ge(&self, other: &Rhs) -> ControlFlow<bool> {
        ControlFlow::Break(self.ge(other))
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, _: &Self) -> Option<Ordering> {
        match self.0 {
            0 => Option::Some(Ordering::Equal),
            1 => Option::Some(Ordering::Less),
            2 => Option::Some(Ordering::Greater),
            _ => Option::None,
        }
    }
}
