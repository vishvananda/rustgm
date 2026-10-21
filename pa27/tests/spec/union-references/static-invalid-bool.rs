union U{bits:u32,value:bool}static S:U=U{bits:2};const R:&bool={let p=&raw const S.value;unsafe{&*p}};const X:bool=*R;fn main(){let _=X;}
