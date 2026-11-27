pub trait Item: Sized {}

pub trait Inherited: Item {
    fn slice(_: &[Self]) -> u32 { 23 }
    fn array(_: &[Self; 2]) -> u32 { 37 }
}

pub trait Scoped {
    fn slice(_: &[Self]) -> u32 where Self: Sized { 61 }
}

impl Item for u32 {}
impl Inherited for u32 {}
impl Scoped for u32 {}
