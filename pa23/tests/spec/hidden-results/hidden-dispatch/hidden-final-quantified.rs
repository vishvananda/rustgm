fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x=outer();check(x.f(&1)==12);}trait A{}impl A for u8{}trait B<'a>{fn f(&self,x:&'a u8)->u8;}impl<'a,T:Send>B<'a> for T{fn f(&self,x:&'a u8)->u8{*x+11}}fn outer()->impl for<'a>B<'a>{inner()}fn inner()->impl A{7u8}
