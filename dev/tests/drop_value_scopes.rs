use crate::qualify as rustc;
use crate::semantic;
use rustgm::body::{
    self,
    ir::{Action, Owner, Rvalue, Terminator},
    scopes::{Kind, Lifetime},
    storage::Location,
};

#[test]
fn discarded_values_and_forwarded_results_retain_cleanup_scopes()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("discarded", "fn f(){D;}", false, Kind::Statement),
        ("block-result", "fn f(){{D};}", false, Kind::Statement),
        ("nested-result", "fn f(){{{D}};}", false, Kind::Statement),
        ("returned", "fn f()->D{D}", true, Kind::Body),
        ("returned-block", "fn f()->D{{D}}", true, Kind::Body),
        (
            "explicit-return",
            "fn f()->D{return {D};}",
            true,
            Kind::Statement,
        ),
    ];
    for (name, function, returned, expected) in cases {
        let source = format!("struct D;impl Drop for D{{fn drop(&mut self){{}}}}{function}");
        rustc::qualify(name, &source, true)?;
        let (report, facts) = semantic::analyze(&source)?;
        let program = body::build(&report, &facts)?;
        let cfg=program.bodies.iter().find(|b|matches!(b.owner,Owner::Function{definition,..} if report.package.names.get(report.resolution.definitions[definition.0 as usize].name)=="f")).ok_or("missing f")?;
        let owned = cfg.owned.as_ref().ok_or("missing owned storage")?;
        assert_eq!(owned.created.len(), 1, "{name}");
        let value = &owned.created[0];
        let Lifetime::Scope(scope) = value.lifetime else {
            return Err("unexpected permanent value".into());
        };
        let storage = cfg.storage.as_ref().ok_or("storage")?;
        assert_eq!(storage.scopes[scope.0 as usize].kind, expected, "{name}");
        assert!(
            cfg.temporaries.is_empty(),
            "unborrowed values must not invent place materializations: {name}"
        );
        assert_eq!(owned.returned.len(), usize::from(returned), "{name}");
        if let Some(release) = owned.returned.first() {
            assert_eq!(release.value, value.value);
            let Location::At { block, before } = release.location else {
                return Err("return is not at a point".into());
            };
            assert!(
                matches!(cfg.blocks[block.0 as usize].terminator,Terminator::Return(id) if id==value.value)
            );
            assert!(value.before <= before);
            let exit = &storage.exits[release.exits_before as usize];
            assert_eq!(exit.location, release.location);
        }
        let Action::Evaluate(created) = cfg.statements[(value.before - 1) as usize].action else {
            return Err("birth does not follow evaluation".into());
        };
        assert_eq!(created, value.value);
    }
    Ok(())
}

#[test]
fn borrowed_owned_values_keep_extension_and_constructors_transfer_their_result()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct D;impl Drop for D{fn drop(&mut self){}}struct W(D);fn f(){let _hold=&{D};let _again=&D;let _pair=W(D);}";
    rustc::qualify("owned-extended-and-constructor", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts)?;
    let mut borrowed = 0;
    let mut constructors = 0;
    for body in &program.bodies {
        let Some(owned) = &body.owned else { continue };
        for place in &body.temporaries {
            if let Some(value) = owned.creation(place.value) {
                assert_eq!(value.lifetime, place.lifetime);
                borrowed += 1;
            }
        }
        if let Owner::Function { definition, .. } = body.owner
            && facts.members.constructors.contains_key(&definition)
        {
            constructors += 1;
            let release = owned
                .returned
                .first()
                .ok_or("constructor result is not transferred")?;
            assert!(matches!(
                body.values[release.value.0 as usize].kind,
                Rvalue::Aggregate { .. }
            ));
            assert_eq!(owned.created.len(), 2);
        }
        for entry in &owned.created {
            assert!(owned.creation(entry.value).is_some());
        }
    }
    assert_eq!(borrowed, 2);
    assert_eq!(constructors, 1);
    Ok(())
}
