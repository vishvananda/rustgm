//! Auto promises constrain conversion while runtime dispatch uses ordinary tables.
const CASES: &[(&str, &str, bool)] = &[
    (
        "auto-explicit-dispatch",
        "trait A{fn f(&self)->u8;}struct W(*const u8,u8);unsafe impl Send for W{}impl A for W{fn f(&self)->u8{self.1}}fn main(){let w=W(0usize as *const u8,7);let p:&(dyn A+Send)=&w;check(p.f()==7);}",
        false,
    ),
    (
        "auto-conditional-slot",
        "trait A{fn f(&self)->u8 where Self:Send{7}}struct W(*const u8);unsafe impl Send for W{}impl A for W{}fn main(){let w=W(0usize as *const u8);let p:&(dyn A+Send)=&w;check(p.f()==7);}",
        false,
    ),
    (
        "auto-vacant-slot",
        "trait A{fn f(&self)->u8 where Self:Send{7}fn g(&self)->u8{9}}struct W(*const u8);impl A for W{}fn main(){let w=W(0usize as *const u8);let p:&dyn A=&w;check(p.g()==9);}",
        false,
    ),
    (
        "auto-dispatch",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self+2}}fn main(){let x=7u8;let p:&(dyn A+Send+Sync)=&x;check(p.f()==9);}",
        false,
    ),
    (
        "auto-drop",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn erase(x:&(dyn A+Send))->&dyn A{x}fn main(){let x=7u8;let p:&(dyn A+Send)=&x;check(erase(p).f()==7);}",
        false,
    ),
    (
        "auto-inherited",
        "trait A:Send{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn add(x:&dyn A)->&(dyn A+Send){x}fn main(){let x=7u8;check(add(&x).f()==7);}",
        false,
    ),
    (
        "auto-only-data",
        "fn main(){let x=7u8;let p:&dyn Send=&x;check(p as *const dyn Send as *const u8==&x as *const u8);}",
        false,
    ),
    (
        "auto-only-upcast",
        "trait A{}impl A for u8{}fn erase(x:&(dyn A+Send))->&dyn Send{x}fn main(){let x=7u8;let p=erase(&x);check(p as *const dyn Send as *const u8==&x as *const u8);}",
        false,
    ),
    (
        "auto-only-drop-bound",
        "fn erase(x:&(dyn Send+Sync))->&dyn Send{x}fn main(){let x=7u8;let p=erase(&x);check(p as *const dyn Send as *const u8==&x as *const u8);}",
        false,
    ),
    (
        "auto-only-constant",
        "const P:&dyn Send=&7u8;fn need<T:?Sized+Send>(_:&T)->u8{9}fn main(){check(need(P)==9);}",
        false,
    ),
];

#[test]
fn marker_objects_lower() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, _) in CASES {
        let source = format!("{}{source}", super::native::CHECK);
        super::rustc::qualify(name, &source, true)?;
        if let Err(error) = super::build(&source) {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn marker_objects_execute() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("pa24-pa25-work/native-auto-objects", CASES)
}
