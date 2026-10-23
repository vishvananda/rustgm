//! Metadata validation derives ownership/operation checks without proving execution.
use super::*;
use crate::support::crates::{
    ExternalDef, ExternalKind, Scalar,
    owner_catalog::{BodyRef, DeclarationRef},
};
use crate::support::edition::Edition;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(width: u32) -> Result<(CrateIdentity, Rows)> {
    let owner = CrateIdentity::new("transport", "first")?;
    let mut declarations = vec![Declaration::Function {
        identity: ExternalDef::new(owner.clone(), vec!["id".into()], ExternalKind::Function)?,
        public: true,
    }];
    for ordinal in 0..width {
        declarations.push(Declaration::TypeParameter {
            owner: DeclarationRef(0),
            ordinal,
            name: format!("T{ordinal:08}"),
            sized: true,
        });
    }
    let binding = DeclarationRef(width + 1);
    declarations.push(Declaration::ValueParameter {
        body: BodyRef(0),
        ordinal: 0,
        name: "value".into(),
    });
    let mut types = (1..=width)
        .map(|i| Type::Parameter(DeclarationRef(i)))
        .collect::<Vec<_>>();
    let signature = |binder| Signature {
        binder: BinderRef(binder),
        parameters: Range { start: 0, end: 1 },
        result: TypeRef(0),
        unsafe_: false,
        convention: Convention::Rust,
        variadic: false,
    };
    types.push(Type::Function(signature(0)));
    types.push(Type::Function(signature(1)));
    Ok((
        owner,
        Rows {
            edition: Edition::Rust2024,
            declaration_sites: vec![2; declarations.len()],
            declarations,
            binders: vec![
                Binder {
                    owner: DeclarationRef(0),
                    parent: None,
                    kind: BinderKind::OriginalFunction,
                    count: 0,
                },
                Binder {
                    owner: DeclarationRef(0),
                    parent: None,
                    kind: BinderKind::PublicFunction,
                    count: 0,
                },
            ],
            types,
            roots: vec![TypeRef(0)],
            parameters: (1..=width).map(DeclarationRef).collect(),
            clauses: vec![Clause {
                owner: DeclarationRef(0),
                site: 1,
                subject: TypeRef(0),
                quantifier: None,
                bound: Bound::Copy(CopyIdentity::LegacyBuiltin),
            }],
            functions: vec![Function {
                declaration: DeclarationRef(0),
                body: BodyRef(0),
                original: TypeRef(width),
                callable: TypeRef(width + 1),
                original_frame: BinderRef(0),
                callable_frame: BinderRef(1),
                parameters: Range {
                    start: 0,
                    end: width,
                },
                clauses: Range { start: 0, end: 1 },
                declared_clauses: 1,
                constant: true,
            }],
            originals: vec![original(binding)],
            calls: None,
        },
    ))
}
#[test]
fn generic_rows_preserve_unused_slots_and_derive_return_obligations() -> Result {
    let (owner, rows) = fixture(3)?;
    let mut work = Work::default();
    let payload = Payload::new(&owner, rows.clone(), &mut work)?;
    assert_eq!(payload.rows().parameters.len(), 3);
    assert_eq!(payload.owners().owner(), &owner);
    assert_eq!(
        payload.relations(BodyRef(0))?.collect::<Vec<_>>(),
        vec![Relation {
            actual: TypeRef(0),
            expected: TypeRef(0),
            site: 0
        }]
    );
    assert!(work.steps > 0 && work.operations > 0 && work.roots > 0);
    assert!(payload.relations(BodyRef(1)).is_err());
    let interface = crate::support::crates::Interface::new(owner, vec![], vec![])?
        .with_generic_payload(payload)?;
    let bytes = crate::support::crates::encode(&interface)?;
    assert_eq!(bytes[12], 9);
    let decoded = crate::support::crates::decode(&bytes)?;
    assert_eq!(decoded, interface);
    assert_eq!(crate::support::crates::encode(&decoded)?, bytes);
    for end in 0..bytes.len() {
        assert!(crate::support::crates::decode(&bytes[..end]).is_err());
    }
    Ok(())
}
#[test]
fn malformed_generic_owners_inputs_and_operations_never_publish() -> Result {
    let (owner, rows) = fixture(2)?;
    for mutation in 0..14 {
        let mut rows = rows.clone();
        match mutation {
            0 => rows.functions[0].parameters.end -= 1,
            1 => rows.functions[0].callable_frame = BinderRef(0),
            2 => rows.originals[0].locals[0].definition = Some(DeclarationRef(1)),
            3 => rows.originals[0].parameter_bindings.clear(),
            4 => rows.originals[0].parameter_bindings[0].input = 1,
            5 => rows.originals[0].values[0].kind = ValueKind::Constrain(0),
            6 => rows.originals[0].blocks[0].terminator = Terminator::Return(1),
            7 => rows.originals[0].scopes[0].parent = Some(0),
            8 => rows.clauses[0].site = 2,
            9 => rows.functions[0].declared_clauses = 0,
            10 => rows.binders[0].count = 1,
            11 => rows.originals[0].blocks[0].site = 1,
            12 => rows.types[0] = Type::Scalar(Scalar::Unit),
            _ => rows.originals[0].body = BodyRef(1),
        }
        let mut work = Work::default();
        assert!(
            Payload::new(&owner, rows, &mut work).is_err(),
            "mutation {mutation}"
        );
        assert!(work.steps > 0);
    }
    Ok(())
}
#[test]
fn generic_validation_work_tracks_distinct_slots_and_retains_failure_work() -> Result {
    let mut counts = Vec::new();
    for width in [8, 32, 128] {
        let (owner, rows) = fixture(width)?;
        let mut work = Work::default();
        let payload = Payload::new(&owner, rows, &mut work)?;
        assert!(work.steps <= width as usize * 40 + 200);
        assert!(payload.retained_bytes() <= width as usize * 200 + 4000);
        counts.push(work.steps);
    }
    assert!(
        counts
            .windows(2)
            .all(|pair| pair[1] > pair[0] && pair[1] <= pair[0] * 4)
    );
    let (owner, rows) = fixture(1)?;
    let mut work = Work {
        steps: crate::support::crates::MAX_COMPONENTS,
        ..Work::default()
    };
    assert!(Payload::new(&owner, rows, &mut work).is_err());
    assert_eq!(work.steps, crate::support::crates::MAX_COMPONENTS + 1);
    Ok(())
}

fn original(binding: DeclarationRef) -> Original {
    Original {
        arguments: Vec::new(),
        body: BodyRef(0),
        declaration: DeclarationRef(0),
        result: TypeRef(0),
        sites: 1,
        locals: vec![Local {
            scope: 0,
            storage_parent: None,
            ty: TypeRef(0),
            kind: LocalKind::Parameter(0),
            definition: Some(binding),
            mutable: false,
            site: 0,
        }],
        parameter_bindings: vec![ParameterBinding {
            declaration: binding,
            input: 0,
            local: 0,
        }],
        values: vec![Value {
            ty: TypeRef(0),
            kind: ValueKind::Read(0),
        }],
        statements: vec![],
        blocks: vec![Block {
            statements: Range { start: 0, end: 0 },
            site: 0,
            terminator: Terminator::Return(0),
        }],
        scopes: vec![Scope {
            parent: None,
            kind: ScopeKind::Body,
            site: 0,
        }],
        exits: vec![],
        temporaries: vec![],
    }
}

fn recursive_call() -> Result<(CrateIdentity, Rows)> {
    let (owner, mut rows) = fixture(1)?;
    rows.calls = Some(Box::new(Calls {
        header_types: 3,
        applications: vec![Application {
            target: DeclarationRef(0),
            arguments: Range { start: 0, end: 1 },
        }],
        arguments: vec![TypeRef(0)],
    }));
    rows.types.extend([
        Type::FunctionItem(ApplicationRef(0)),
        Type::ApplicationSignature(ApplicationRef(0)),
    ]);
    let body = &mut rows.originals[0];
    body.arguments.push(0);
    body.values.extend([
        Value {
            ty: TypeRef(3),
            kind: ValueKind::Function(ApplicationRef(0)),
        },
        Value {
            ty: TypeRef(0),
            kind: ValueKind::Call {
                signature: TypeRef(4),
                callee: 1,
                arguments: Range { start: 0, end: 1 },
                site: 0,
            },
        },
    ]);
    body.blocks[0].terminator = Terminator::Return(2);
    Ok((owner, rows))
}
#[test]
fn recursive_callable_edges_roundtrip_without_lexical_owner_cycles() -> Result {
    let (owner, rows) = recursive_call()?;
    let payload = Payload::new(&owner, rows, &mut Work::default())?;
    let interface = crate::support::crates::Interface::new(owner, vec![], vec![])?
        .with_generic_payload(payload)?;
    let encoded = crate::support::crates::encode(&interface)?;
    assert_eq!(crate::support::crates::decode(&encoded)?, interface);
    for end in 0..encoded.len() {
        assert!(crate::support::crates::decode(&encoded[..end]).is_err());
    }
    Ok(())
}
#[test]
fn malformed_applications_lists_versions_and_budgets_never_publish() -> Result {
    let (owner, rows) = recursive_call()?;
    for mutation in 0..9 {
        let mut rows = rows.clone();
        match mutation {
            0 => rows.calls = None,
            1 => rows.calls.as_mut().ok_or("calls")?.arguments.clear(),
            2 => rows.calls.as_mut().ok_or("calls")?.arguments[0] = TypeRef(3),
            3 => rows.calls.as_mut().ok_or("calls")?.applications[0].target = DeclarationRef(1),
            4 => rows.types.swap(3, 4),
            5 => rows.originals[0].arguments[0] = 2,
            6 => {
                rows.originals[0].values[2].kind = ValueKind::Call {
                    signature: TypeRef(3),
                    callee: 1,
                    arguments: Range { start: 0, end: 1 },
                    site: 0,
                }
            }
            7 => rows.calls.as_mut().ok_or("calls")?.applications.clear(),
            _ => rows.originals[0].arguments.push(0),
        }
        let mut work = Work::default();
        assert!(
            Payload::new(&owner, rows, &mut work).is_err(),
            "mutation {mutation}"
        );
        assert!(work.steps > 0);
    }
    let mut work = Work {
        steps: crate::support::crates::MAX_COMPONENTS,
        ..Work::default()
    };
    assert!(Payload::new(&owner, rows, &mut work).is_err());
    assert!(work.steps > crate::support::crates::MAX_COMPONENTS);
    Ok(())
}
