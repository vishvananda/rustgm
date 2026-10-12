macro_rules! m { () => {fn f()->i32{$crate::value()}
fn value()->i32{1}} }
m!();
