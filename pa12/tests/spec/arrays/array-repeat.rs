fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
#[derive(Clone,Copy)]struct S(u64,u8); fn arr()->[S;7]{[S(12,3);7]} fn main(){let mut a=arr();a[5].0=20;check(a[0].0==12 && a[5].0==20 && a[6].1==3);}
