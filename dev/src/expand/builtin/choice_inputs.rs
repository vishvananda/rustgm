//! Borrow actual opaque payloads and charge before their typed transfer or clone.
use crate::{
    lex::token::{LexError, Token},
    literal::storage::StoredValue,
    support::edition::Edition,
    syntax::{
        ast::Syntax,
        interpolation::{Inputs, Nonterminal, ParsedSyntax, Splice},
    },
};

pub(crate) type Charge<'a> = dyn Fn(usize, usize) -> Result<(), LexError> + 'a;
pub(crate) struct Counted<'a> {
    pub inputs: &'a dyn Inputs,
    pub charge: &'a Charge<'a>,
}
impl Inputs for Counted<'_> {
    fn is_empty(&self) -> bool {
        self.inputs.is_empty()
    }
    fn edition(&self, token: Token) -> Result<Edition, LexError> {
        (self.charge)(1, 0)?;
        self.inputs.edition(token)
    }
    fn get(&self, token: Token) -> Result<Option<Splice<'_>>, LexError> {
        (self.charge)(1, 0)?;
        let result = self.inputs.get(token)?;
        if let Some(splice) = result {
            match splice.fragment {
                Nonterminal::Metadata {
                    forest, expression, ..
                } => {
                    (self.charge)(
                        forest.nodes().len(),
                        forest.payload_bytes().saturating_mul(4),
                    )?;
                    if let Some(parsed) = expression {
                        syntax(parsed, self.charge)?;
                    }
                }
                Nonterminal::Syntax { parsed, .. }
                | Nonterminal::Visibility {
                    path: Some(parsed), ..
                } => syntax(parsed, self.charge)?,
                Nonterminal::Visibility { path: None, .. } => {}
            }
        }
        Ok(result)
    }
}
fn syntax(parsed: &ParsedSyntax, charge: &Charge<'_>) -> Result<(), LexError> {
    let ast = &parsed.ast;
    charge(ast.captures.len().saturating_add(ast.attributes.len()), 0)?;
    let bytes = syntax_payload(ast).saturating_add(parsed.literals.decoded_bytes());
    let work = ast
        .nodes
        .len()
        .saturating_add(ast.edges.len())
        .saturating_add(ast.literals.len());
    charge(work, bytes.saturating_mul(4))
}
fn syntax_payload(ast: &Syntax<StoredValue>) -> usize {
    use std::mem::size_of_val;
    let fixed = [
        size_of_val(ast.nodes.as_slice()),
        size_of_val(ast.edges.as_slice()),
        size_of_val(ast.items.as_slice()),
        size_of_val(ast.variants.as_slice()),
        size_of_val(ast.use_trees.as_slice()),
        size_of_val(ast.generics.as_slice()),
        size_of_val(ast.function_types.as_slice()),
        size_of_val(ast.attributes.as_slice()),
        size_of_val(ast.literals.as_slice()),
        size_of_val(ast.captures.as_slice()),
    ]
    .into_iter()
    .fold(0_usize, usize::saturating_add);
    ast.captures
        .iter()
        .map(crate::syntax::token_tree::Forest::payload_bytes)
        .chain(
            ast.attributes
                .iter()
                .map(|attribute| attribute.forest.payload_bytes()),
        )
        .fold(fixed, usize::saturating_add)
}
