//! Component checks for internal guard-view storage and lowering.
use rustgm::{
    body::{self, ir::*},
    expand::config::Config,
    lowir::writer,
    resolve::{
        self,
        load::{InputFile, Sources},
        model::DefId,
    },
    sema::{bodies, declarations},
    support::source::SourceFile,
};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
const CHECK: &str = "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:i32)->i32{1/x}";
const CASES: &[(&str, &str, &str)] = &[
    (
        "scalar-place",
        "let x=7u64;let n=x;check(n==7);check(x==7);",
        "let x=7u64;let n=&x;check(*n==7);check(x==7);",
    ),
    (
        "scalar-temporary",
        "let n=7u64;check(n==7);",
        "let n=&7u64;check(*n==7);",
    ),
    (
        "product",
        "let x=(7u64,9u64);let n=x;check(n.0+n.1==16);check(x.0==7);",
        "let x=(7u64,9u64);let n=&x;check(n.0+n.1==16);check(x.0==7);",
    ),
    (
        "noncopy",
        "let x=N(7,9);let n=x;check(n.0+n.1==16);let m=x;check(m.0==7);",
        "let x=N(7,9);let n=&x;check(n.0+n.1==16);let m=x;check(m.0==7);",
    ),
    (
        "noncopy-temporary",
        "let n=N(7,9);check(n.0+n.1==16);",
        "let n=&N(7,9);check(n.0+n.1==16);",
    ),
    (
        "array",
        "let x=[7u64,9];let n=x;check(n[0]+n[1]==16);check(x[1]==9);",
        "let x=[7u64,9];let n=&x;check(n[0]+n[1]==16);check(x[1]==9);",
    ),
    (
        "zero-sized",
        "let x=((),7u64);let n=x.0;let _=n;check(x.1==7);",
        "let x=((),7u64);let n=&x.0;let _=n;check(x.1==7);",
    ),
    (
        "branch-address",
        "let x=(7u64,9u64);let n=if x.0==7{x}else{(1,2)};check(n.0+n.1==16);",
        "let x=(7u64,9u64);let n=&if x.0==7{x}else{(1,2)};check(n.0+n.1==16);",
    ),
    (
        "repeated-view",
        "let x=[N(3,0),N(5,0)];let mut i=0usize;let mut sum=0u64;while i<2{let n=x[i];sum+=n.0;i+=1;}check(sum==8);",
        "let x=[N(3,0),N(5,0)];let mut i=0usize;let mut sum=0u64;while i<2{let n=&x[i];sum+=n.0;i+=1;}check(sum==8);",
    ),
    (
        "function-item",
        "let n=check;n(true);",
        "let n=&check;n(true);",
    ),
];
fn install_view(body: &mut Body, local: LocalId) -> Result<(), &'static str> {
    let source = body.locals[local.0 as usize].source;
    body.locals[local.0 as usize].kind = LocalKind::Guard(GuardId(0));
    body.guards.push(Guard {
        tested_start: 0,
        tested_end: 0,
        views_start: local.0,
        views_end: local.0 + 1,
    });
    let mut selected = None;
    for statement in &body.statements {
        if let Action::Store {
            place: Place::Local(target),
            value,
        } = statement.action
            && target == local
            && selected.replace(value).is_some()
        {
            return Err("fixture binding initialized twice");
        }
    }
    let value = selected.ok_or("missing fixture initializer")?;
    let origin = match body.values[value.0 as usize].kind {
        Rvalue::Read(place) => place,
        _ => Place::Temporary(value),
    };
    let old = std::mem::take(&mut body.statements);
    let mut locations = std::collections::HashMap::new();
    let mut birth = None;
    for (index, block) in body.blocks.iter_mut().enumerate() {
        let block_id = BlockId(u32::try_from(index).map_err(|_| "blocks")?);
        let range = block.start as usize..block.end as usize;
        block.start = u32::try_from(body.statements.len()).map_err(|_| "statements")?;
        let old_end = u32::try_from(range.end).map_err(|_| "statements")?;
        for at in range {
            locations.insert(
                (block_id, u32::try_from(at).map_err(|_| "statements")?),
                u32::try_from(body.statements.len()).map_err(|_| "statements")?,
            );
            let statement = &old[at];
            match statement.action {
                Action::Evaluate(id)
                    if id == value
                        && matches!(body.values[id.0 as usize].kind, Rvalue::Read(_)) => {}
                Action::Store {
                    place: Place::Local(target),
                    ..
                } if target == local => {
                    body.statements.push(Statement {
                        source,
                        action: Action::GuardStart(GuardId(0)),
                    });
                    birth = Some((
                        block_id,
                        u32::try_from(body.statements.len()).map_err(|_| "statements")?,
                    ));
                    body.statements.push(Statement {
                        source,
                        action: Action::BindView {
                            view: local,
                            place: origin,
                        },
                    });
                }
                action => body.statements.push(Statement {
                    source: statement.source,
                    action,
                }),
            }
        }
        block.end = u32::try_from(body.statements.len()).map_err(|_| "statements")?;
        locations.insert((block_id, old_end), block.end);
    }
    remap_storage(body, &locations)?;
    if let Place::Temporary(value) = origin
        && !body.temporaries.iter().any(|t| t.value == value)
    {
        let (block, before) = birth.ok_or("missing temporary view birth")?;
        body.temporaries.push(Temporary {
            value,
            block,
            before,
            source,
            exits_before: 0,
            lifetime: body::scopes::Lifetime::Scope(body.locals[local.0 as usize].scope),
        });
    }
    Ok(())
}
fn remap_storage(
    body: &mut Body,
    locations: &std::collections::HashMap<(BlockId, u32), u32>,
) -> Result<(), &'static str> {
    for temporary in &mut body.temporaries {
        temporary.before = *locations
            .get(&(temporary.block, temporary.before))
            .ok_or("missing remapped temporary")?;
    }
    for exit in &mut body
        .storage
        .as_mut()
        .ok_or("missing fixture storage")?
        .exits
    {
        if let body::storage::Location::At { block, before } = &mut exit.location {
            *before = *locations
                .get(&(*block, *before))
                .ok_or("missing remapped exit")?;
        }
    }
    Ok(())
}
fn lower(source: &str) -> Result<Vec<u8>, String> {
    let report = resolve::expansion::analyze(
        InputFile {
            identity: "views.rs".into(),
            path: "views.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )
    .and_then(resolve::analyze_expanded)
    .map_err(|e| e.to_string())?;
    let mut facts = declarations::analyze(&report)
        .and_then(|f| bodies::analyze(&report, f))
        .map_err(|e| e.to_string())?;
    let mut program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let mut changed = 0;
    for body in &mut program.bodies {
        let binding = body.locals.iter().position(|local| {
            local.definition.is_some_and(|id| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[id.0 as usize].name)
                    == "n"
            })
        });
        if let Some(index) = binding {
            install_view(body, LocalId(u32::try_from(index).map_err(|_| "locals")?))?;
            changed += 1;
        }
    }
    if changed != 1 {
        return Err("expected one fixture view".into());
    }
    let contracts = rustgm::body::regions::collect(&facts)?;
    let contracts = rustgm::borrow::regions::check_bodies(
        &facts.types,
        &facts.aggregates,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )
    .map_err(|e| e.to_string())?;
    let checked = rustgm::borrow::verify(
        &mut rustgm::borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            copies: &mut facts.copies,
        },
        &contracts,
        &program,
    )
    .map_err(|e| e.message)?;
    let entry = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == "main")
        .ok_or("entry")?;
    let lowered = rustgm::lower::build(
        &mut facts,
        &checked,
        DefId(u32::try_from(entry).map_err(|_| "definition")?),
    )
    .map_err(|e| e.message)?;
    let mut text = Vec::new();
    writer::write(&lowered, &mut text).map_err(|e| e.to_string())?;
    Ok(text)
}
#[test]
fn guard_views_lower_without_owning_their_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/pa13-work/guard-view-native-{stamp}"));
    std::fs::create_dir_all(&folder)?;
    for &(name, candidate, independent) in CASES {
        let source = format!("struct N(u64,u64);{CHECK}fn main(){{{candidate}}}");
        let expected = format!("struct N(u64,u64);{CHECK}fn main(){{{independent}}}");
        let lowered = lower(&source).map_err(|e| format!("{name}: {e}"))?;
        assert!(!lowered.is_empty());
        std::fs::write(folder.join(format!("{name}.lowir")), lowered)?;
        std::fs::write(folder.join(format!("{name}.rs")), source)?;
        std::fs::write(folder.join(format!("{name}.expected.rs")), expected)?;
    }
    eprintln!("Guard view native inputs: {}", folder.display());
    Ok(())
}

#[test]
fn guard_views_reject_unaligned_packed_fields_even_when_zero_sized() {
    for source in [
        "#[repr(packed)]struct S{a:u8,b:u64}fn main(){let x=S{a:1,b:7};let n=x.b;let _=n;}",
        "#[repr(packed)]struct S{a:u8,b:[u64;0]}fn main(){let x=S{a:1,b:[]};let n=x.b;let _=n;}",
    ] {
        let error = lower(source).err();
        assert_eq!(
            error.as_deref(),
            Some("guard binding would reference an unaligned packed field")
        );
    }
}
