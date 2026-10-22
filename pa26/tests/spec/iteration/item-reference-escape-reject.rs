fn check(b:bool){if !b{let z=0;let _=1/z;}}struct I{v:u8}impl core::iter::Iterator for I{type Item=&'static u8;fn next(&mut self)->Option<Self::Item>{Some(&self.v)}}fn main(){}
