pub fn run<'a>(make:fn(&mut u8)->&'a mut u8,cursor:&mut u8,slot:&mut &'a mut u8,stop:bool){loop{*slot=make(cursor);if stop{break}}}
