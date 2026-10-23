//! Paired block delimiters own tail narrowing independently of captured syntax.
use super::*;
use crate::{
    expand::config::Config,
    resolve::unit::{FileId, Unit},
    support::{edition::Edition, source::SourceFile},
    syntax::{ast::BlockFlavor, token_tree::Span},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn prepared(edition: Edition) -> Result<(Report, Declarations, Function, NodeId, Span)> {
    let source = "macro_rules! make{($e:expr)=>{unsafe{let local=1u8;$e}}}fn touch()->u8{7}fn f(){let result=make!(touch());}";
    let mut report = crate::test_source::resolve(source, true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let facts = crate::sema::bodies::analyze(&report, facts)?;
    let function = *facts.functions.last().ok_or("function")?;
    let unit = &report.package.units[0];
    let block = unit
        .configured
        .ast
        .nodes
        .iter()
        .find(|node| {
            matches!(
                node.kind,
                Syntax::Block {
                    flavor: BlockFlavor::Unsafe,
                    ..
                }
            ) && node.span.start as usize >= unit.source.physical_len()
        })
        .ok_or("generated unsafe block")?;
    let Syntax::Block {
        tail: Some(tail), ..
    } = block.kind
    else {
        return Err("tail".into());
    };
    let closing = Span {
        start: block.span.end - 1,
        end: block.span.end,
    };
    let opening = block.span.start
        + u32::try_from(
            unit.source.text()[block.span.start as usize..block.span.end as usize]
                .find('{')
                .ok_or("generated opening delimiter")?,
        )?;
    let text = "fn donor(){}";
    let open_offset = u32::try_from(text.find('{').ok_or("donor opening delimiter")?)?;
    let offset = u32::try_from(text.rfind('}').ok_or("donor delimiter")?)?;
    let donor = Unit::parse(
        SourceFile::new(text.as_bytes().to_vec())?,
        &Config::in_edition(edition),
        &mut report.package.names,
    )
    .map_err(|error| format!("{error:?}"))?;
    for (generated, original) in [(opening, open_offset), (closing.start, offset)] {
        let origin = report.package.units[0]
            .origins
            .iter_mut()
            .find(|origin| {
                origin.generated.start == generated && origin.generated.end == generated + 1
            })
            .ok_or("exact generated paired delimiter")?;
        origin.original.file = FileId(1);
        origin.original.span = Span {
            start: original,
            end: original + 1,
        };
    }
    report.package.units.push(donor);
    Ok((report, facts, function, tail, closing))
}

#[test]
fn qualified_block_uses_delimiter_edition_instead_of_keyword_or_tail() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        let (report, facts, owner, tail, _) = prepared(edition)?;
        let scopes = function(&report, &facts, &owner)?;
        let Lifetime::Scope(temporary) = scopes.node(tail).ok_or("tail scopes")?.temporary else {
            return Err("stack temporary".into());
        };
        assert_eq!(
            scopes.scopes()[temporary.0 as usize].kind,
            if edition == Edition::Rust2021 {
                Kind::Statement
            } else {
                Kind::Tail
            }
        );
    }
    Ok(())
}

#[test]
fn missing_demanded_delimiter_origin_cannot_default_to_the_owner_edition() -> Result {
    let (mut report, facts, owner, _, closing) = prepared(Edition::Rust2021)?;
    report.package.units[0]
        .origins
        .retain(|origin| origin.generated != closing);
    let error = function(&report, &facts, &owner)
        .err()
        .ok_or("missing origin must fail")?;
    assert!(
        error
            .to_string()
            .contains("missing exact token edition origin")
    );
    Ok(())
}
