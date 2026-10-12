fn g(x:i32){} fn f(){g(#[cfg(false)] unknown(),1);}
