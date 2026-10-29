const COUNT:usize=2;
#[derive(Clone,Copy)]struct Literal([u32;2]);
#[derive(Clone,Copy)]struct Named([u32;COUNT]);
#[derive(Clone,Copy)]struct Compound([u32;{(1usize+3usize)/2usize}]);
fn main(){let a=Literal([3,5]);let b=Named([5,7]);let c=Compound([7,11]);let aa=a.clone();let bb=b.clone();let cc=c.clone();assert_eq!(aa.0[1],5);assert_eq!(bb.0[1],7);assert_eq!(cc.0[1],11);println!("literal named compound 5 7 11");}
