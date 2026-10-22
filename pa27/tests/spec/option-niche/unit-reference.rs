static S:()=();union U{o:Option<&'static ()>,p:&'static ()}const N:Option<&()>=unsafe{(U{p:&S}).o};fn main(){check(match N{Some(_)=>true,None=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
