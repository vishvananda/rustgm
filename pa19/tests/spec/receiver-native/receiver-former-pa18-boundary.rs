fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Get{fn get(&self)->u8;}struct S;impl Get for S{fn get(&self)->u8{7}}fn main(){let _=S.get();}
