struct D{n:*mut u32,w:u32}impl Drop for D{fn drop(&mut self){unsafe{*self.n+=self.w;}}}fn main(){let mut n=0u32;{let mut d=D{n:&raw mut n,w:1};d=D{n:&raw mut n,w:10};}}
