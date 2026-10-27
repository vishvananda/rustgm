pub trait Convert<R> { fn convert(value: R) -> Self; }
pub trait Product: Convert<Self::Residual> { type Residual; }
pub trait Family<O>: Sized { type Output: Product<Residual = Self>; }
pub fn bridge<R: Family<O>, O>(value: R) -> R::Output {
    <R::Output as Convert<R>>::convert(value)
}
fn main() {}
