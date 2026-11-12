fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct It{n:u32}
impl It{fn next(&mut self)->Option<u32>{if self.n==0{None}else{self.n-=1;Some(self.n)}}}
fn run(it:&mut It,radix:u32,max:u32)->Option<u32>{let f=|it:&mut It|{let mut result=0;let mut count=0;while let Some(d)=it.next(){if count>=max{return None;}result*=radix;result+=d;count+=1;}Some(result)};f(it)}
fn main(){check(match run(&mut It{n:3},10,3){Some(v)=>v==210,None=>false});check(match run(&mut It{n:4},10,3){Some(_)=>false,None=>true});}
