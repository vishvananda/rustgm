pub fn run<'a>(make:fn()->u8,slot:&mut &'a mut u8,stop:bool){loop{*slot=&mut make();if stop{break}}}
