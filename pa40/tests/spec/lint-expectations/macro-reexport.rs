#[deprecated] #[macro_export] macro_rules! old {()=>{()}} mod prelude {#[expect(deprecated)] pub use crate::old;} fn main(){}
