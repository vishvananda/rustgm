macro_rules! body {($e:expr)=>{loop{break $e;}}} fn f()->u64{body!(9)}
