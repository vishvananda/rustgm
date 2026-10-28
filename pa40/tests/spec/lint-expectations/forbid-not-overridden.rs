#![forbid(deprecated)] #[deprecated] fn old(){} #[expect(deprecated)] fn main(){old();}
