pub fn run<'a>(make:fn()->&'a mut u8,slot:&mut &'a mut u8,stop:bool){loop{*slot=make();if stop{break}}}
