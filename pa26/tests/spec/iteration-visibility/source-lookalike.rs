fn check(b:bool){if !b{let z=0;let _=1/z;}}trait Iterator{fn next(&mut self)->u8;}struct L;impl Iterator for L{fn next(&mut self)->u8{9}}fn main(){let mut l=L;check(l.next()==9);}
