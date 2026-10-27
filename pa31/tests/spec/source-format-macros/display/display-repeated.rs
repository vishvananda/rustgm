use std::sync::atomic::{AtomicUsize, Ordering};
static TRACE:AtomicUsize=AtomicUsize::new(0);
fn mark(n:usize){TRACE.fetch_update(Ordering::SeqCst,Ordering::SeqCst,|v|Some(v*10+n)).unwrap();}
struct Probe(u32);
fn make(n:u32)->Probe{mark(n as usize);Probe(n)}
impl std::fmt::Display for Probe { fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{mark(4);write!(f,"M{}",self.0)} }
impl std::fmt::Debug for Probe { fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{mark(3);write!(f,"D{}",self.0)} }
fn main(){let out=format!("{0}|{0}",make(1));println!("{out}|{}",TRACE.load(Ordering::SeqCst));}
