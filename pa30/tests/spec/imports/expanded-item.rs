macro_rules! decl{()=>{safe fn imported(x:i32)->i32;}}unsafe extern "C"{decl!();}fn main(){let _=imported(39);}
