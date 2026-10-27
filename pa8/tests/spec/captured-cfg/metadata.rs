macro_rules! pick{($p:meta)=>{#[cfg($p)]fn selected(){}};}pick!(target_os="linux");fn main(){selected();}
