struct S;type R<'a>=&'a mut S;impl Drop for S{fn drop(self:R<'_>){}}fn main(){}
