const fn count()->usize{2}
const fn take<T>(a:usize,_b:usize)->usize{a}
#[derive(Clone,Copy)]struct Direct([u32;count()]);
#[derive(Clone,Copy)]struct Generic([u32;take::<u8>(2,0)]);
#[derive(Clone,Copy)]struct Sized([*const ();16/core::mem::size_of::<*const ()>()]);
fn main(){let a=Direct([3,5]).clone();let b=Generic([5,7]).clone();let c=Sized([core::ptr::null();2]).clone();assert_eq!(c.0.len(),2);println!("{} {} {}",a.0[1],b.0[1],c.0.len());}
