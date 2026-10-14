struct S{n:u8}impl Drop for S{fn drop(&mut self){self.n=true;}}fn main(){}
