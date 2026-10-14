use rustgm::types::Kind;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

// Declared outcomes were checked with pinned rustc before implementing headers.
const CASES: &[(&str, bool, &str)] = &[
    (
        "ordinary",
        true,
        "struct S;impl Drop for S{fn drop(&mut self){}}",
    ),
    (
        "explicit-self",
        true,
        "struct S;impl Drop for S{fn drop(self:&mut Self){}}",
    ),
    (
        "explicit-nominal",
        true,
        "struct S;impl Drop for S{fn drop(self:&mut S){}}",
    ),
    (
        "mutable-binding",
        true,
        "struct S;impl Drop for S{fn drop(mut self:&mut Self){}}",
    ),
    (
        "named-receiver",
        true,
        "struct S;impl Drop for S{fn drop<'a>(&'a mut self){}}",
    ),
    (
        "unused-method-lifetime",
        true,
        "struct S;impl Drop for S{fn drop<'a>(&mut self){}}",
    ),
    (
        "receiver-placeholder",
        true,
        "struct S;impl Drop for S{fn drop(&'_ mut self){}}",
    ),
    (
        "impl-lifetime",
        true,
        "struct S<'a>(&'a mut u8);impl<'a> Drop for S<'a>{fn drop(&mut self){*self.0+=1;}}",
    ),
    (
        "renamed-lifetime",
        true,
        "struct S<'a>(&'a mut u8);impl<'b> Drop for S<'b>{fn drop(&mut self){*self.0+=1;}}",
    ),
    (
        "permuted-lifetime",
        true,
        "struct S<'a,'b>(&'a u8,&'b u8);impl<'x,'y> Drop for S<'y,'x>{fn drop(&mut self){}}",
    ),
    (
        "required-bound",
        true,
        "struct S<'a:'b,'b>(&'a u8,&'b u8);impl<'x:'y,'y> Drop for S<'x,'y>{fn drop(&mut self){}}",
    ),
    (
        "implied-bound",
        true,
        "struct S<'a:'b,'b>(&'a u8,&'b u8);impl<'x,'y> Drop for S<'x,'y>{fn drop(&mut self){}}",
    ),
    (
        "implied-field-bound",
        true,
        "struct S<'a,'b>(&'a &'b u8);impl<'a,'b:'a> Drop for S<'a,'b>{fn drop(&mut self){}}",
    ),
    (
        "alias-target",
        true,
        "struct S;type A=S;impl Drop for A{fn drop(&mut self){}}",
    ),
    (
        "enum-target",
        true,
        "enum S{A,B}impl Drop for S{fn drop(&mut self){}}",
    ),
    ("missing-method", false, "struct S;impl Drop for S{}"),
    (
        "extra-method",
        false,
        "struct S;impl Drop for S{fn drop(&mut self){}fn extra(){}}",
    ),
    (
        "extra-constant",
        false,
        "struct S;impl Drop for S{fn drop(&mut self){}const X:u8=0;}",
    ),
    (
        "shared-self",
        false,
        "struct S;impl Drop for S{fn drop(&self){}}",
    ),
    (
        "owned-self",
        false,
        "struct S;impl Drop for S{fn drop(self){}}",
    ),
    (
        "not-a-method",
        false,
        "struct S;impl Drop for S{fn drop(_: &mut Self){}}",
    ),
    (
        "extra-parameter",
        false,
        "struct S;impl Drop for S{fn drop(&mut self,_:u8){}}",
    ),
    (
        "return-type",
        false,
        "struct S;impl Drop for S{fn drop(&mut self)->u8{0}}",
    ),
    (
        "unsafe-method",
        false,
        "struct S;impl Drop for S{unsafe fn drop(&mut self){}}",
    ),
    (
        "const-method",
        false,
        "struct S;impl Drop for S{const fn drop(&mut self){}}",
    ),
    (
        "async-method",
        false,
        "struct S;impl Drop for S{async fn drop(&mut self){}}",
    ),
    (
        "foreign-method",
        false,
        "struct S;impl Drop for S{extern \"C\" fn drop(&mut self){}}",
    ),
    (
        "generic-method",
        false,
        "struct S;impl Drop for S{fn drop<T>(&mut self){}}",
    ),
    (
        "unsafe-impl",
        false,
        "struct S;unsafe impl Drop for S{fn drop(&mut self){}}",
    ),
    ("negative-impl", false, "struct S;impl !Drop for S{}"),
    (
        "duplicate-impl",
        false,
        "struct S;impl Drop for S{fn drop(&mut self){}}impl Drop for S{fn drop(&mut self){}}",
    ),
    (
        "copy-drop",
        false,
        "#[derive(Clone,Copy)]struct S;impl Drop for S{fn drop(&mut self){}}",
    ),
    (
        "primitive-target",
        false,
        "impl Drop for u8{fn drop(&mut self){}}",
    ),
    (
        "tuple-target",
        false,
        "impl Drop for (u8,u8){fn drop(&mut self){}}",
    ),
    (
        "receiver-tied-to-impl",
        false,
        "struct S<'a>(&'a u8);impl<'a> Drop for S<'a>{fn drop(&'a mut self){}}",
    ),
    (
        "receiver-static",
        false,
        "struct S;impl Drop for S{fn drop(&'static mut self){}}",
    ),
    (
        "receiver-bounded",
        false,
        "struct S;impl Drop for S{fn drop<'a:'static>(&'a mut self){}}",
    ),
    (
        "specialized-static",
        false,
        "struct S<'a>(&'a u8);impl Drop for S<'static>{fn drop(&mut self){}}",
    ),
    (
        "repeated-lifetime",
        false,
        "struct S<'a,'b>(&'a u8,&'b u8);impl<'a> Drop for S<'a,'a>{fn drop(&mut self){}}",
    ),
    (
        "unused-impl-lifetime",
        true,
        "struct S;impl<'a> Drop for S{fn drop(&mut self){}}",
    ),
    (
        "strengthened-bound",
        false,
        "struct S<'a,'b>(&'a u8,&'b u8);impl<'a:'b,'b> Drop for S<'a,'b>{fn drop(&mut self){}}",
    ),
    (
        "unused-bound",
        true,
        "struct S;impl<'a:'b,'b> Drop for S{fn drop(&mut self){}}",
    ),
    (
        "unused-static-bound",
        true,
        "struct S;impl<'a:'static> Drop for S{fn drop(&mut self){}}",
    ),
    (
        "extra-unused-bound-to-used",
        true,
        "struct S<'a>(&'a u8);impl<'a:'b,'b> Drop for S<'a>{fn drop(&mut self){}}",
    ),
    (
        "extra-unused-bound-from-used",
        true,
        "struct S<'a>(&'a u8);impl<'a,'b:'a> Drop for S<'a>{fn drop(&mut self){}}",
    ),
    (
        "erased-alias",
        true,
        "struct S;type A<'a>=S;impl<'a> Drop for A<'a>{fn drop(&mut self){}}",
    ),
    (
        "unused-transitive",
        false,
        "struct S<'a,'b>(&'a u8,&'b u8);impl<'a:'x,'x:'b,'b> Drop for S<'a,'b>{fn drop(&mut self){}}",
    ),
    (
        "unused-static-chain",
        false,
        "struct S<'a>(&'a u8);impl<'a:'x,'x:'static> Drop for S<'a>{fn drop(&mut self){}}",
    ),
    (
        "unused-common-shorter",
        true,
        "struct S<'a,'b>(&'a u8,&'b u8);impl<'a:'x,'b:'x,'x> Drop for S<'a,'b>{fn drop(&mut self){}}",
    ),
    (
        "anonymous-target",
        true,
        "struct S<'a>(&'a u8);impl Drop for S<'_>{fn drop(&mut self){}}",
    ),
    (
        "elided-target",
        false,
        "struct S<'a>(&'a u8);impl Drop for S{fn drop(&mut self){}}",
    ),
    (
        "mixed-target",
        true,
        "struct S<'a,'b>(&'a u8,&'b u8);impl<'a> Drop for S<'a,'_>{fn drop(&mut self){}}",
    ),
    (
        "two-anonymous",
        true,
        "struct S<'a,'b>(&'a u8,&'b u8);impl Drop for S<'_,'_>{fn drop(&mut self){}}",
    ),
    (
        "method-tautological-bound",
        false,
        "struct S;impl Drop for S{fn drop<'a:'a>(&'a mut self){}}",
    ),
    (
        "method-unused-bound",
        false,
        "struct S;impl Drop for S{fn drop<'a:'static>(&mut self){}}",
    ),
    (
        "method-impl-bound",
        true,
        "struct S<'a:'b,'b>(&'a u8,&'b u8);impl<'a,'b> Drop for S<'a,'b>{fn drop(&mut self) where 'a:'b{}}",
    ),
    (
        "method-stronger-impl-bound",
        false,
        "struct S<'a,'b>(&'a u8,&'b u8);impl<'a,'b> Drop for S<'a,'b>{fn drop(&mut self) where 'a:'b{}}",
    ),
    (
        "explicit-placeholder",
        true,
        "struct S;impl Drop for S{fn drop(self:&'_ mut Self){}}",
    ),
    (
        "self-nested-callable",
        true,
        "struct S<'a>(&'a u8);fn helper<'a>(_:&S<'a>){}impl<'b> Drop for S<'b>{fn drop(&mut self){let f:fn(&Self)=helper;f(self);}}",
    ),
    (
        "named-impl-explicit-receiver",
        true,
        "struct S<'a>(&'a mut u8);impl<'b> Drop for S<'b>{fn drop<'r>(self:&'r mut Self){*self.0+=1;}}",
    ),
    (
        "placeholder-helper-call",
        true,
        "fn bump(x:&mut u8){*x+=1;}struct S<'a>(&'a mut u8);impl Drop for S<'_>{fn drop(&mut self){bump(self.0);}}",
    ),
    (
        "mutable-receiver-assignment",
        true,
        "struct S;impl Drop for S{fn drop(mut self:&mut Self){self=self;}}",
    ),
    (
        "different-receiver-type",
        false,
        "struct S;struct T;impl Drop for S{fn drop(self:&mut T){}}",
    ),
    (
        "reference-result",
        false,
        "struct S;impl Drop for S{fn drop(&mut self)->&Self{self}}",
    ),
    (
        "private-method-body-error",
        false,
        "struct S{n:u8}impl Drop for S{fn drop(&mut self){self.n=true;}}",
    ),
    (
        "cfg-method",
        true,
        "struct S;impl Drop for S{#[cfg(any())]fn extra(){}#[cfg(all())]fn drop(&mut self){}}",
    ),
    (
        "aliased-duplicate",
        false,
        "struct S;type A=S;impl Drop for S{fn drop(&mut self){}}impl Drop for A{fn drop(&mut self){}}",
    ),
    (
        "enum-lifetime-receiver",
        true,
        "enum S<'a>{A(&'a mut u8),B}impl Drop for S<'_>{fn drop(&mut self){match self{S::A(n)=>**n+=1,S::B=>()}}}",
    ),
    (
        "explicit-destructor-call",
        false,
        "struct S;impl Drop for S{fn drop(&mut self){Self::drop(self);}}",
    ),
    (
        "explicit-destructor-value",
        false,
        "struct S;impl Drop for S{fn drop(&mut self){let f=Self::drop;}}",
    ),
    (
        "receiver-alias",
        true,
        "struct S;type R<'a>=&'a mut S;impl Drop for S{fn drop(self:R<'_>){}}",
    ),
    (
        "extern-rust",
        true,
        "struct S;impl Drop for S{extern \"Rust\" fn drop(&mut self){}}",
    ),
    (
        "explicit-rust-convention",
        true,
        r#"extern "\x52ust" fn helper(_: &S){}struct S;impl Drop for S{extern r"Rust" fn drop(&mut self){let f:extern "Rust" fn(&S)=helper;f(self);}}"#,
    ),
];

#[test]
fn destructor_headers_match_independent_language_contracts()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, source) in CASES {
        rustc::qualify(name, source, accepted)?;
        let result = semantic::analyze(source);
        if !accepted {
            assert!(result.is_err(), "{name}: invalid destructor accepted");
            continue;
        }
        let (report, facts) = result.map_err(|e| format!("{name}: {e}"))?;
        assert_eq!(facts.destructors.implementations().len(), 1, "{name}");
        assert_eq!(facts.receivers.len(), 1, "{name}");
        let implementation = &facts.destructors.implementations()[0];
        let receiver = facts.receivers.values().next().ok_or("missing receiver")?;
        assert!(
            matches!(facts.types.kind(receiver.ty), Some(Kind::Reference { mutable: true, referent, .. }) if referent == implementation.target),
            "{name}"
        );
        assert_eq!(
            facts.definitions[receiver.definition.0 as usize],
            Some(receiver.ty),
            "{name}"
        );
        assert_eq!(
            receiver.mutable,
            matches!(name, "mutable-binding" | "mutable-receiver-assignment"),
            "{name}"
        );
        let program = rustgm::body::build(&report, &facts).map_err(|e| format!("{name}: {e}"))?;
        let pending = program.drops.as_ref().ok_or("missing pending destructor")?;
        assert_eq!(pending.methods.len(), 1, "{name}");
        let local = program
            .bodies
            .iter()
            .flat_map(|body| &body.locals)
            .find(|local| local.definition == Some(receiver.definition))
            .ok_or("missing receiver local")?;
        assert_eq!(local.ty, receiver.ty, "{name}");
        assert_eq!(local.mutable, receiver.mutable, "{name}");
        assert_eq!(local.scope.0, 0, "{name}");
    }
    Ok(())
}

#[test]
fn typed_drop_inspection_keeps_borrow_boundaries_and_lowir_proves_cleanup()
-> Result<(), Box<dyn std::error::Error>> {
    use std::{path::Path, process::Command};
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/drop-boundary-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    let input = folder.join("main.rs");
    std::fs::write(
        &input,
        "struct S{n:u8}impl Drop for S{fn drop(&mut self){self.n+=1;}}fn main(){let _s=S{n:1};}",
    )?;
    let run = |mode| {
        Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg(mode)
            .arg(&input)
            .output()
    };
    let typed = run("--emit-types")?;
    assert_eq!(typed.status.code(), Some(0));
    assert!(typed.stderr.is_empty());
    let text = String::from_utf8(typed.stdout)?;
    assert!(
        text.contains("destructor item=")
            && text.contains("cleanup=pending")
            && text.contains("receiver parameter=")
    );
    let counts = run("--count-types")?;
    assert_eq!(counts.status.code(), Some(0));
    assert!(String::from_utf8(counts.stdout)?.contains("destructors=1 receivers=1"));
    for mode in ["--emit-body-ir", "--count-body-ir"] {
        let output = run(mode)?;
        assert_eq!(output.status.code(), Some(0), "{mode}");
        assert!(
            String::from_utf8(output.stdout)?.contains("pending-destructor"),
            "{mode}"
        );
    }
    for mode in [
        "--emit-ownership",
        "--emit-regions",
        "--count-regions",
        "--emit-loans",
        "--count-loans",
    ] {
        let result = run(mode)?;
        assert_eq!(result.status.code(), Some(1), "{mode}");
        assert!(result.stdout.is_empty(), "{mode}");
        assert!(
            String::from_utf8(result.stderr)?.contains("require destructor elaboration"),
            "{mode}"
        );
    }
    for mode in ["--emit-lowir", "--count-lowir"] {
        let result = run(mode)?;
        assert_eq!(result.status.code(), Some(0), "{mode}");
        assert!(result.stderr.is_empty(), "{mode}");
        assert!(!result.stdout.is_empty(), "{mode}");
    }
    Ok(())
}

#[test]
fn wide_impl_binders_preserve_named_method_slots_and_diagnose_region_capacity()
-> Result<(), Box<dyn std::error::Error>> {
    let source = |count| {
        let names = (0..count)
            .map(|i| format!("'a{i}"))
            .collect::<Vec<_>>()
            .join(",");
        format!("struct S;impl<{names}> Drop for S{{fn drop<'r>(&'r mut self){{}}}}")
    };
    let input = source(4096);
    rustc::qualify("wide-method-binder", &input, true)?;
    let (_, facts) = semantic::analyze(&input)?;
    let implementation = &facts.destructors.implementations()[0];
    assert_eq!(implementation.parameters, 4096);
    let scheme = facts
        .lifetimes
        .functions
        .get(&implementation.method)
        .ok_or("missing method environment")?;
    assert_eq!(scheme.total, 4097);
    assert_eq!(scheme.late, [4096]);
    let error = semantic::analyze(&source(65_536))
        .err()
        .ok_or("region capacity was not diagnosed")?;
    assert!(
        error.contains("signature has too many lifetime parameters"),
        "{error}"
    );
    Ok(())
}
