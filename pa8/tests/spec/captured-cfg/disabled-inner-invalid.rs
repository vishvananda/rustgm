macro_rules! pick{($s:literal)=>{#[cfg_attr(any(),cfg(target_os=$s))]fn selected(){}};}pick!(8);fn main(){selected();}
