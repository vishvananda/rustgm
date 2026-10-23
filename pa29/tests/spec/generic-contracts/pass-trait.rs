struct Counter(u8);impl base::Step for Counter{fn step(self)->u8{self.0+2}}fn fail(x:u8)->u8{1/x}fn main(){if base::invoke(Counter(5))!=7{let _=fail(0);}}
