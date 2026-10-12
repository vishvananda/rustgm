trait A {type Output; fn f(&self)->Self::Output;} impl A for i32 {type Output=i32; fn f(&self)->i32{*self}}
