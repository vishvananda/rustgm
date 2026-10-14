//! Independently classified executable destructor counter programs.
pub const CASES: &[(&str, &str, bool)] = &[
    (
        "edge-conditional",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn f(n:&mut u8,flag:bool,branch:bool){let d:D<'_>;if flag{d=D(n);}if branch{return;}}fn main(){let mut n=0;f(&mut n,false,false);f(&mut n,false,true);check(n==0);f(&mut n,true,false);check(n==1);f(&mut n,true,true);check(n==2);}",
        true,
    ),
    (
        "borrowed-owner-order",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct R(u8);impl Drop for R{fn drop(&mut self){self.0=11;}}struct D<'a>(&'a mut R);impl Drop for D<'_>{fn drop(&mut self){check(self.0.0==7);}}fn main(){let mut r=R(7);let _d=D(&mut r);}",
        true,
    ),
    (
        "extended-owner-order",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct R(u8);impl Drop for R{fn drop(&mut self){self.0=11;}}struct D<'a>(&'a mut R);impl Drop for D<'_>{fn drop(&mut self){check(self.0.0==7);}}fn main(){let _d=D(&mut R(7));}",
        true,
    ),
    (
        "counted-array",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn main(){let mut a=0;let mut b=0;let mut c=0;let mut d=0;let mut e=0;{let _items=[D(&mut a),D(&mut b),D(&mut c),D(&mut d),D(&mut e)];}check(a==1&&b==1&&c==1&&d==1&&e==1);}",
        true,
    ),
    (
        "conditional-array-gaps",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn run(flag:bool){let mut a=0;let mut b=0;let mut c=0;let mut d=0;let mut e=0;{let items=[D(&mut a),D(&mut b),D(&mut c),D(&mut d),D(&mut e)];if flag{let [first,..,last]=items;}}check(a==1&&b==1&&c==1&&d==1&&e==1);}fn main(){run(false);run(true);}",
        true,
    ),
    (
        "enum-partial",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}enum E<'a>{A(D<'a>),B{first:D<'a>,last:D<'a>},C}fn run(flag:u8){let mut a=0;let mut b=0;let mut c=0;{let e=if flag==0{E::A(D(&mut a))}else if flag==1{E::B{first:D(&mut b),last:D(&mut c)}}else{E::C};if let E::B{first,..}=e{let _moved=first;}}check(a==if flag==0{1}else{0});check(b==if flag==1{1}else{0});check(c==b);}fn main(){run(0);run(1);run(2);}",
        true,
    ),
    (
        "packed-field",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}#[repr(C,packed)]struct P<'a>{lead:u8,first:D<'a>,last:D<'a>}fn main(){let mut a=0;let mut b=0;{let _p=P{lead:3,first:D(&mut a),last:D(&mut b)};}check(a==1&&b==1);}",
        true,
    ),
    (
        "owned-index-replacement",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn run(index:usize){let mut a=0;let mut b=0;let mut c=0;let mut d=0;{let mut items=[D(&mut a),D(&mut b),D(&mut c)];items[index]=D(&mut d);}check(a==1&&b==1&&c==1&&d==1);}fn main(){run(0);run(1);run(2);}",
        true,
    ),
    (
        "custom-before-fields",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}struct O<'a>{field:D<'a>}impl Drop for O<'_>{fn drop(&mut self){check(*self.field.0==0);*self.field.0=10;}}fn main(){let mut a=0;{let _o=O{field:D(&mut a)};}check(a==11);}",
        true,
    ),
    (
        "partial-array",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn main(){let mut a=0;let mut b=0;{let [first,..]=[D(&mut a),D(&mut b)];let _moved=first;}check(a==1&&b==1);}\n",
        true,
    ),
    (
        "conditional",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn run(flag:bool)->u8{let mut n=0;{let d:D<'_>;if flag{d=D(&mut n);}}n}fn main(){check(run(false)==0);check(run(true)==1);}\n",
        true,
    ),
    (
        "move",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn main(){let mut n=0;{let a=D(&mut n);let b=a;let _c=b;}check(n==1);}\n",
        true,
    ),
    (
        "loop",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn main(){let mut n=0;let mut i=0;while i<3{let _d=D(&mut n);i+=1;if i<3{continue;}break;}check(n==3);}\n",
        true,
    ),
    (
        "return",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn pass<'a>(value:D<'a>)->D<'a>{value}fn main(){let mut n=0;{let _d=pass(D(&mut n));}check(n==1);}\n",
        true,
    ),
    (
        "let-else",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nenum E<'a>{A(D<'a>),B(D<'a>)}fn f(n:&mut u8){let E::A(ref _a)=E::B(D(n))else{check(*n==1);return;};}fn main(){let mut n=0;f(&mut n);check(n==1);}\n",
        true,
    ),
    (
        "indirect-replacement",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn replace<'a>(slot:&mut D<'a>,value:D<'a>){*slot=value;}fn main(){let mut a=0;let mut b=0;{let mut d=D(&mut a);replace(&mut d,D(&mut b));}check(a==1&&b==1);}\n",
        true,
    ),
    (
        "partial-tuple",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn main(){let mut a=0;let mut b=0;{let pair=(D(&mut a),D(&mut b));{let _first=pair.0;}}check(a==1&&b==1);}\n",
        true,
    ),
    (
        "scope",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn main(){let mut n=0;{let _d=D(&mut n);}check(n==1);}\n",
        true,
    ),
    (
        "custom-and-fields",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nstruct Outer<'a>{own:&'a mut u8,field:D<'a>}impl Drop for Outer<'_>{fn drop(&mut self){*self.own+=10;}}fn main(){let mut a=0;let mut b=0;{let _o=Outer{own:&mut a,field:D(&mut b)};}check(a==10&&b==1);}\n",
        true,
    ),
    (
        "zero-repeat",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn main(){let mut n=0;let _empty=[D(&mut n);0];check(n==1);}\n",
        true,
    ),
    (
        "early-return",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn f(n:&mut u8){let _d=D(n);return;}fn main(){let mut n=0;f(&mut n);check(n==1);}\n",
        true,
    ),
    (
        "replacement",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn main(){let mut a=0;let mut b=0;{let mut d=D(&mut a);d=D(&mut b);check(a==1);}check(a==1&&b==1);}\n",
        true,
    ),
    (
        "partial-tuple-borrow-live",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn main(){let mut a=0;let mut b=0;{let pair=(D(&mut a),D(&mut b));{let _first=pair.0;}check(a==1);check(b==0);}check(a==1&&b==1);}\n",
        false,
    ),
    (
        "partial-array-borrow-live",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}\nstruct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}\nfn main(){let mut a=0;let mut b=0;{let [first,..]=[D(&mut a),D(&mut b)];check(b==1);let _moved=first;}check(a==1&&b==1);}\n",
        false,
    ),
];
