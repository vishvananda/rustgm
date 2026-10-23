struct Counter(u8);impl base::Step for Counter{fn step(self)->u8{self.0}}fn main(){let _=other::invoke(Counter(7));}
