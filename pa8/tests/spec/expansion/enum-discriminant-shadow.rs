enum A{X=3}
macro_rules! value{()=>{A::X as isize}}
enum E{A=value!()}
fn main(){}
