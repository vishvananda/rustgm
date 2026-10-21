macro_rules! decl{()=>{unsafe extern "C"{safe fn imported(x:i32)->i32;}}}decl!();fn main(){let _=imported(39);}
