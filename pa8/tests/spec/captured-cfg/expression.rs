macro_rules! pick { ($s:expr) => { #[cfg(target_os = $s)] fn selected() {} }; } pick!(concat!("lin","ux")); fn main(){selected();}
