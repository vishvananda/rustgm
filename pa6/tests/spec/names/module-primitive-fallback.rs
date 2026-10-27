mod str{}
mod u8{pub struct Named;}
type Text=str;
fn collision(_: &Text,_:u8,_:u8::Named){}
mod shadows{struct u8;type str=u8;fn nominal(_:str){}}
