struct It{n:u32}
impl It{fn next(&mut self)->Option<u32>{if self.n==0{None}else{self.n-=1;Some(self.n)}}}
struct N(u32);
fn take(n:N)->u32{n.0}
fn main(){let n=N(5);let f=move|it:&mut It|{let mut c=0;while let Some(_)=it.next(){c+=take(n);}c};let _=f(&mut It{n:2});}
