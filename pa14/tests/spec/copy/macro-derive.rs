macro_rules! m{()=>{#[derive(Clone,Copy)] struct N;}} m!(); fn f(x:N){let _a=x;let _b=x;}
