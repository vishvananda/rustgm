struct R;impl core::iter::Iterator for R{type Item=u8;fn next(&mut self)->Option<u8>{None}}impl<'a>core::iter::Iterator for &'a mut R{type Item=u8;fn next(&mut self)->Option<u8>{None}}fn main(){}
