fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct It{n:u32}
impl It{fn next(&mut self)->Option<u32>{if self.n==0{None}else{self.n-=1;Some(self.n)}}}
fn main(){let mut total=0;let mut it=It{n:4};{let mut f=|it:&mut It|{while let Some(d)=it.next(){total+=d;}};f(&mut it);}check(total==6);}
