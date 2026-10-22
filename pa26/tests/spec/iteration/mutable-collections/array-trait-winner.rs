trait Local{fn iter_mut(&mut self)->u8;}impl Local for [u8;2]{fn iter_mut(&mut self)->u8{7}}fn main(){let mut a=[1u8,2];if a.iter_mut()!=7{let z=0;let _=1/z;}}
