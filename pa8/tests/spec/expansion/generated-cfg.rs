macro_rules! m{()=>{#[cfg(any())] fn disabled(){missing!();} fn enabled(){}}} m!();
