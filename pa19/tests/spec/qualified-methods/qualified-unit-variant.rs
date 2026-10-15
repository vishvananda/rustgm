fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{V}impl E{fn V()->u8{9}}const C:E=<E>::V;fn main(){let E::V=C;let E::V=<E>::V;}