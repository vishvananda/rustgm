static mut TRACE:u64=0;
fn mark(n:u8){unsafe{TRACE=TRACE*10+(n as u64);}}
struct D(u8);
impl Drop for D{fn drop(&mut self){mark(self.0);}}
fn temp(n:u8)->D{D(n)}
fn see(_: &D)->bool{true}
fn run(){let x={let local=temp(1);see(&temp(2))};mark(3);}
fn main(){run();unsafe{let _=1u8/((TRACE==123) as u8);}}
