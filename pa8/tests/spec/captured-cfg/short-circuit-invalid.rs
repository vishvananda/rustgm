macro_rules! pick{($s:literal)=>{#[cfg(any(unix,target_os=$s))]fn selected(){}};}pick!(8);fn main(){}
