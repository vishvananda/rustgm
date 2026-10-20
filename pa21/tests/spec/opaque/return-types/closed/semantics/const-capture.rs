fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Len{fn len(&self)->usize;}impl<const N:usize>Len for [u8;N]{fn len(&self)->usize{N}}fn make<const N:usize>()->impl Len{[7u8;N]}fn main(){check(make::<3>().len()==3);}
