union U{word:u32,byte:u8}const IMAGE:U=U{word:37};const fn read(u:&U)->u32{unsafe{u.word}}const X:u32=read(&IMAGE);fn main(){let _=X;}
