//! A location never authenticates a changed canonical predicate or subject.
use super::*;
use crate::{
    resolve::{
        body_origin::{CatalogSlot, DeclarationBinding, Edition, Input as CatalogInput},
        model::{Access, Definition, Namespace, ScopeId},
        names::NameId,
    },
    support::crates::{
        CrateIdentity,
        owner_catalog::{BodyCatalog, DeclarationBody},
    },
    types::{
        TypeId,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};

fn catalog() -> Result<std::rc::Rc<Catalog>, &'static str> {
    let owner = CrateIdentity::new("canonical_inputs", "one")?;
    let definitions = [Definition {
        name: NameId(0),
        namespace: Namespace::Type,
        kind: DefKind::Trait,
        scope: ScopeId(0),
        node: None,
        access: Access::Public,
        members: None,
    }];
    let mut catalog = CatalogSlot::default();
    catalog.ensure()?.install(
        &definitions,
        CatalogInput {
            owners: BodyCatalog::new(owner, &[DeclarationBody::Other], Vec::new())?,
            edition: Edition::Rust2024,
            definitions: vec![DefId(0)],
            declarations: vec![DeclarationBinding {
                kind: DefKind::Trait,
                sites: 0..1,
            }],
            bindings: Vec::new(),
            files: Vec::new(),
            labels: Vec::new(),
            declaration_labels: vec![None],
        },
    )?;
    catalog.freeze()?;
    catalog.shared()?.ok_or("missing frozen test catalog")
}

#[test]
fn imported_input_classification_rejects_same_site_changed_predicate_subject_and_catalog()
-> Result<(), &'static str> {
    let catalog = catalog()?;
    let declaration = catalog.declaration(DefId(0))?;
    let site = catalog.declaration_site(declaration, 0)?;
    let original = Input {
        source: catalog.declaration_proof_site(declaration, site)?,
        value: Clause::Predicate(Predicate {
            ty: TypeId::UNIT,
            bound: Bound::Copy,
        }),
    };
    // Exercise the storage boundary, not a constructor of permission or proof.
    let inputs = HeaderInputs::Imported(Imported {
        declaration,
        inputs: Arc::from([original]),
        self_subjects: Box::new([true]),
    });
    inputs.require_catalog(&catalog, DefId(0))?;
    assert!(inputs.is_self(0, original)?);
    for changed in [
        Predicate {
            ty: TypeId::UNIT,
            bound: Bound::Clone,
        },
        Predicate {
            ty: TypeId::primitive(crate::types::Primitive::U8),
            bound: Bound::Copy,
        },
    ] {
        assert!(
            inputs
                .is_self(
                    0,
                    Input {
                        value: changed.into(),
                        ..original
                    }
                )
                .is_err()
        );
    }
    assert!(inputs.is_self(1, original).is_err());
    let foreign = self::catalog()?;
    assert!(inputs.require_catalog(&foreign, DefId(0)).is_err());
    assert!(inputs.require_catalog(&catalog, DefId(1)).is_err());
    let source_empty = HeaderInputs::source(HashSet::default(), Vec::new());
    assert!(source_empty.require_catalog(&catalog, DefId(0)).is_err());
    assert_eq!(inputs.view().get(0).map(|i| i.value), Some(original.value));
    Ok(())
}

#[test]
fn source_header_adapter_preserves_the_actual_vector_and_set_without_copying()
-> Result<(), &'static str> {
    let mut predicates = Vec::with_capacity(3);
    let site = crate::resolve::unit::NodeKey {
        file: crate::resolve::unit::FileId(0),
        node: crate::syntax::ast::NodeId::new(0),
    };
    predicates.push(Located {
        source: site,
        value: Predicate {
            ty: TypeId::UNIT,
            bound: Bound::Sized,
        }
        .into(),
    });
    let pointer = predicates.as_ptr();
    let capacity = predicates.capacity();
    let mut subjects = HashSet::default();
    subjects.insert(site);
    let subject_capacity = subjects.capacity();
    let inputs = HeaderInputs::source(subjects, predicates);
    let source = inputs.source_ref()?;
    assert_eq!(source.predicates.as_ptr(), pointer);
    assert_eq!(source.predicates.capacity(), capacity);
    assert_eq!(source.self_bounds.capacity(), subject_capacity);
    let input = inputs.view().get(0).ok_or("missing actual source input")?;
    assert!(inputs.is_self(0, input)?);
    Ok(())
}
