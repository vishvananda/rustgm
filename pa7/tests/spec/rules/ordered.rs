macro_rules! m {(first)=>{};($e:expr, no)=>{};($e:expr, yes)=>{};} m!(first); m!(1+2, yes);
