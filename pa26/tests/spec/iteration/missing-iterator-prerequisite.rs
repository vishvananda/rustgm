fn check(b:bool){if !b{let z=0;let _=1/z;}}struct Bad;struct S;impl core::iter::IntoIterator for S{type Item=u8;type IntoIter=Bad;fn into_iter(self)->Bad{Bad}}fn main(){}
