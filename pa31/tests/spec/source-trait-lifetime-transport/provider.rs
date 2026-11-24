#[lang="freeze"]
pub unsafe auto trait Freeze {}

pub trait StaticOnly: 'static {}

pub trait Borrowable<'a>: 'a {
    fn payload(&self) -> u8;
}
