fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{X::<3>::C}>{fn value()->usize{3}}trait P<'a>{}trait Q<'a>{const C:usize;}trait A<'a>{const C:usize;}impl<'a,T:P<'a>>A<'a>for T{const C:usize=3;}impl<'a>P<'a>for X<{<u16 as Q<'static>>::C}>{}impl<'a>Q<'a>for u16{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
