struct S;impl Drop for S{fn drop(&mut self){Self::drop(self);}}fn main(){}
