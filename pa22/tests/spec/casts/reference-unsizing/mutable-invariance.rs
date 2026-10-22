fn cast<'a,'b>(p:&'b mut [&'a u8;1])->&'b mut [&'b u8] where 'a:'b{p as &'b mut [&'b u8]}fn main(){}
