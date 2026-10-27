macro_rules! pick{($s:literal)=>{#[cfg_attr(all(any(),target_os=$s),allow(dead_code))]fn selected(){}};}pick!(8);fn main(){selected();}
