//! PA7 performs one matching/transcription step for source-written invocations.
//! Generated syntax, path imports and hygiene are consumers of these PA8 facts.
use super::{
    compile,
    input::Input,
    matcher::{Compiled, Matched, Workspace},
    prepare::{self, Prepared},
    transcribe::{self, Transcription},
};
use crate::support::hash::HashMap;
use crate::{
    expand::{config::Config, crate_syntax},
    lex::token::LexError,
    support::names::{NameId, Names},
    syntax::{
        ast::{Kind, NodeId},
        item::ItemKind,
        token_tree::{Forest, Span, error},
    },
};

pub struct Definition {
    pub name: NameId,
    pub span: Span,
    pub compiled: Compiled,
}
pub struct Invocation {
    pub span: Span,
    pub definition: usize,
    pub input: Input,
    pub prepared: Option<Prepared>,
    pub matched: Matched,
    pub transcription: Transcription,
}
#[derive(Default)]
pub struct Report {
    pub names: Names,
    pub definitions: Vec<Definition>,
    pub invocations: Vec<Invocation>,
}
enum Work {
    Visit(NodeId),
    Restore(usize),
}
/// # Errors
/// Reports source syntax/configuration errors, invalid definitions and matching failures.
pub fn analyze(source: &str, config: &Config) -> Result<Report, LexError> {
    let configured = crate_syntax::analyze(source, config)?;
    let ast = &configured.ast;
    let mut report = Report::default();
    let mut visible = HashMap::default();
    let mut changes = Vec::new();
    let mut work = vec![Work::Visit(ast.root)];
    let mut children = Vec::new();
    let mut scratch = Workspace::default();
    while let Some(task) = work.pop() {
        let node = match task {
            Work::Restore(mark) => {
                for (name, previous) in changes.drain(mark..).rev() {
                    if let Some(previous) = previous {
                        visible.insert(name, previous);
                    } else {
                        visible.remove(&name);
                    }
                }
                continue;
            }
            Work::Visit(node) => node,
        };
        if !configured.active[node.0 as usize] {
            continue;
        }
        let data = ast.node(node);
        let mut scope = matches!(data.kind, Kind::Block { .. });
        if let Kind::Item(index) = data.kind {
            let item = ast.items[index as usize];
            if let ItemKind::MacroRules { capture } = item.kind {
                let name_span = item
                    .name
                    .ok_or_else(|| error(data.span.start, "macro definition has no name"))?;
                let name = report
                    .names
                    .intern(&source[name_span.start as usize..name_span.end as usize])
                    .map_err(|e| error(name_span.start, e))?;
                let forest = &ast.captures[capture as usize];
                let definition = compile::compile(source, forest, &mut report.names)?;
                let index = report.definitions.len();
                report.definitions.push(Definition {
                    name,
                    span: data.span,
                    compiled: Compiled::new(definition),
                });
                changes.push((name, visible.insert(name, index)));
                continue;
            }
            scope |= matches!(
                item.kind,
                ItemKind::Module { .. } | ItemKind::Trait { .. } | ItemKind::Impl { .. }
            );
        }
        if let Kind::Macro { path, capture, .. } = data.kind {
            let span = match ast.node(path).kind {
                Kind::Name(span) => span,
                Kind::Path {
                    absolute: false,
                    segments,
                } if ast.list(segments).len() == 1 => ast.node(ast.list(segments)[0]).span,
                _ => {
                    return Err(error(
                        data.span.start,
                        "qualified macro lookup requires PA8 expansion resolution",
                    ));
                }
            };
            let name = report
                .names
                .intern(&source[span.start as usize..span.end as usize])
                .map_err(|e| error(span.start, e))?;
            let definition = *visible
                .get(&name)
                .ok_or_else(|| error(span.start, "macro is not in textual scope"))?;
            report.invoke(
                source,
                &ast.captures[capture as usize],
                definition,
                data.span,
                &mut scratch,
            )?;
            continue;
        }
        if scope {
            work.push(Work::Restore(changes.len()));
        }
        children.clear();
        ast.push_children(node, &mut children);
        work.extend(children.iter().rev().copied().map(Work::Visit));
    }
    Ok(report)
}
fn image<'a>(
    source: &'a str,
    forest: &'a Forest,
    prepared: Option<&'a Prepared>,
) -> (&'a str, &'a Forest) {
    prepared.map_or((source, forest), |p| (&p.source, &p.forest))
}
fn original_error(mut error: LexError, prepared: Option<&Prepared>) -> LexError {
    if let Some(prepared) = prepared {
        error.offset = prepared.original_offset(error.offset);
    }
    error
}
impl Report {
    fn invoke(
        &mut self,
        source: &str,
        forest: &Forest,
        definition: usize,
        span: Span,
        scratch: &mut Workspace,
    ) -> Result<(), LexError> {
        let prepared = prepare::if_needed(source, forest)?;
        let (spelling, forest) = image(source, forest, prepared.as_ref());
        let range = forest
            .children(0)
            .ok_or_else(|| error(span.start, "macro invocation requires delimiters"))?;
        let input = Input::from_forest(forest, range)?;
        let entry = &mut self.definitions[definition];
        let matched = entry
            .compiled
            .invoke_with_scratch(source, spelling, &input, &mut self.names, scratch)
            .map_err(|e| original_error(e, prepared.as_ref()))?;
        let transcription =
            transcribe::transcribe(&entry.compiled.definition, &matched, spelling, &input)
                .map_err(|mut e| {
                    e.offset = span.start;
                    e
                })?;
        self.invocations.push(Invocation {
            span,
            definition,
            input,
            prepared,
            matched,
            transcription,
        });
        Ok(())
    }
}
