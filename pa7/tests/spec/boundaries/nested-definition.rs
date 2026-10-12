macro_rules! m { () => {macro_rules! n {($x:ident)=>{struct $x;}}} }
m!();
