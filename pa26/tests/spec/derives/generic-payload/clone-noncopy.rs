use std::sync::atomic::{AtomicUsize,Ordering};
static COPIES:AtomicUsize=AtomicUsize::new(0);
struct Leaf(u8);impl Clone for Leaf{fn clone(&self)->Self{COPIES.fetch_add(1,Ordering::Relaxed);Self(self.0)}}
#[derive(Clone)]struct Wrap<T>(T);
#[derive(Clone)]enum Choice<T>{Unit,Tuple(T),Record{field:T}}
fn main(){let a=Wrap(Leaf(7));let b=a.clone();let c=Choice::Tuple(Leaf(9));let d=c.clone();let e=Choice::Record{field:Leaf(11)};let f=e.clone();let n=match d{Choice::Tuple(x)=>x.0,_=>0};let m=match f{Choice::Record{field:x}=>x.0,_=>0};let _:Choice<Leaf>=Choice::Unit.clone();println!("{} {} {} {}",b.0.0,n,m,COPIES.load(Ordering::Relaxed));}
