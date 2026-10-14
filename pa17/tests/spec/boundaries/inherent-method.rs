struct D;impl Drop for D{fn drop(&mut self){}}impl D{fn touch(&self){}}fn main(){let d=D;d.touch();}
