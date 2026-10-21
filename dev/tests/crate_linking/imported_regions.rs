//! Each artifact's graph ordinals are remapped once before ordinary call inference.
use rustgm::{
    support::crates::{
        CrateIdentity, Dependencies, Dependency, ExternalDef, ExternalKind, Function, Interface,
        Lifetime, Scalar, Signature, Symbol, TypeNode, TypeRef,
    },
    types::{Kind, RegionId},
};
use std::sync::Arc;
fn artifact(owner: CrateIdentity, width: usize, second: bool) -> Result<Interface, &'static str> {
    let mut functions = Vec::new();
    for index in 0..width {
        let signature = Signature::with_types(
            vec![
                TypeNode::Scalar(Scalar::U8),
                TypeNode::Reference {
                    region: Lifetime::Bound(0),
                    mutable: false,
                    referent: TypeRef(0),
                },
                TypeNode::Reference {
                    region: Lifetime::Bound(1),
                    mutable: false,
                    referent: TypeRef(0),
                },
            ],
            2,
            vec![TypeRef(1), TypeRef(2)],
            TypeRef(if second { 2 } else { 1 }),
            false,
        )?;
        functions.push(Function::new(
            ExternalDef::new(
                owner.clone(),
                vec![format!("choose{index}")],
                ExternalKind::Function,
            )?,
            signature,
            Symbol::Rust,
        )?);
    }
    Interface::new(owner, vec![], functions)
}
#[test]
fn artifact_local_type_ids_do_not_merge_different_output_slots()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    for width in [8usize, 32, 128] {
        let first = CrateIdentity::new("same", "first")?;
        let second = CrateIdentity::new("same", "second")?;
        let mut text = String::from("fn main(){let first=3u8;let second=9u8;");
        for index in 0..width {
            write!(
                text,
                "let _=one::choose{index}(&first,&second);let _=two::choose{index}(&first,&second);"
            )?;
        }
        text.push('}');
        let mut previous = None;
        for reverse in [false, true] {
            let mut artifacts = vec![
                artifact(first.clone(), width, false)?,
                artifact(second.clone(), width, true)?,
            ];
            let mut aliases = vec![
                Dependency::new("one".into(), first.clone())?,
                Dependency::new("two".into(), second.clone())?,
            ];
            if reverse {
                artifacts.reverse();
                aliases.reverse();
            }
            let dependencies = Arc::new(Dependencies::new(artifacts, aliases)?);
            let report = super::imports::resolve(&text, dependencies)?;
            let rustgm::pipeline::Analysis { facts, .. } =
                rustgm::pipeline::analyze_source(&report)?;
            let imported = report.resolution.crates.as_deref().ok_or("imports")?;
            let mut remaps = Vec::new();
            for (definition, function) in imported.functions() {
                assert!(
                    report.resolution.definitions[definition.0 as usize]
                        .node
                        .is_none()
                );
                assert!(!facts.is_const_function(definition));
                let item = facts.definitions[definition.0 as usize].ok_or("imported type")?;
                let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(item) else {
                    return Err("imported item".into());
                };
                let Some(Kind::Function {
                    bound_regions: 2,
                    result,
                    ..
                }) = facts.types.kind(signature)
                else {
                    return Err("imported signature".into());
                };
                let Some(Kind::Reference { region, .. }) = facts.types.kind(result) else {
                    return Err("imported reference".into());
                };
                assert_eq!(
                    region,
                    RegionId::bound(0, u16::from(function.definition().owner() == &second))?
                );
                remaps.push((function.definition().clone(), definition, item));
            }
            if let Some(prior) = previous.as_ref() {
                assert_eq!(&remaps, prior);
            } else {
                previous = Some(remaps);
            }
            assert_eq!(facts.functions.len(), 1);
            assert!(facts.initializers.is_empty());
            let work = facts.crates.as_deref().ok_or("linkage")?.import_work();
            assert_eq!(work.signatures, 2 * width);
            assert_eq!(work.type_nodes, 6 * width);
            assert_eq!(work.roots, 6 * width);
        }
    }
    Ok(())
}
