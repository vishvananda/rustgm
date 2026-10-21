//! Corrupted typed capture paths cannot acquire raw/reference or foreign field permissions.
#[path = "../support/semantic.rs"]
mod semantic;
use crate::sema::{
    closures::captures::{Graph, Mode, Step},
    members::{FieldId, Name, Selection},
};
#[test]
fn replay_rejects_wrong_capture_capabilities_before_projection()
-> Result<(), Box<dyn std::error::Error>> {
    for case in [
        "raw-reference",
        "raw-mutability",
        "union-category",
        "foreign-owner",
    ] {
        let source = if case.starts_with("raw") {
            "fn main(){let x=7u32;let p=&raw const x;let c=||unsafe{*p};let _=c();}"
        } else {
            "struct Other{foreign:u32}union U{a:u32}fn main(){let u=U{a:7};let c=||unsafe{u.a};let _=c();}"
        };
        let (report, mut facts) = semantic::analyze(source)?;
        let foreign=facts.members.fields.iter().position(|field|matches!(field.name,Name::Named(name) if facts.members.names.get(name)=="foreign"));
        let environment = facts
            .closures
            .environments
            .values_mut()
            .find(|e| !e.plan.captures().is_empty())
            .ok_or("missing capture")?;
        assert_eq!(environment.plan.path_count(), 2);
        let capture = environment.plan.captures()[0];
        let definition = environment.plan.root(capture.path).ok_or("root")?;
        let original = environment
            .accesses
            .values()
            .find_map(|path| environment.plan.parent(*path).map(|(_, step)| step))
            .ok_or("leaf")?;
        let replacement = match case {
            "raw-reference" => Step::Dereference { mutable: false },
            "raw-mutability" => Step::RawDereference { mutable: true },
            "union-category" => {
                let Step::UnionField(field) = original else {
                    return Err("union leaf".into());
                };
                Step::Field(field)
            }
            "foreign-owner" => Step::UnionField(Selection::Nominal(FieldId(u32::try_from(
                foreign.ok_or("foreign field")?,
            )?))),
            _ => return Err("case".into()),
        };
        let mut graph = Graph::default();
        let root = graph.root(definition)?;
        graph.project(root, replacement)?;
        graph.require(root, Mode::Shared)?;
        environment.plan = graph.finish()?;
        let error = crate::body::build_concrete(&report, &facts)
            .err()
            .ok_or("corrupted capture was accepted")?;
        let expected = if case.starts_with("raw") {
            "capture projection dereference capability changed"
        } else if case == "foreign-owner" {
            "capture projection field owner changed"
        } else {
            "capture projection field capability changed"
        };
        assert!(error.to_string().contains(expected), "{case}: {error}");
    }
    Ok(())
}
