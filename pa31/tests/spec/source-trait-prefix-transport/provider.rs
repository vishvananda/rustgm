#[lang="freeze"]
pub unsafe auto trait Freeze {}

pub trait Parent {
    type Output;
    fn produce(&self) -> Self::Output;
}
pub trait Marker {
    fn score(&self) -> u8;
}
pub trait Child: Parent<Output = u8> where Self::Other: Marker {
    type Other;
}
