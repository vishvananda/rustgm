#![feature(stmt_expr_attributes)]
struct N;fn take(_:N){}pub fn check(x:&N){let f=#[cold]||take(*x);f()}
