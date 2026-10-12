fn f(x:&[i32])->usize{match x{[_,tail @ ..]=>tail.len(),[]=>0}}
