//! Source-free named records retain nominal identity, privacy and ordinary move checks.
use rustgm::{
    resolve::model::Access,
    sema::members::Origin,
    support::crates::{
        CrateIdentity, Dependencies, Dependency, ExternalDef, ExternalKind, Function, Interface,
        Lifetime, Nominals, Record, RecordField, RecordRef, Scalar, Signature, Symbol, TypeGraph,
        TypeNode, TypeRef,
    },
    types::ItemId,
};
use std::sync::Arc;

fn definition(
    owner: &CrateIdentity,
    name: &str,
    kind: ExternalKind,
) -> Result<ExternalDef, &'static str> {
    ExternalDef::new(owner.clone(), vec![name.to_owned()], kind)
}
fn record(
    owner: &CrateIdentity,
    name: &str,
    public: bool,
    fields: &[(&str, u16, bool)],
) -> Result<Record, &'static str> {
    Record::new(
        definition(owner, name, ExternalKind::Record)?,
        public,
        fields
            .iter()
            .map(|&(name, ty, public)| RecordField::new(name.to_owned(), TypeRef(ty), public))
            .collect::<Result<_, _>>()?,
    )
}
fn function(
    owner: &CrateIdentity,
    name: &str,
    graph: &TypeGraph,
    parameters: Vec<TypeRef>,
    result: TypeRef,
) -> Result<Function, &'static str> {
    Function::new(
        definition(owner, name, ExternalKind::Function)?,
        Signature::with_shared_types(graph.clone(), 0, parameters, result, false)?,
        Symbol::Rust,
    )
}
fn artifact(owner: CrateIdentity) -> Result<Interface, &'static str> {
    let graph = TypeGraph::new(vec![
        TypeNode::Record(RecordRef(2)),
        TypeNode::Record(RecordRef(1)),
        TypeNode::Scalar(Scalar::U8),
        TypeNode::Scalar(Scalar::U16),
        TypeNode::Record(RecordRef(0)),
        TypeNode::Reference {
            region: Lifetime::Static,
            mutable: false,
            referent: TypeRef(2),
        },
    ])?;
    let functions = vec![
        function(&owner, "make", &graph, vec![], TypeRef(0))?,
        function(&owner, "pair", &graph, vec![], TypeRef(1))?,
        function(&owner, "pass", &graph, vec![TypeRef(1)], TypeRef(1))?,
    ];
    let records = vec![
        record(&owner, "Hidden", false, &[("value", 2, true)])?,
        record(
            &owner,
            "Pair",
            true,
            &[("left", 2, true), ("right", 3, true)],
        )?,
        record(
            &owner,
            "Wrapper",
            true,
            &[("pair", 1, true), ("hidden", 4, false), ("borrow", 5, true)],
        )?,
    ];
    Interface::with_nominals(
        owner,
        vec![],
        functions,
        vec![],
        Nominals::new(graph, records)?,
    )
}
fn dependencies(reverse: bool) -> Result<Arc<Dependencies>, &'static str> {
    let first = CrateIdentity::new("shapes", "one")?;
    let second = CrateIdentity::new("shapes", "two")?;
    let mut artifacts = vec![artifact(first.clone())?, artifact(second.clone())?];
    let mut aliases = vec![
        Dependency::new("one".into(), first)?,
        Dependency::new("two".into(), second)?,
    ];
    if reverse {
        artifacts.reverse();
        aliases.reverse();
    }
    Ok(Arc::new(Dependencies::new(artifacts, aliases)?))
}
fn checked(text: &str) -> Result<(), String> {
    let mut report = super::imports::resolve(text, dependencies(false)?)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[test]
fn records_remap_once_without_source_nodes_or_fake_constructors()
-> Result<(), Box<dyn std::error::Error>> {
    let mut previous = None;
    for reverse in [false, true] {
        let report = super::imports::resolve("fn main(){}", dependencies(reverse)?)?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        let imported = report.resolution.crates.as_deref().ok_or("imports")?;
        let mut remaps = Vec::new();
        for (id, record) in imported.records() {
            assert_eq!(
                analysis.facts.types.nominal_size_rule(ItemId(id.0)),
                Some(rustgm::types::sizedness::Rule::Sized)
            );
            let definition = &report.resolution.definitions[id.0 as usize];
            assert!(definition.node.is_none());
            assert_eq!(
                definition.access,
                if record.public() {
                    Access::Public
                } else {
                    Access::ExternalPrivate
                }
            );
            let variants = analysis.facts.members.variants(ItemId(id.0));
            assert_eq!(variants.len(), 1);
            let variant = variants[0];
            assert_eq!(
                analysis.facts.members.variants[variant.0 as usize].origin,
                Origin::External(id)
            );
            let fields = analysis.facts.members.fields(variant).ok_or("fields")?;
            assert_eq!(fields.len(), record.fields().len());
            for field in fields {
                assert_eq!(field.origin, Origin::External(id));
                assert!(field.origin.diagnostic().is_none());
            }
            assert!(!analysis.facts.members.constructors.contains_key(&id));
            remaps.push((
                record.definition().clone(),
                id,
                analysis.facts.definitions[id.0 as usize],
            ));
        }
        if let Some(old) = &previous {
            assert_eq!(&remaps, old);
        } else {
            previous = Some(remaps);
        }
        let work = analysis
            .facts
            .crates
            .as_deref()
            .ok_or("linkage")?
            .import_work();
        assert_eq!(
            (
                work.records,
                work.fields,
                work.type_nodes,
                work.signatures,
                work.roots
            ),
            (6, 12, 12, 6, 8)
        );
        assert_eq!(analysis.facts.functions.len(), 1);
        assert!(analysis.facts.initializers.is_empty());
    }
    Ok(())
}

#[test]
fn imported_record_consumers_keep_field_privacy_type_identity_and_move_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    for text in [
        "fn main(){let p=one::Pair{left:7,right:9};let q=one::pass(p);let _:u16=q.right;}",
        "fn main(){let p=one::make();let _:u8=p.pair.left;let one::Wrapper{pair,..}=p;let _:u16=pair.right;}",
        "fn get()->&'static u8{one::make().borrow}fn main(){let _=*get();}",
        "fn main(){let p=one::pair();let one::Pair{left,right}=p;let _:u16=right+left as u16;}",
    ] {
        checked(text).map_err(|error| format!("{text}: {error}"))?;
    }
    for (text, reason) in [
        (
            "fn main(){let p=one::pair();let q=one::pass(p);let x=p.left+1;let _=q.right+x as u16;}",
            "uninitialized at this read",
        ),
        (
            "fn main(){let p=one::pair();let _=two::pass(p);}",
            "incompatible types",
        ),
        ("fn main(){let p=one::make();let _=p.hidden;}", "private"),
        (
            "fn main(){let p=one::make();let one::Wrapper{hidden,..}=p;}",
            "private",
        ),
        ("fn main(){let _:one::Hidden;}", "unresolved"),
    ] {
        let Err(error) = checked(text) else {
            return Err(format!("accepted {text}").into());
        };
        assert!(error.contains(reason), "{text}: {error}");
    }
    Ok(())
}

fn growing(records: u16, functions: u16) -> Result<Interface, &'static str> {
    let owner = CrateIdentity::new("many", "records")?;
    let mut nodes: Vec<_> = (0..records)
        .map(|index| TypeNode::Record(RecordRef(index)))
        .collect();
    nodes.push(TypeNode::Scalar(Scalar::U8));
    let graph = TypeGraph::new(nodes)?;
    let functions = (0..functions.max(records))
        .map(|index| {
            let ty = TypeRef(index % records);
            function(&owner, &format!("f{index:03}"), &graph, vec![ty], ty)
        })
        .collect::<Result<_, _>>()?;
    let records = (0..records)
        .map(|index| {
            record(
                &owner,
                &format!("R{index:03}"),
                true,
                &[("value", records, true)],
            )
        })
        .collect::<Result<_, _>>()?;
    Interface::with_nominals(
        owner,
        vec![],
        functions,
        vec![],
        Nominals::new(graph, records)?,
    )
}

#[test]
fn many_functions_share_one_record_graph_remap() -> Result<(), Box<dyn std::error::Error>> {
    for width in [8u16, 32, 128] {
        for (records, functions) in [(1, width), (width, width)] {
            let artifact = growing(records, functions)?;
            let alias = Dependency::new("many".into(), artifact.identity().clone())?;
            let dependencies = Arc::new(Dependencies::new(vec![artifact], vec![alias])?);
            let report = super::imports::resolve("fn main(){}", dependencies)?;
            let analysis = rustgm::pipeline::analyze_source(&report)?;
            let work = analysis
                .facts
                .crates
                .as_deref()
                .ok_or("linkage")?
                .import_work();
            assert_eq!(work.records, usize::from(records));
            assert_eq!(work.fields, usize::from(records));
            assert_eq!(work.signatures, usize::from(functions));
            assert_eq!(work.type_nodes, usize::from(records) + 1);
            assert_eq!(work.roots, 2 * usize::from(functions));
        }
    }
    Ok(())
}
