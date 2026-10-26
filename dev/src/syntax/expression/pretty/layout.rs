//! Bounded box/break layout, following the pinned pretty-printer's Oppen model.
//!
//! Rust's implementation attributes the model to Philip Karlton and Derek Oppen,
//! "Pretty Printing" (1979), STAN-CS-79-770. We compute selected document widths
//! in one paid pass rather than retaining a second compiler syntax graph.
use crate::{lex::token::LexError, syntax::token_tree::error};
const MARGIN: isize = 78;
const MIN_SPACE: isize = 60;
const MAX_GROUPS: usize = 4096;
#[derive(Clone, Copy)]
pub(super) enum Token<'a> {
    Text(&'a str),
    Break {
        blank: usize,
        offset: i16,
        hard: bool,
        comma: bool,
    },
    Begin {
        indent: u16,
        consistent: bool,
    },
    End,
}
#[derive(Clone, Copy)]
enum Width {
    Finite(usize),
    Infinite,
}
impl Width {
    fn fits(self, remaining: isize) -> bool {
        matches!(self, Self::Finite(bytes) if usize::try_from(remaining).is_ok_and(|remaining| bytes <= remaining))
    }
}
#[derive(Clone, Copy, Default)]
struct Prefix {
    bytes: usize,
    lines: usize,
}
impl Prefix {
    fn width(self, earlier: Self) -> Width {
        if self.lines == earlier.lines {
            Width::Finite(self.bytes.saturating_sub(earlier.bytes))
        } else {
            Width::Infinite
        }
    }
}
struct Group {
    begin: usize,
    start: Prefix,
    last_break: Option<(usize, Prefix)>,
}
#[derive(Clone, Copy)]
enum Frame {
    Flat,
    Broken { indent: usize, consistent: bool },
}
type Charge<'a> = dyn Fn(usize, usize) -> Result<(), LexError> + 'a;
fn widths(tokens: &[Token<'_>], charge: &Charge<'_>) -> Result<Vec<Width>, LexError> {
    charge(
        tokens.len().saturating_add(1),
        tokens.len().saturating_mul(size_of::<Width>()),
    )?;
    let mut sizes = Vec::new();
    sizes
        .try_reserve_exact(tokens.len())
        .map_err(|_| error(0, "expression layout width allocation failed"))?;
    sizes.resize(tokens.len(), Width::Infinite);
    let mut groups: Vec<Group> = Vec::new();
    let mut total = Prefix::default();
    for (index, token) in tokens.iter().copied().enumerate() {
        charge(1, 0)?;
        match token {
            Token::Text(text) => {
                total.bytes = total
                    .bytes
                    .checked_add(text.len())
                    .ok_or_else(|| error(0, "expression layout width overflow"))?;
            }
            Token::Begin { .. } => {
                charge(1, 4 * size_of::<Group>())?;
                if groups.len() >= MAX_GROUPS {
                    return Err(error(0, "expression layout group budget exceeded"));
                }
                groups
                    .try_reserve(1)
                    .map_err(|_| error(0, "expression layout group allocation failed"))?;
                groups.push(Group {
                    begin: index,
                    start: total,
                    last_break: None,
                });
            }
            Token::Break { blank, hard, .. } => {
                let group = groups
                    .last_mut()
                    .ok_or_else(|| error(0, "expression layout break has no enclosing box"))?;
                if let Some((previous, start)) = group.last_break {
                    sizes[previous] = total.width(start);
                }
                group.last_break = Some((index, total));
                if hard {
                    total.lines = total
                        .lines
                        .checked_add(1)
                        .ok_or_else(|| error(0, "expression layout line overflow"))?;
                } else {
                    total.bytes = total
                        .bytes
                        .checked_add(blank)
                        .ok_or_else(|| error(0, "expression layout blank overflow"))?;
                }
            }
            Token::End => {
                let group = groups
                    .pop()
                    .ok_or_else(|| error(0, "expression layout box underflow"))?;
                if let Some((previous, start)) = group.last_break {
                    sizes[previous] = total.width(start);
                }
                sizes[group.begin] = total.width(group.start);
            }
        }
    }
    if !groups.is_empty() {
        return Err(error(0, "expression layout has an unclosed box"));
    }
    Ok(sizes)
}
struct Output {
    text: String,
    remaining: isize,
    pending: usize,
}
impl Output {
    fn text(&mut self, value: &str, charge: &Charge<'_>) -> Result<(), LexError> {
        let reserve = self
            .pending
            .checked_add(value.len())
            .ok_or_else(|| error(0, "expression layout output overflow"))?;
        charge(reserve.saturating_add(1), reserve.saturating_mul(4))?;
        self.text
            .try_reserve(reserve)
            .map_err(|_| error(0, "expression layout output allocation failed"))?;
        self.text.extend(std::iter::repeat_n(' ', self.pending));
        self.pending = 0;
        self.text.push_str(value);
        let width = isize::try_from(value.len())
            .map_err(|_| error(0, "expression layout word width overflow"))?;
        self.remaining = self
            .remaining
            .checked_sub(width)
            .ok_or_else(|| error(0, "expression layout column overflow"))?;
        Ok(())
    }
    fn newline(&mut self, indent: usize, charge: &Charge<'_>) -> Result<(), LexError> {
        charge(1, 4)?;
        self.text
            .try_reserve(1)
            .map_err(|_| error(0, "expression layout newline allocation failed"))?;
        self.text.push('\n');
        let width =
            isize::try_from(indent).map_err(|_| error(0, "expression layout indent overflow"))?;
        self.remaining = MARGIN
            .checked_sub(width)
            .ok_or_else(|| error(0, "expression layout margin overflow"))?
            .max(MIN_SPACE);
        self.pending = indent;
        Ok(())
    }
}
pub(super) fn render(tokens: &[Token<'_>], charge: &Charge<'_>) -> Result<String, LexError> {
    let sizes = widths(tokens, charge)?;
    let mut frames = Vec::new();
    let mut indentation = 0usize;
    let mut output = Output {
        text: String::new(),
        remaining: MARGIN,
        pending: 0,
    };
    for (index, token) in tokens.iter().copied().enumerate() {
        charge(1, 0)?;
        match token {
            Token::Text(text) => output.text(text, charge)?,
            Token::Begin { indent, consistent } => {
                charge(1, 4 * size_of::<Frame>())?;
                if frames.len() >= MAX_GROUPS {
                    return Err(error(0, "expression layout print depth exceeded"));
                }
                frames
                    .try_reserve(1)
                    .map_err(|_| error(0, "expression layout frame allocation failed"))?;
                let frame = if sizes[index].fits(output.remaining) {
                    Frame::Flat
                } else {
                    let previous = indentation;
                    indentation = indentation
                        .checked_add(usize::from(indent))
                        .ok_or_else(|| error(0, "expression box indentation overflow"))?;
                    Frame::Broken {
                        indent: previous,
                        consistent,
                    }
                };
                frames.push(frame);
            }
            Token::End => {
                let frame = frames
                    .pop()
                    .ok_or_else(|| error(0, "expression layout print underflow"))?;
                if let Frame::Broken { indent, .. } = frame {
                    indentation = indent;
                }
            }
            Token::Break {
                blank,
                offset,
                hard,
                comma,
            } => {
                let frame = *frames
                    .last()
                    .ok_or_else(|| error(0, "expression layout print break has no box"))?;
                let next_line = match frame {
                    Frame::Flat => false,
                    Frame::Broken { consistent, .. } => {
                        hard || consistent || !sizes[index].fits(output.remaining)
                    }
                };
                if next_line {
                    if comma {
                        output.text(",", charge)?;
                    }
                    let indent = indentation
                        .checked_add_signed(isize::from(offset))
                        .ok_or_else(|| error(0, "expression layout indentation underflow"))?;
                    output.newline(indent, charge)?;
                } else {
                    output.pending = output
                        .pending
                        .checked_add(blank)
                        .ok_or_else(|| error(0, "expression layout pending blanks overflow"))?;
                    let width = isize::try_from(blank)
                        .map_err(|_| error(0, "expression layout blank width overflow"))?;
                    output.remaining = output
                        .remaining
                        .checked_sub(width)
                        .ok_or_else(|| error(0, "expression layout column overflow"))?;
                }
            }
        }
    }
    if !frames.is_empty() {
        return Err(error(0, "expression layout print boxes remain open"));
    }
    Ok(output.text)
}
