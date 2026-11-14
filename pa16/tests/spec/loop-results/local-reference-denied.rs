pub fn run<'a>(make:fn()->&'a mut u8,slot:&mut &'a mut u8){let reference=make();*slot=&mut *reference;let another=&mut *reference;*another=7;}
