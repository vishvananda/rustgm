//! Aliases preserve identities and validate exposure before resolver allocation.
use super::{CrateIdentity, ExternalDef, ExternalKind, Interface, Symbol, crates, function};
use crates::{Dependencies, PublicBinding, PublicModule, PublicNames};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(super) fn roundtrip(artifact: &Interface) -> Result {
    let old = crates::encode(artifact)?;
    let definitions = artifact
        .functions()
        .iter()
        .map(crates::Function::definition)
        .chain(
            artifact
                .constants()
                .iter()
                .map(crates::Constant::definition),
        )
        .chain(
            artifact
                .records()
                .iter()
                .filter(|row| row.public())
                .map(crates::Record::definition),
        )
        .chain(
            artifact
                .storage()
                .into_iter()
                .flat_map(crates::Storage::statics)
                .filter(|row| row.public())
                .map(crates::Static::definition),
        );
    let bindings = definitions
        .enumerate()
        .map(|(index, definition)| {
            PublicBinding::new(format!("exported{index}"), definition.clone())
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let named = crates::decode(&old)?.with_public_names(root(bindings.clone())?)?;
    let encoded = crates::encode(&named)?;
    assert_eq!(&encoded[12..14], &[6, old[12]]);
    assert_eq!(&encoded[14..=old.len()], &old[13..]);
    assert_eq!(crates::decode(&encoded)?, named);
    for hidden in artifact
        .records()
        .iter()
        .filter(|row| !row.public())
        .map(crates::Record::definition)
        .chain(
            artifact
                .storage()
                .into_iter()
                .flat_map(crates::Storage::statics)
                .filter(|row| !row.public())
                .map(crates::Static::definition),
        )
    {
        let mut forged = bindings.clone();
        forged.push(PublicBinding::new("leaked".into(), hidden.clone())?);
        assert!(
            crates::decode(&old)?
                .with_public_names(root(forged)?)
                .is_err()
        );
    }
    Ok(())
}

fn module(owner: &CrateIdentity, path: &[&str]) -> Result<ExternalDef> {
    Ok(ExternalDef::new(
        owner.clone(),
        path.iter().map(|s| (*s).into()).collect(),
        ExternalKind::Module,
    )?)
}
fn root(bindings: Vec<PublicBinding>) -> Result<PublicNames> {
    Ok(PublicNames::new(vec![PublicModule::new(
        Vec::new(),
        bindings,
    )?])?)
}
fn provider() -> Result<Interface> {
    let owner = CrateIdentity::new("same", "first")?;
    let item = function(&owner, &["implementation", "run"], Symbol::Rust)?;
    let names = root(vec![
        PublicBinding::new("first".into(), item.definition().clone())?,
        PublicBinding::new("second".into(), item.definition().clone())?,
    ])?;
    Ok(Interface::new(owner, vec![], vec![item])?.with_public_names(names)?)
}

#[test]
fn renamed_bindings_retain_one_definition_and_legacy_payload() -> Result {
    let named = provider()?;
    let original = Interface::new(named.identity().clone(), vec![], named.functions().to_vec())?;
    let old = crates::encode(&original)?;
    let bytes = crates::encode(&named)?;
    assert_eq!(&bytes[..14], b"RGMETA\r\n\x01\0\0\0\x06\0");
    assert_eq!(&bytes[14..=old.len()], &old[13..]);
    assert_eq!(crates::decode(&bytes)?, named);
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    assert_eq!(named.functions().len(), 1);
    assert_eq!(
        named.functions()[0].definition(),
        original.functions()[0].definition()
    );
    assert_eq!(
        named.functions()[0].definition().rust_symbol(),
        original.functions()[0].definition().rust_symbol()
    );
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(crates::decode(&trailing).is_err());
    let mut recursive = bytes;
    recursive[13] = 6;
    assert!(crates::decode(&recursive).is_err());
    Ok(())
}

#[test]
fn shared_module_aliases_and_resolved_cycles_do_not_expand_paths() -> Result {
    let owner = CrateIdentity::new("modules", "first")?;
    let item = function(&owner, &["private", "run"], Symbol::Rust)?;
    for width in [8, 32, 128] {
        let mut aliases = (0..width)
            .map(|n| {
                PublicBinding::new(
                    format!("alias{n:03}"),
                    module(&owner, &["private"]).map_err(|_| "module")?,
                )
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        aliases.reverse();
        let names = PublicNames::new(vec![
            PublicModule::new(
                vec!["private".into()],
                vec![
                    PublicBinding::new("back".into(), module(&owner, &[])?)?,
                    PublicBinding::new("run".into(), item.definition().clone())?,
                ],
            )?,
            PublicModule::new(vec![], aliases)?,
        ])?;
        assert_eq!(names.modules().len(), 2);
        let interface =
            Interface::new(owner.clone(), vec![], vec![item.clone()])?.with_public_names(names)?;
        let bytes = crates::encode(&interface)?;
        assert!(bytes.len() < 400 + 40 * width);
        assert_eq!(crates::decode(&bytes)?, interface);
    }
    Ok(())
}

#[test]
fn duplicate_names_conflict_only_in_the_same_namespace() -> Result {
    let owner = CrateIdentity::new("names", "first")?;
    let item = function(&owner, &["run"], Symbol::Rust)?;
    let binding = PublicBinding::new("same".into(), item.definition().clone())?;
    assert!(PublicModule::new(vec![], vec![binding.clone(), binding.clone()]).is_err());
    let names = root(vec![
        binding,
        PublicBinding::new("same".into(), module(&owner, &[])?)?,
    ])?;
    let interface = Interface::new(owner, vec![], vec![item])?.with_public_names(names)?;
    assert_eq!(crates::decode(&crates::encode(&interface)?)?, interface);
    assert!(PublicNames::new(vec![]).is_err());
    assert!(
        PublicNames::new(vec![PublicModule::new(vec!["absent_root".into()], vec![])?]).is_err()
    );
    assert!(
        PublicNames::new(vec![
            PublicModule::new(vec![], vec![])?,
            PublicModule::new(vec![], vec![])?
        ])
        .is_err()
    );
    Ok(())
}

#[test]
fn missing_and_unreachable_owned_targets_are_rejected() -> Result {
    let owner = CrateIdentity::new("missing", "first")?;
    let item = function(&owner, &["run"], Symbol::Rust)?;
    let base = || Interface::new(owner.clone(), vec![], vec![item.clone()]);
    assert!(base()?.with_public_names(root(vec![])?).is_err());
    let absent = function(&owner, &["absent"], Symbol::Rust)?;
    assert!(
        base()?
            .with_public_names(root(vec![PublicBinding::new(
                "run".into(),
                absent.definition().clone()
            )?])?)
            .is_err()
    );
    let names = PublicNames::new(vec![
        PublicModule::new(
            vec![],
            vec![PublicBinding::new("run".into(), item.definition().clone())?],
        )?,
        PublicModule::new(vec!["unreachable".into()], vec![])?,
    ])?;
    assert!(base()?.with_public_names(names).is_err());
    let foreign = CrateIdentity::new("missing", "second")?;
    assert!(
        base()?
            .with_public_names(root(vec![PublicBinding::new(
                "module".into(),
                module(&foreign, &[])?
            )?])?)
            .is_err()
    );
    Ok(())
}

fn facade(target: ExternalDef, dependency: CrateIdentity) -> Result<Interface> {
    Ok(Interface::new(
        CrateIdentity::new("facade", "first")?,
        vec![dependency],
        vec![],
    )?
    .with_public_names(root(vec![PublicBinding::new("exported".into(), target)?])?)?)
}
#[test]
fn dependency_targets_use_defining_identity_and_complete_exposure() -> Result {
    let first = provider()?;
    let target = first.functions()[0].definition().clone();
    let alias = facade(target.clone(), first.identity().clone())?;
    let canonical = crates::encode(&alias)?;
    let graph = Dependencies::new(vec![alias, first], vec![])?;
    assert_eq!(graph.edges(), 1);
    let first = provider()?;
    let alias = facade(target.clone(), first.identity().clone())?;
    assert_eq!(crates::encode(&alias)?, canonical);
    assert_eq!(Dependencies::new(vec![first, alias], vec![])?.edges(), 1);
    let first = provider()?;
    let absent = ExternalDef::new(
        target.owner().clone(),
        vec!["hidden".into()],
        ExternalKind::Function,
    )?;
    let alias = facade(absent, first.identity().clone())?;
    assert!(Dependencies::new(vec![first, alias], vec![]).is_err());
    let first = provider()?;
    let private_module = facade(
        module(first.identity(), &["implementation"])?,
        first.identity().clone(),
    )?;
    assert!(Dependencies::new(vec![first, private_module], vec![]).is_err());
    let other = CrateIdentity::new("same", "second")?;
    assert!(facade(target, other).is_err());
    Ok(())
}

#[test]
fn legacy_dependency_modules_retain_their_original_public_paths() -> Result {
    let owner = CrateIdentity::new("legacy", "first")?;
    let item = function(&owner, &["nested", "run"], Symbol::Rust)?;
    let base = Interface::new(owner.clone(), vec![], vec![item])?;
    let alias = facade(module(&owner, &["nested"])?, owner)?;
    assert_eq!(Dependencies::new(vec![alias, base], vec![])?.edges(), 1);
    Ok(())
}
