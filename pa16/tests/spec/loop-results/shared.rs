pub fn run<'a>(make:fn()->&'a u8,slot:&mut &'a u8,stop:bool){loop{*slot=make();if stop{break}}}
