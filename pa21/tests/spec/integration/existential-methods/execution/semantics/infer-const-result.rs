fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Make{fn make<const N:usize>()->impl Copy;}impl Make for u8{fn make<const N:usize>()->[u8;N]{[97;N]}}fn main(){let a:[u8;3]=<u8 as Make>::make();check(a[2]==97);}
