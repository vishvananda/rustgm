fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const C:[f32;0]=[0.0/0.0;0];fn main(){match []{C=>()}check(true);}