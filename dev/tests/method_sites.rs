//! Method search keeps its lexical site without resolving the spelling as a value.
#[path = "helpers/resolve.rs"]
mod names;
#[path = "helpers/qualify.rs"]
mod rustc;
use rustgm::resolve::model::{DefKind, Target};

#[test]
fn method_calls_retain_import_scope_and_separate_generic_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "mod m{pub struct S;pub trait A{fn f<T>(&self,x:T)->T{x}}impl A for S{}}fn main(){let s=m::S;{use m::A as _;let _=s.r#f(1u8);}{use m::A as B;let _=s.f::<u16>(2);}}";
    rustc::qualify("method-sites-scopes", source, true)?;
    for expanded in [false, true] {
        let report = names::resolve(source, expanded)?;
        let facts = &report.resolution;
        assert_eq!(facts.method_calls.len(), 2);
        let mut scopes = Vec::new();
        let mut generic = 0;
        for (key, site) in &facts.method_calls {
            assert_eq!(report.package.names.get(site.name), "f");
            assert!(facts.imports.iter().any(|i| i.scope == site.scope));
            scopes.push(site.scope);
            generic += usize::from(site.method.is_some());
            assert_eq!(site.arguments.end - site.arguments.start, 1);
            let receiver = facts
                .references
                .iter()
                .find(|r| r.node.file == key.file && r.node.node == site.receiver)
                .ok_or("missing resolved receiver")?;
            let Target::Definition(id) = receiver.target else {
                return Err("receiver did not retain its local binding".into());
            };
            assert_eq!(facts.definitions[id.0 as usize].kind, DefKind::Local);
            assert_eq!(
                report
                    .package
                    .names
                    .get(facts.definitions[id.0 as usize].name),
                "s"
            );
            assert!(!facts.references.iter().any(|r| r.node == *key));
        }
        assert_eq!(generic, 1);
        assert_ne!(scopes[0], scopes[1]);
    }
    Ok(())
}

#[test]
fn expanded_method_names_share_canonical_identity_and_call_site_scope()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct S;impl S{fn café<T>(&self,x:T)->T{x}}macro_rules! call{($s:expr)=>{$s.cafe\u{301}::<u8>(7)}}fn main(){let s=S;let _=call!(s);let _=s.café(8u8);}";
    rustc::qualify("method-sites-expanded", source, true)?;
    let report = names::resolve(source, true)?;
    let facts = &report.resolution;
    assert_eq!(facts.method_calls.len(), 2);
    let declaration = facts
        .definitions
        .iter()
        .find(|d| report.package.names.get(d.name) == "café")
        .ok_or("missing canonical method declaration")?;
    let main = facts
        .definitions
        .iter()
        .find(|d| report.package.names.get(d.name) == "main")
        .ok_or("missing main")?;
    let module = facts.scopes[main.scope.0 as usize].module;
    for site in facts.method_calls.values() {
        assert_eq!(site.name, declaration.name);
        assert_eq!(facts.scopes[site.scope.0 as usize].module, module);
        assert_ne!(site.scope, declaration.scope);
    }
    Ok(())
}

#[test]
fn method_candidates_stay_deferred_but_operand_names_must_resolve()
-> Result<(), Box<dyn std::error::Error>> {
    let deferred = "struct S;fn main(){S.missing();}";
    rustc::qualify("method-sites-deferred", deferred, false)?;
    for expanded in [false, true] {
        let report = names::resolve(deferred, expanded)?;
        assert_eq!(report.resolution.method_calls.len(), 1);
        for source in [
            "struct S;fn main(){missing.f();}",
            "struct S;fn main(){S.f(missing);}",
            "struct S;fn main(){S.f::<Missing>();}",
        ] {
            assert!(names::resolve(source, expanded).is_err());
        }
    }
    Ok(())
}
