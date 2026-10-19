//! Witness lifetime hypotheses shared by rustc qualification and typed boundary tests.
pub struct Case {
    pub name: &'static str,
    pub source: &'static str,
    pub accepted: bool,
}
impl Case {
    pub fn body_source(&self) -> String {
        // The source gate is separate from the typed call consumer under test.
        // Keep every other requirement, including the ordinary sibling goal.
        self.source
            .replace("where for<'a>u8:A<'a,T>,", "where ")
            .replace("where for<'a>u8:A<'a,T>", "")
    }
}
pub const CASES: &[Case] = &[
    Case {
        name: "private-free",
        source: r"#![allow(dead_code)]
trait A<'a,T> {}
impl<'a,'b> A<'a,&'b u8> for u8 {}
fn need<T>() where for<'a>u8:A<'a,T> {}
fn main(){need();}
",
        accepted: true,
    },
    Case {
        name: "private-longer",
        source: r"#![allow(dead_code)]
trait A<'a,T> {}
impl<'a,'b> A<'a,&'b u8> for u8 where 'b:'a {}
fn need<T>() where for<'a>u8:A<'a,T> {}
fn main(){need();}
",
        accepted: true,
    },
    Case {
        name: "private-shorter",
        source: r"#![allow(dead_code)]
trait A<'a,T> {}
impl<'a,'b> A<'a,&'b u8> for u8 where 'a:'b {}
fn need<T>() where for<'a>u8:A<'a,T> {}
fn main(){need();}
",
        accepted: false,
    },
    Case {
        name: "captured-longer-unproven",
        source: r"#![allow(dead_code)]
trait A<'a,T> {}
impl<'a,'b> A<'a,&'b u8> for u8 where 'b:'a {}
fn need<T>() where for<'a>u8:A<'a,T> {}
fn use_it<'b>(){need::<&'b u8>();}
fn main(){}
",
        accepted: false,
    },
    Case {
        name: "captured-longer-static",
        source: r"#![allow(dead_code)]
trait A<'a,T> {}
impl<'a,'b> A<'a,&'b u8> for u8 where 'b:'a {}
fn need<T>() where for<'a>u8:A<'a,T> {}
fn use_it<'b:'static>(){need::<&'b u8>();}
fn main(){use_it();}
",
        accepted: true,
    },
    Case {
        name: "private-equal-through-two",
        source: r"#![allow(dead_code)]
trait A<'a,T> {}
impl<'a,'b,'c> A<'a,(&'b u8,&'c u8)> for u8 where 'a:'b,'b:'c,'c:'a {}
fn need<T>() where for<'a>u8:A<'a,T> {}
fn main(){need();}
",
        accepted: false,
    },
    Case {
        name: "unused-private",
        source: r"#![allow(dead_code)]
trait A<'a,T>{}impl<'a:'b,'b>A<'a,u16>for u8{}fn need<T>()where for<'a>u8:A<'a,T>{}fn main(){need();}
",
        accepted: true,
    },
    Case {
        name: "callable-free",
        source: r"#![allow(dead_code)]
trait A<'a,T>{}impl<'a,'b>A<'a,for<'c>fn(&'c u8,&'b u8)>for u8{}fn need<T>()where for<'a>u8:A<'a,T>{}fn main(){need();}
",
        accepted: true,
    },
    Case {
        name: "callable-shorter",
        source: r"#![allow(dead_code)]
trait A<'a,T>{}impl<'a,'b>A<'a,for<'c>fn(&'c u8,&'b u8)>for u8 where 'a:'b{}fn need<T>()where for<'a>u8:A<'a,T>{}fn main(){need();}
",
        accepted: false,
    },
    Case {
        name: "partial-free",
        source: r"#![allow(dead_code)]
trait A<'a,T>{}trait C<T>{}impl C<(&'static u8,u16)>for u8{}impl<'a,'b,U>A<'a,(&'b u8,U)>for u8 {}fn need<T>()where for<'a>u8:A<'a,T>,u8:C<T>{}fn main(){need();}
",
        accepted: true,
    },
    Case {
        name: "partial-shorter",
        source: r"#![allow(dead_code)]
trait A<'a,T>{}trait C<T>{}impl C<(&'static u8,u16)>for u8{}impl<'a,'b,U>A<'a,(&'b u8,U)>for u8 where 'a:'b{}fn need<T>()where for<'a>u8:A<'a,T>,u8:C<T>{}fn main(){need();}
",
        accepted: false,
    },
    Case {
        name: "later-witness-free",
        source: r"#![allow(dead_code)]
trait A<'a,T>{}trait B<'a,U>{}impl<'a,'c>B<'a,&'c u8>for u16 {}impl<'a,'b,V,W>A<'a,(&'b u8,V,W)>for u8 where V:B<'a,W>{}fn need<T>()where for<'a>u8:A<'a,T>{}fn main(){need::<(_,u16,_)>();}
",
        accepted: true,
    },
    Case {
        name: "later-witness-shorter",
        source: r"#![allow(dead_code)]
trait A<'a,T>{}trait B<'a,U>{}impl<'a,'c>B<'a,&'c u8>for u16 where 'a:'c{}impl<'a,'b,V,W>A<'a,(&'b u8,V,W)>for u8 where V:B<'a,W>{}fn need<T>()where for<'a>u8:A<'a,T>{}fn main(){need::<(_,u16,_)>();}
",
        accepted: false,
    },
];
