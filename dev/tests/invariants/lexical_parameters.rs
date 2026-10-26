//! The real Self producer retains formal slots without fabricating trait evidence.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{self, load::{InputFile, Sources}, model::{DefKind, Namespace}},
    sema::regions::{Context, Position},
    support::source::SourceFile,
    types::{RegionId, TypeId, arguments::{Argument, ConstKind}},
};
use std::path::Path;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> { Ok(None) }
}
fn report() -> TestResult<crate::resolve::expansion::Report> {
    let source="trait P<'a,T,const N:usize>{}fn main(){}";
    let report=resolve::expansion::analyze(InputFile {
        identity:"self.rs".into(),path:"self.rs".into(),source:SourceFile::new(source.as_bytes().to_vec())?,
    },&mut NoFiles,&Config::default())?;
    Ok(resolve::analyze_expanded(report)?)
}
fn owner(report:&crate::resolve::expansion::Report)->TestResult<(DefId,crate::sema::associated::SourceOwner)> {
    let index=report.resolution.definitions.iter().position(|d| d.kind==DefKind::Trait && report.package.names.get(d.name)=="P").ok_or("actual generic trait")?;
    let definition=DefId(u32::try_from(index)?);
    let facts=crate::sema::associated::collect(report)?;
    let owner=facts.get(facts.owner(definition).ok_or("actual owner")?).ok_or("actual owner row")?.source_owner()?;
    Ok((definition,owner))
}
#[test]
fn lexical_self_preparation_retains_full_formals_without_trait_evidence_or_early_completion() -> TestResult {
    let report=report()?;
    let (definition,owner)=owner(&report)?;
    let mut checker=Checker::new(&report)?;
    let mut schema=None;
    checker.reserve_lexical_self(owner.source,definition,owner.self_definition,|_,value|{schema=Some(value);Ok(())})?;
    let schema=schema.ok_or("actual shared schema producer")?;
    assert_eq!(schema.captures,1);
    assert!(schema.bounds.is_empty());
    assert!(checker.facts.associated.schema(definition).is_none(),"schema installation belongs to the actual declaration kind");
    assert!(!checker.facts.generics.trait_self_bounds.contains_key(&definition));
    assert!(checker.facts.generics.traits.is_none());
    assert_eq!(checker.facts.generics.parents.get(&definition),Some(&owner.self_definition));
    let self_type=checker.facts.definitions[owner.self_definition.0 as usize].ok_or("complete rigid Self")?;
    assert_eq!(self_type,schema.self_type);
    let Type::Parameter(id)=checker.facts.types.kind(self_type).ok_or("actual Self kind")? else{return Err("Self must retain its own parameter identity".into())};
    let parameter=checker.facts.types.parameter(id).ok_or("canonical Self parameter")?;
    assert_eq!(parameter.owner,ItemId(owner.self_definition.0));
    assert_eq!(parameter.kind,ParameterKind::Type{sized:false});
    let binder=checker.owner_binders[&owner.self_definition];
    checker.declaration_parameters(definition,owner.source,Context{binder:Some(binder),position:Position::Output})?;
    checker.drain(&mut 0,10_000)?;
    let parameters=checker.facts.generics.parameters.get(&definition).ok_or("actual full formals")?;
    assert_eq!(parameters.len(),3,"unused lifetime/type/const formals cannot disappear");
    assert!(matches!(parameters[0].argument,Argument::Lifetime(region) if region==RegionId::bound(0,0)?));
    let Argument::Type(ty)=parameters[1].argument else{return Err("actual T argument".into())};
    let Type::Parameter(id)=checker.facts.types.kind(ty).ok_or("T kind")? else{return Err("actual T parameter".into())};
    let parameter=checker.facts.types.parameter(id).ok_or("T declaration")?;
    assert_eq!((parameter.owner,parameter.index),(ItemId(definition.0),1));
    let Argument::Const(term)=parameters[2].argument else{return Err("actual N argument".into())};
    let ConstKind::Parameter(id)=checker.facts.types.const_kind(term).ok_or("N kind")? else{return Err("actual N parameter".into())};
    let parameter=checker.facts.types.parameter(id).ok_or("N declaration")?;
    assert_eq!((parameter.owner,parameter.index),(ItemId(definition.0),2));
    assert_eq!(parameter.kind,ParameterKind::Const{ty:TypeId::USIZE});
    assert_eq!(report.resolution.definitions[parameters[0].binding.0 as usize].namespace,Namespace::Lifetime);
    let mut rejected=Checker::new(&report)?;
    let error=rejected.reserve_lexical_self(owner.source,definition,owner.self_definition,|_,_|Err("actual owner rejected schema")).err().ok_or("rejected owner completed Self")?;
    assert_eq!(error.message,"actual owner rejected schema");
    assert!(rejected.facts.definitions[owner.self_definition.0 as usize].is_none());
    assert!(!rejected.owner_binders.contains_key(&owner.self_definition));
    assert!(!rejected.facts.lifetimes.parameters.contains_key(&definition));
    Ok(())
}
