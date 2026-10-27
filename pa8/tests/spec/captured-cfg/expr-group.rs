macro_rules! pick { ($s:expr) => { #[cfg(target_os = $s)] fn selected() {} }; } pick!(("linux")); fn main(){selected();}
