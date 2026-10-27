macro_rules! pick { ($s:literal) => { #[cfg(target_os = $s)] fn selected() {} }; } pick!("\x6cinux"); fn main(){selected();}
