macro_rules! local {()=>{{let value=7;format!("{value}")}}}
macro_rules! literal {($s:literal)=>{format!($s)}}
macro_rules! expression {($s:expr)=>{format!($s)}}
fn main(){let value=3;let __format_values=11;println!("{} {} {} {}",local!(),literal!("{value}"),expression!("{value}"),format!("{__format_values}"));}
