//! Formation imports visit a shared source ancestry once and exclude unused roots.
use super::{Environment, FormationInput};
use crate::{
    resolve::model::DefId,
    sema::closures::captures::{Graph, Mode, Step},
    types::TypeId,
};

fn environment(depth: usize) -> Result<(Environment, Vec<FormationInput>), &'static str> {
    let mut graph = Graph::default();
    let root = graph.root(DefId(1))?;
    let mut prefix = root;
    for _ in 0..depth {
        prefix = graph.project(prefix, Step::Dereference { mutable: false })?;
    }
    for index in 0..16 {
        let leaf = graph.project(
            prefix,
            Step::Field(crate::sema::members::Selection::Tuple(index)),
        )?;
        graph.require(leaf, Mode::Shared)?;
    }
    let mentioned = graph.root(DefId(2))?;
    graph.root(DefId(3))?;
    let plan = graph.finish()?;
    let mut child = vec![TypeId::UNIT; plan.path_count()];
    let mut parent = child.clone();
    child[root.index()] = TypeId(11);
    parent[root.index()] = TypeId(13);
    child[mentioned.index()] = TypeId(12);
    parent[mentioned.index()] = TypeId(14);
    Ok((
        Environment {
            plan,
            path_types: child,
            formation_types: parent,
            field_types: Vec::new(),
            formation_fields: Vec::new(),
            accesses: crate::support::hash::HashMap::default(),
            adjusted_accesses: crate::support::hash::HashMap::default(),
            nested_accesses: crate::support::hash::HashMap::default(),
            mentions: vec![mentioned, mentioned],
            binding_mutability: crate::support::hash::HashMap::default(),
        },
        vec![
            FormationInput {
                root,
                child: TypeId(11),
                parent: TypeId(13),
            },
            FormationInput {
                root: mentioned,
                child: TypeId(12),
                parent: TypeId(14),
            },
        ],
    ))
}
#[test]
fn selected_roots_preserve_both_views_with_linear_shared_ancestry() -> Result<(), &'static str> {
    for depth in [8, 32, 128, 2048] {
        let (environment, expected) = environment(depth)?;
        let (inputs, work) = environment.formation_inputs()?;
        assert_eq!(inputs, expected);
        assert_eq!(work, depth + 21);
    }
    Ok(())
}
#[test]
fn incomplete_parent_type_coverage_is_rejected() -> Result<(), &'static str> {
    let (mut environment, _) = environment(8)?;
    environment.formation_types.pop();
    assert!(environment.formation_inputs().is_err());
    Ok(())
}
