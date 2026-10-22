//! Shared nominal artifacts reject malformed contracts before semantic publication.
use rustgm::support::crates::{
    self, CrateIdentity, ExternalDef, ExternalKind, Function, Interface, Lifetime, Nominals,
    Record, RecordField, RecordRef, Scalar, Signature, Symbol, TypeGraph, TypeNode, TypeRef,
};

fn owner() -> Result<CrateIdentity, &'static str> {
    CrateIdentity::new("records", "one")
}
fn record(name: &str, public: bool, fields: &[(&str, u16, bool)]) -> Result<Record, &'static str> {
    Record::new(
        ExternalDef::new(owner()?, vec![name.to_owned()], ExternalKind::Record)?,
        public,
        fields
            .iter()
            .map(|&(name, ty, public)| RecordField::new(name.to_owned(), TypeRef(ty), public))
            .collect::<Result<_, _>>()?,
    )
}
fn reference(region: Lifetime, referent: u16) -> TypeNode {
    TypeNode::Reference {
        region,
        mutable: false,
        referent: TypeRef(referent),
    }
}
fn function(
    name: &str,
    graph: &TypeGraph,
    slots: u16,
    input: u16,
    output: u16,
) -> Result<Function, &'static str> {
    Function::new(
        ExternalDef::new(owner()?, vec![name.to_owned()], ExternalKind::Function)?,
        Signature::with_shared_types(
            graph.clone(),
            slots,
            vec![TypeRef(input)],
            TypeRef(output),
            false,
        )?,
        Symbol::Rust,
    )
}
fn artifact() -> Result<Interface, &'static str> {
    let types = TypeGraph::new(vec![
        TypeNode::Record(RecordRef(0)),
        reference(Lifetime::Static, 0),
        TypeNode::Record(RecordRef(1)),
        TypeNode::Scalar(Scalar::U8),
    ])?;
    let functions = vec![
        function("f", &types, 0, 0, 0)?,
        function("g", &types, 3, 1, 1)?,
    ];
    let records = vec![
        record("A", true, &[("hidden", 2, false), ("parent", 1, true)])?,
        record("Z", false, &[("byte", 3, true)])?,
    ];
    Interface::with_nominals(
        owner()?,
        vec![],
        functions,
        vec![],
        Nominals::new(types, records)?,
    )
}

#[test]
fn records_roundtrip_with_one_shared_graph_and_private_layouts()
-> Result<(), Box<dyn std::error::Error>> {
    let interface = artifact()?;
    let bytes = crates::encode(&interface)?;
    assert_eq!(bytes[12], 3);
    let decoded = crates::decode(&bytes)?;
    assert_eq!(decoded, interface);
    assert_eq!(crates::encode(&decoded)?, bytes);
    let nominals = decoded.nominals().ok_or("missing nominal profile")?;
    assert!(!nominals.records()[1].public());
    assert!(!nominals.records()[0].fields()[0].public());
    assert_eq!(decoded.functions()[1].signature().bound_regions(), 3);
    for function in decoded.functions() {
        assert!(
            function
                .signature()
                .types()
                .shares_storage(nominals.types())
        );
    }
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(crates::decode(&trailing).is_err());
    Ok(())
}

#[test]
fn record_storage_cycles_are_distinct_from_reference_cycles()
-> Result<(), Box<dyn std::error::Error>> {
    let nodes = vec![
        TypeNode::Record(RecordRef(0)),
        TypeNode::Record(RecordRef(1)),
    ];
    assert!(
        Nominals::new(
            TypeGraph::new(nodes.clone())?,
            vec![
                record("A", true, &[("b", 1, true)])?,
                record("B", true, &[("a", 0, true)])?
            ]
        )
        .is_err()
    );
    assert!(
        Nominals::new(
            TypeGraph::new(vec![TypeNode::Record(RecordRef(2))])?,
            vec![record("A", true, &[("bad", 0, true)])?]
        )
        .is_err()
    );
    let types = TypeGraph::new(vec![nodes[0], reference(Lifetime::Static, 0)])?;
    let records = Nominals::new(types, vec![record("A", true, &[("next", 1, true)])?])?;
    let interface = Interface::with_nominals(owner()?, vec![], vec![], vec![], records)?;
    assert_eq!(crates::decode(&crates::encode(&interface)?)?, interface);
    Ok(())
}

#[test]
fn function_binders_validate_only_their_roots_while_fields_remain_static()
-> Result<(), Box<dyn std::error::Error>> {
    let types = TypeGraph::new(vec![
        TypeNode::Record(RecordRef(0)),
        reference(Lifetime::Bound(0), 0),
        reference(Lifetime::Bound(2), 0),
    ])?;
    let functions = vec![
        function("f", &types, 1, 1, 1)?,
        function("g", &types, 4, 2, 2)?,
    ];
    assert!(function("bad", &types, 2, 2, 2).is_err());
    assert!(function("escape", &types, 4, 1, 2).is_err());
    assert!(
        Nominals::new(
            types.clone(),
            vec![record("A", true, &[("borrow", 1, true)])?]
        )
        .is_err()
    );
    let interface = Interface::with_nominals(
        owner()?,
        vec![],
        functions,
        vec![],
        Nominals::new(types, vec![record("A", true, &[])?])?,
    )?;
    assert_eq!(crates::decode(&crates::encode(&interface)?)?, interface);
    Ok(())
}

#[test]
fn record_identity_reachability_and_shared_storage_cannot_be_forged()
-> Result<(), Box<dyn std::error::Error>> {
    assert!(record("A", true, &[("x", 0, true), ("x", 0, false)]).is_err());
    let scalar = TypeGraph::new(vec![TypeNode::Scalar(Scalar::U8)])?;
    assert!(Nominals::new(scalar.clone(), vec![]).is_err());
    assert!(
        Nominals::new(
            scalar.clone(),
            vec![record("Z", true, &[])?, record("A", true, &[])?]
        )
        .is_err()
    );
    let hidden = Nominals::new(scalar.clone(), vec![record("A", false, &[("x", 0, true)])?])?;
    assert!(Interface::with_nominals(owner()?, vec![], vec![], vec![], hidden).is_err());
    let foreign = Nominals::new(scalar.clone(), vec![record("A", true, &[("x", 0, true)])?])?;
    assert!(
        Interface::with_nominals(
            CrateIdentity::new("other", "one")?,
            vec![],
            vec![],
            vec![],
            foreign
        )
        .is_err()
    );
    let copied = TypeGraph::new(scalar.nodes().to_vec())?;
    assert!(!scalar.shares_storage(&copied));
    let functions = vec![function("f", &copied, 0, 0, 0)?];
    let nominals = Nominals::new(scalar, vec![record("A", true, &[("x", 0, true)])?])?;
    assert!(Interface::with_nominals(owner()?, vec![], functions, vec![], nominals).is_err());
    let empty = Interface::with_nominals(
        owner()?,
        vec![],
        vec![],
        vec![],
        Nominals::new(TypeGraph::new(vec![])?, vec![record("A", true, &[])?])?,
    )?;
    assert_eq!(crates::decode(&crates::encode(&empty)?)?, empty);
    Ok(())
}

#[test]
fn shared_graph_roots_do_not_let_legacy_profiles_emit_unreachable_types()
-> Result<(), Box<dyn std::error::Error>> {
    let graph = TypeGraph::new(vec![
        TypeNode::Scalar(Scalar::U8),
        TypeNode::Scalar(Scalar::U16),
    ])?;
    assert!(Interface::new(owner()?, vec![], vec![function("f", &graph, 0, 0, 0)?]).is_err());
    let graph = TypeGraph::new(vec![TypeNode::Record(RecordRef(0))])?;
    assert!(Interface::new(owner()?, vec![], vec![function("f", &graph, 0, 0, 0)?]).is_err());
    Ok(())
}

#[test]
fn private_record_layouts_do_not_grant_public_type_contracts()
-> Result<(), Box<dyn std::error::Error>> {
    for root in [0, 1] {
        let mut nodes = vec![TypeNode::Record(RecordRef(0))];
        if root == 1 {
            nodes.push(reference(Lifetime::Static, 0));
        }
        let byte = u16::try_from(nodes.len())?;
        nodes.push(TypeNode::Scalar(Scalar::U8));
        let graph = TypeGraph::new(nodes)?;
        let records = Nominals::new(
            graph.clone(),
            vec![record("Hidden", false, &[("byte", byte, true)])?],
        )?;
        let result = Interface::with_nominals(
            owner()?,
            vec![],
            vec![function("f", &graph, 0, root, root)?],
            vec![],
            records,
        );
        assert!(result.is_err_and(|error| error.contains("unexported nominal")));
    }
    for visible in [false, true] {
        let graph = TypeGraph::new(vec![
            TypeNode::Scalar(Scalar::U8),
            TypeNode::Record(RecordRef(0)),
        ])?;
        let records = Nominals::new(
            graph,
            vec![
                record("Hidden", false, &[("byte", 0, true)])?,
                record("Public", true, &[("hidden", 1, visible)])?,
            ],
        )?;
        let result = Interface::with_nominals(owner()?, vec![], vec![], vec![], records);
        assert_eq!(result.is_ok(), !visible);
    }
    Ok(())
}

#[test]
fn record_names_cannot_replace_module_prefixes_but_keep_separate_value_namespaces()
-> Result<(), Box<dyn std::error::Error>> {
    let graph = TypeGraph::new(vec![TypeNode::Scalar(Scalar::U8)])?;
    let nominal = || Nominals::new(graph.clone(), vec![record("A", true, &[("x", 0, true)])?]);
    let signature =
        Signature::with_shared_types(graph.clone(), 0, vec![TypeRef(0)], TypeRef(0), false)?;
    let collision = Function::new(
        ExternalDef::new(
            owner()?,
            vec!["A".into(), "f".into()],
            ExternalKind::Function,
        )?,
        signature,
        Symbol::Rust,
    )?;
    assert!(
        Interface::with_nominals(owner()?, vec![], vec![collision], vec![], nominal()?).is_err()
    );
    // A named record has a type binding, unlike a tuple/unit value constructor.
    let legal = Interface::with_nominals(
        owner()?,
        vec![],
        vec![function("A", &graph, 0, 0, 0)?],
        vec![],
        nominal()?,
    )?;
    assert_eq!(crates::decode(&crates::encode(&legal)?)?, legal);
    Ok(())
}
