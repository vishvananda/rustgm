fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check([1u8,9].min([2,0])==[1,9]);check([1u8,9].max([2,0])==[2,0]);check([1u8,9].clamp([0,0],[2,0])==[1,9]);}