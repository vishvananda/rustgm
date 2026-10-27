macro_rules! outer{($s:literal)=>{inner!($s);};} macro_rules! inner{($s:literal)=>{#[cfg(target_os=$s)]fn selected(){}};}outer!("linux");fn main(){selected();}
