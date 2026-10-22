fn check(b:bool){if !b{let z=0;let _=1/z;}}use std::primitive::usize as Word;fn main(){check(Word::MAX==18446744073709551615usize);}
