macro_rules! pick { ($s:literal) => { #[cfg(target_os = $s)] fn selected() {} }; } pick!(true); fn main(){selected();}
