#![allow(dead_code,unused_variables)]
pub fn make<T>(slice:&[T])->Option<&[T]>{Some(slice)}
pub fn probe0<T>(input:Option<u8>,slice:&[T])->Option<&[T]>{input?;make(slice)}
pub fn probe1<T>(input:Option<u8>,slice:&[T])->Option<&[T]>{input?;make(slice)}
pub fn probe2<T>(input:Option<u8>,slice:&[T])->Option<&[T]>{input?;make(slice)}
pub fn probe3<T>(input:Option<u8>,slice:&[T])->Option<&[T]>{input?;make(slice)}
pub fn probe4<T>(input:Option<u8>,slice:&[T])->Option<&[T]>{input?;make(slice)}
pub fn probe5<T>(input:Option<u8>,slice:&[T])->Option<&[T]>{input?;make(slice)}
pub fn probe6<T>(input:Option<u8>,slice:&[T])->Option<&[T]>{input?;make(slice)}
pub fn probe7<T>(input:Option<u8>,slice:&[T])->Option<&[T]>{input?;make(slice)}
fn main(){}
