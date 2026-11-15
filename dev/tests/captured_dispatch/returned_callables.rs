//! A returned `impl Fn` converts to a callable object through its hidden closure.
use super::{checked, native};
use std::path::PathBuf;

const ADDER: &str = "fn adder(n:u8)->impl Fn(u8)->u8{move|x|n+x}\n";
const COUNTER: &str = "fn counter(s:u8)->impl FnMut()->u8{let mut n=s;move||{n+=1;n}}\n";
/// Each body follows the shared `check` prefix and its opaque constructors.
const CASES: &[(&str, &str)] = &[
    (
        "shared",
        "fn main(){let p:&dyn Fn(u8)->u8=&adder(5);check(p(2)==7);check(counter(1)()==2);}\n",
    ),
    (
        "mutable",
        "fn main(){let mut c=counter(5);let p:&mut dyn FnMut()->u8=&mut c;check(p()==6);check(p()==7);check(adder(1)(1)==2);}\n",
    ),
    (
        "weaker",
        "fn apply(f:&mut dyn FnMut(u8)->u8)->u8{f(2)}\nfn main(){let mut a=adder(3);check(apply(&mut a)==5);check(counter(1)()==2);}\n",
    ),
    (
        "nested",
        "fn main(){let a=adder(3);let r=&a;let p:&dyn Fn(u8)->u8=&r;check(p(4)==7);check(counter(1)()==2);}\n",
    ),
    (
        "generic-callers",
        "fn call<F:Fn(u8)->u8>(f:F,v:u8)->u8{f(v)}\nfn twice<F:FnMut()->u8>(mut f:F)->u8{f()+f()}\nfn main(){let a=adder(3);let mut c=counter(10);check(call(&a,1)==4);check(call(&&a,2)==5);check(twice(&mut c)==23);check(twice(&mut &mut c)==27);}\n",
    ),
    (
        "composed",
        "fn compose<A,B,C>(f:impl Fn(A)->B,g:impl Fn(B)->C)->impl Fn(A)->C{move|v|g(f(v))}\nfn main(){let both=compose(adder(1),adder(10));let p:&dyn Fn(u8)->u8=&both;check(p(0)==11);check(counter(1)()==2);}\n",
    ),
];

fn source(body: &str) -> String {
    format!("{}{ADDER}{COUNTER}{body}", native::CHECK)
}

#[test]
fn returned_callables_convert_to_objects_as_rustc_accepts() -> Result<(), Box<dyn std::error::Error>>
{
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/closures/returned-callable-objects/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, body) in CASES {
        let path = evidence.join(format!("{name}.rs"));
        std::fs::write(&path, source(body))?;
        checked::check(&path, &evidence, name, (0, 0, ""))?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM backend; run make test-maintainer-native"]
fn returned_callables_execute_through_objects() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body) in CASES {
        let body = format!("{ADDER}{COUNTER}{body}");
        native::check_cases(
            "parallel/closures/returned-callable-objects/native",
            &[(name, &body, false)],
        )?;
    }
    Ok(())
}
