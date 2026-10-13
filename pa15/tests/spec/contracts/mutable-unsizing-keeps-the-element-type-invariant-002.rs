fn f<'a:'b,'b>(x:&'b mut [&'a u8;1])->&'b mut [&'b u8]{x}
fn main() {}
