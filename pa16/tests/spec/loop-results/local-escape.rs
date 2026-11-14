pub fn run<'a>(slot:&mut &'a mut u8,stop:bool){loop{let mut x=7u8;*slot=&mut x;if stop{break}}}
