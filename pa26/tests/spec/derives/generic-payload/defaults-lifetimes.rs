#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]struct Pair<'a,T=u8>{left:&'a T,right:T}
fn main(){let n=7;let a=Pair{left:&n,right:9};let b=a.clone();println!("{} {:?}",a==b,b);}
