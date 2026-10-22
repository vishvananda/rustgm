struct IterMut;trait Local{fn next(&mut self)->u8;}impl Local for IterMut{fn next(&mut self)->u8{7}}fn main(){let mut i=IterMut;if i.next()!=7{let z=0;let _=1/z;}}
