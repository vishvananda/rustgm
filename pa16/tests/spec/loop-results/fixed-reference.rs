pub fn run<'a>(make:fn()->&'a mut u8,slot:&mut &'a mut u8,stop:bool){let reference=make();loop{*slot=&mut *reference;if stop{break}}}
