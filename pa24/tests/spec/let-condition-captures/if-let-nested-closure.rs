fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct It{n:u32}
impl It{fn next(&mut self)->Option<u32>{if self.n==0{None}else{self.n-=1;Some(self.n)}}}
fn apply<F:Fn(u32)->u32>(it:&mut It,g:F)->Option<u32>{match it.next(){Some(x)=>Some(g(x)),None=>None}}
fn run(k:u32)->u32{let f=|it:&mut It|{if let Some(v)=apply(it,|x|x+k){v}else{0}};f(&mut It{n:3})}
fn main(){check(run(5)==7);}
