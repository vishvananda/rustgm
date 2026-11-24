//! Validate the source directives that describe a failed call or macro match.
use super::attribute_directives::{OptionKind, Parser};
use crate::{
    lex::token::Punctuation,
    resolve::{
        expansion::Report,
        package::Error,
        unit::{FileId, NodeKey},
    },
    sema::model::Declarations,
    support::diagnostics::WarningCode,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::Forest,
    },
};

/// The directives whose crate feature makes rustc read them.
#[derive(Clone, Copy)]
enum Directive {
    Const,
    UnmatchedArgs,
    Move,
}

/// Crate requests only control diagnostic metadata; no operation is authorized.
#[derive(Default)]
pub(super) struct Requests([Option<bool>; 3]);
impl Requests {
    fn enabled(&mut self, report: &Report, directive: Directive) -> Result<bool, Error> {
        let index = directive as usize;
        if let Some(enabled) = self.0[index] {
            return Ok(enabled);
        }
        let package = &report.package;
        let enabled = match directive {
            Directive::Const => package.requested_diagnostic_directive(false),
            Directive::UnmatchedArgs => package.requested_diagnostic_directive(true),
            Directive::Move => package.requested_move_directive(),
        }
        .map_err(Error::unlocated)?;
        self.0[index] = Some(enabled);
        Ok(enabled)
    }
}

fn recognized(attribute: Attribute, forest: &Forest, source: &str) -> Option<Directive> {
    let path = attribute.path;
    if path.end != path.start + 3
        || forest.name(source, path.start) != Some("diagnostic")
        || !forest.punctuation(path.start + 1, Punctuation::ColonColon)
    {
        return None;
    }
    match forest.name(source, path.start + 2)? {
        "on_const" => Some(Directive::Const),
        "on_unmatched_args" => Some(Directive::UnmatchedArgs),
        // `#[diagnostic::on_move]` only rewords a use-after-move error.
        "on_move" => Some(Directive::Move),
        _ => None,
    }
}

pub(super) fn validate_literals(
    file: FileId,
    source: &str,
    attribute: Attribute,
    forest: &Forest,
) -> Result<(), Error> {
    let range = match attribute.input {
        Input::Delimited(group) => forest.children(group),
        Input::Equals(range) => Some(range),
        Input::Empty | Input::Doc(_) => None,
    };
    if let Some(range) = range {
        // Flat retained input visits each actual token, including nested groups,
        // once. Disabled owners still require valid literal escapes.
        for index in range.start..range.end {
            if let Some(token) = forest.token(index) {
                crate::literal::validate_attribute_text(source, token)
                    .map_err(|error| Error::at(file, error.offset, error.message))?;
            }
        }
    }
    Ok(())
}

pub(super) fn defer(
    report: &Report,
    facts: &mut Declarations,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    requests: &mut Requests,
) -> Result<bool, Error> {
    let file = key.file;
    let source = report.package.units[file.0 as usize].source.text();
    let Some(directive) = recognized(attribute, forest, source) else {
        return Ok(false);
    };
    if attribute.unsafe_wrapper {
        return Err(Error::at(
            file,
            attribute.span.start,
            "diagnostic directive cannot use an unsafe attribute wrapper",
        ));
    }
    validate_literals(file, source, attribute, forest)?;
    facts.require_lint_context(report)?;
    if !report.package.lint_cap().suppresses_lints() {
        // The pinned handlers ignore metadata when their crate feature is absent;
        // allow_internal_unstable does not enable any diagnostic handler.
        if !requests.enabled(report, directive)? {
            return Ok(true);
        }
        if matches!(directive, Directive::Move) {
            return Err(Error::at(
                file,
                attribute.span.start,
                "uncapped move diagnostic directive profile is not implemented",
            ));
        }
        let mut parser = Parser::new(report, facts, key, attribute, forest);
        check_owner(&mut parser, directive)?;
        check_options(&mut parser, attribute, directive)?;
    }
    Ok(true)
}

fn check_owner(parser: &mut Parser<'_, '_>, directive: Directive) -> Result<(), Error> {
    let ast = &parser.report.package.unit(parser.key.file).configured.ast;
    let mut node = parser.key.node;
    while let Kind::Attributed { value, .. } = ast.node(node).kind {
        node = value;
    }
    let item = if let Kind::Item(index) = ast.node(node).kind {
        ast.items.get(index as usize).map(|item| item.kind)
    } else {
        None
    };
    let invalid = match directive {
        Directive::Const
            if !matches!(
                item,
                Some(ItemKind::Impl {
                    constant: false,
                    trait_path: Some(_),
                    ..
                })
            ) =>
        {
            Some("on_const requires a non-const trait implementation")
        }
        Directive::UnmatchedArgs
            if !matches!(
                item,
                Some(ItemKind::MacroRules { .. } | ItemKind::DeclarativeMacro { .. })
            ) =>
        {
            Some("on_unmatched_args requires a macro definition")
        }
        _ => None,
    };
    if let Some(message) = invalid {
        parser.warn(WarningCode::MisplacedDiagnosticAttributes, message)?;
    }
    Ok(())
}

fn check_options(
    parser: &mut Parser<'_, '_>,
    attribute: Attribute,
    directive: Directive,
) -> Result<(), Error> {
    let Some(parts) = parser.option_parts(attribute)? else {
        return parser.warn(
            WarningCode::MalformedDiagnosticAttributes,
            "diagnostic directive requires a nonempty list",
        );
    };
    let mut seen = [false; 2];
    for part in parts {
        let Some((option, value)) = parser.option(part)? else {
            parser.warn(
                WarningCode::MalformedDiagnosticAttributes,
                "malformed diagnostic directive option",
            )?;
            continue;
        };
        let single = match option {
            OptionKind::Message => Some(0),
            OptionKind::Label => Some(1),
            OptionKind::Note => None,
        };
        if single.is_some_and(|index| std::mem::replace(&mut seen[index], true)) {
            parser.warn(
                WarningCode::MalformedDiagnosticAttributes,
                "duplicate diagnostic directive option",
            )?;
        }
        parser.charge(value.len(), 0)?;
        if !valid_format(&value, directive) {
            let message = if matches!(directive, Directive::Const) {
                "invalid on_const format literal"
            } else {
                "invalid on_unmatched_args format literal"
            };
            parser.warn(WarningCode::MalformedDiagnosticFormatLiterals, message)?;
        }
    }
    Ok(())
}

fn valid_format(mut text: &str, directive: Directive) -> bool {
    while let Some(index) = text.find(['{', '}']) {
        let brace = text.as_bytes()[index];
        text = &text[index + 1..];
        if text.as_bytes().first() == Some(&brace) {
            text = &text[1..];
            continue;
        }
        let Some((name, rest)) = text.split_once('}') else {
            return false;
        };
        if brace == b'}' || !valid_argument(name, directive) {
            return false;
        }
        text = rest;
    }
    true
}

fn valid_argument(name: &str, directive: Directive) -> bool {
    if matches!(directive, Directive::UnmatchedArgs) {
        return name == "This";
    }
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || unicode_ident::is_xid_start(first))
        && chars.all(unicode_ident::is_xid_continue)
}

#[cfg(test)]
#[path = "../../../tests/invariants/diagnostic_directives.rs"]
mod tests;
