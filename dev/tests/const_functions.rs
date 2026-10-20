//! Const permission follows selected declarations, even in unused and dead code.
#[path = "helpers/source_cases.rs"]
mod source_cases;

const CASES: &[(&str, bool, &str)] = &[
    ("scalar", true, "const fn f(x:u32)->u32{x+1}"),
    (
        "direct-const-call",
        true,
        "const fn f(x:u32)->u32{g(x)}const fn g(x:u32)->u32{x+1}",
    ),
    (
        "recursive-call",
        true,
        "const fn f(x:u32)->u32{if x==0{1}else{x*f(x-1)}}",
    ),
    (
        "mutual-recursion",
        true,
        "const fn f(x:u32)->u32{if x==0{0}else{g(x-1)}}const fn g(x:u32)->u32{f(x)}",
    ),
    (
        "function-item-local",
        true,
        "const fn g(x:u8)->u8{x}const fn f(x:u8)->u8{let call=g;call(x)}",
    ),
    (
        "nested-runtime-declaration",
        true,
        "const fn f(x:u8)->u8{fn g(x:u8)->u8{x}x}",
    ),
    (
        "mutation-and-loops",
        true,
        "const fn f(mut n:u32)->u32{let mut sum=0;while n>0{sum+=n;n-=1;}sum}",
    ),
    (
        "transient-mutable-borrow",
        true,
        "const fn f()->u8{let mut x=1;let r=&mut x;*r+=2;x}",
    ),
    (
        "indirect-mutable-borrow",
        true,
        "const fn f(x:&mut u8){*x+=1;}",
    ),
    (
        "inherent-const-method",
        true,
        "struct S(u8);impl S{const fn get(&self)->u8{self.0}}const fn f(s:&S)->u8{s.get()}",
    ),
    (
        "generic-inherent-const",
        true,
        "struct S;impl S{const fn get<T:Copy>(x:T)->T{x}}const fn f(x:u8)->u8{S::get(x)}",
    ),
    (
        "generic-const-receiver",
        true,
        "struct S<T>(T);impl<T:Copy>S<T>{const fn get(&self)->T{self.0}}const fn f(s:&S<u8>)->u8{s.get()}",
    ),
    (
        "constructor",
        true,
        "struct S(u8);const fn f(x:u8)->S{S(x)}",
    ),
    (
        "enum-constructor",
        true,
        "enum E{V(u8)}const fn f(x:u8)->E{E::V(x)}",
    ),
    (
        "numeric-pointer",
        true,
        "const fn f(x:usize)->*const u8{x as *const u8}",
    ),
    (
        "pointer-pointer",
        true,
        "const fn f(x:*const u8)->*const u32{x as *const u32}",
    ),
    (
        "runtime-call",
        false,
        "fn g(x:u8)->u8{x}const fn f(x:u8)->u8{g(x)}",
    ),
    (
        "dead-runtime-call",
        false,
        "fn g()->u8{1}const fn f()->u8{if false{g()}else{0}}",
    ),
    (
        "unreachable-runtime-call",
        true,
        "#[allow(unreachable_code)]const fn f()->u8{return 0;g()}fn g()->u8{1}",
    ),
    (
        "unreachable-pointer-integer",
        true,
        "#[allow(unreachable_code)]const fn f(x:*const u8)->usize{return 0;x as usize}",
    ),
    (
        "function-pointer-call",
        false,
        "const fn f(call:fn(u8)->u8,x:u8)->u8{call(x)}",
    ),
    (
        "reified-const-call",
        false,
        "const fn g(x:u8)->u8{x}const fn f(x:u8)->u8{let call:fn(u8)->u8=g;call(x)}",
    ),
    (
        "reified-constructor-call",
        false,
        "struct S(u8);const fn f(x:u8)->S{let call:fn(u8)->S=S;call(x)}",
    ),
    (
        "inherent-runtime-method",
        false,
        "struct S(u8);impl S{fn get(&self)->u8{self.0}}const fn f(s:&S)->u8{s.get()}",
    ),
    (
        "trait-runtime-method",
        false,
        "trait Get{fn get(&self)->u8;}const fn f<T:Get>(x:&T)->u8{x.get()}",
    ),
    (
        "qualified-runtime-method",
        false,
        "struct S;impl S{fn get()->u8{1}}const fn f()->u8{S::get()}",
    ),
    (
        "pointer-integer",
        false,
        "const fn f(x:*const u8)->usize{x as usize}",
    ),
    (
        "dead-pointer-integer",
        false,
        "const fn f(x:*const u8)->usize{if false{x as usize}else{0}}",
    ),
    (
        "pointer-equality",
        false,
        "const fn f(x:*const u8,y:*const u8)->bool{x==y}",
    ),
    (
        "pointer-ordering",
        false,
        "const fn f(x:*const u8,y:*const u8)->bool{x<y}",
    ),
    (
        "ordinary-runtime-context",
        true,
        "fn g(x:u8)->u8{x}fn f(x:u8)->u8{g(x)}fn addr(p:*const u8)->usize{p as usize}",
    ),
];

#[test]
fn declared_const_operations_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    source_cases::check("const-functions-eligibility", CASES)
}

const DESTRUCTION: &[(&str, bool, &str)] = &[
    (
        "returned-destructor",
        true,
        "struct S;impl Drop for S{fn drop(&mut self){}}const fn f()->S{S}",
    ),
    ("generic-transfer", true, "const fn f<T>(x:T)->T{x}"),
    (
        "generic-conditional-transfer",
        true,
        "const fn f<T>(x:T,b:bool)->T{if b{return x;}x}",
    ),
    ("copy-parameter", true, "const fn f<T:Copy>(x:T){let _=x;}"),
    (
        "zero-sized-array",
        true,
        "struct S;impl Drop for S{fn drop(&mut self){}}const fn f(){let _x:[S;0]=[];}",
    ),
    (
        "ordinary-runtime-drop",
        true,
        "struct S;impl Drop for S{fn drop(&mut self){}}fn f(){let _x=S;}",
    ),
    (
        "unreachable-drop",
        true,
        "struct S;impl Drop for S{fn drop(&mut self){}}#[allow(unreachable_code)]const fn f(){return;let _x=S;}",
    ),
    (
        "local-drop",
        false,
        "struct S;impl Drop for S{fn drop(&mut self){}}const fn f(){let _x=S;}",
    ),
    (
        "argument-drop",
        false,
        "struct S;impl Drop for S{fn drop(&mut self){}}const fn f(_x:S){}",
    ),
    ("generic-drop", false, "const fn f<T>(_x:T){}"),
    (
        "generic-conditional-drop",
        false,
        "const fn f<T>(_x:T,b:bool){if b{return;}loop{}}",
    ),
    (
        "dead-conditional-drop",
        false,
        "struct S;impl Drop for S{fn drop(&mut self){}}const fn f(){if false{let _x=S;}}",
    ),
    (
        "discarded-temporary",
        false,
        "struct S;impl Drop for S{fn drop(&mut self){}}const fn f(){let _=S;}",
    ),
    (
        "zero-repeat-evaluates-operand",
        false,
        "struct S;impl Drop for S{fn drop(&mut self){}}const fn f(){let _x=[S;0];}",
    ),
    (
        "generic-partial-transfer",
        false,
        "const fn f<T,U>(x:(T,U))->T{x.0}",
    ),
];

#[test]
fn const_destruction_uses_selected_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    source_cases::check("const-functions-destruction", DESTRUCTION)
}

#[path = "harness/native_references.rs"]
mod native;

#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn const_functions_keep_runtime_calls_and_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "pa24-pa25-work/native-const-functions",
        &[
            (
                "recursive",
                "const fn f(n:u32)->u32{if n==0{1}else{n*f(n-1)}}fn main(){check(f(6)==720);}",
                false,
            ),
            (
                "mutation",
                "const fn f(n:u32)->u32{let mut i=0;let mut sum=0;while i<n{sum+=i;i+=1;}sum}fn main(){check(f(10)==45);}",
                false,
            ),
            (
                "returned-owned-value",
                "struct S<'a>(&'a mut u8);impl Drop for S<'_>{fn drop(&mut self){*self.0+=1;}}const fn make(x:&mut u8)->S<'_>{S(x)}fn main(){let mut n=0;{let _s=make(&mut n);}check(n==1);}",
                false,
            ),
            (
                "generic-transfer",
                "const fn identity<T>(x:T)->T{x}fn main(){check(identity((3u8,4u8)).1==4);}",
                false,
            ),
        ],
    )
}
