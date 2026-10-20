trait A:'static{}impl A for u8{}fn f<'a>(x:&'a(dyn A+'a))->&'static dyn A{x}fn main(){let v=7u8;let p=f(&v);let _=p;}
