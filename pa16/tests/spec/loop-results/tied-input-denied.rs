pub fn run<'a>(make:fn(&'a mut u8)->&'a mut u8,input:&'a mut u8,slot:&mut &'a mut u8,stop:bool){loop{*slot=make(input);if stop{break}}}
