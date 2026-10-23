#[macro_export]
macro_rules! make{()=>{fn probe()->u8{let x=if true{Maybe::None}else{Maybe::Some(return 0)};consume(x)}}}
