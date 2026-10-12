//! Thompson-style control graph: repetition choices do not recursively backtrack.
use super::{
    input::delimiters,
    model::{Atom, Definition, Kind, List, NodeId, Repetition, Rule, VariableId},
};
use crate::{
    lex::token::{LexError, Punctuation},
    syntax::token_tree::error,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StateId(pub u32);
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Begin(NodeId),
    Iteration(NodeId),
    End(NodeId),
}
#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    Accept,
    Crate {
        next: StateId,
    },
    Doc {
        next: StateId,
    },
    Atom {
        atom: Atom,
        next: StateId,
    },
    Delimiter {
        punctuation: Punctuation,
        next: StateId,
    },
    Fragment {
        variable: VariableId,
        next: StateId,
    },
    Split {
        first: StateId,
        second: StateId,
    },
    Action {
        action: Action,
        next: StateId,
    },
}
pub struct Program {
    pub states: Vec<Instruction>,
    pub start: StateId,
}
/// # Errors
/// Rejects state-count overflow or invalid manually supplied matcher nodes.
pub fn build(definition: &Definition, rule: &Rule) -> Result<Program, LexError> {
    let mut program = Program {
        states: vec![Instruction::Accept],
        start: StateId(0),
    };
    program.start = program.sequence(definition, rule.matcher, StateId(0))?;
    Ok(program)
}
impl Program {
    fn state(&mut self, instruction: Instruction) -> Result<StateId, LexError> {
        let id = StateId(
            u32::try_from(self.states.len())
                .map_err(|_| error(0, "too many macro matcher states"))?,
        );
        self.states.push(instruction);
        Ok(id)
    }
    fn sequence(
        &mut self,
        definition: &Definition,
        list: List,
        mut next: StateId,
    ) -> Result<StateId, LexError> {
        for node in definition.list(list).iter().rev() {
            next = self.node(definition, *node, next)?;
        }
        Ok(next)
    }
    fn node(
        &mut self,
        definition: &Definition,
        id: NodeId,
        next: StateId,
    ) -> Result<StateId, LexError> {
        let node = definition.node(id);
        match node.kind {
            Kind::Crate => self.state(Instruction::Crate { next }),
            Kind::Doc { .. } => self.state(Instruction::Doc { next }),
            Kind::Atom(atom) => self.state(Instruction::Atom { atom, next }),
            Kind::Capture(variable) => self.state(Instruction::Fragment { variable, next }),
            Kind::Group {
                delimiter,
                children,
            } => {
                let (open, close) = delimiters(delimiter);
                let close = self.state(Instruction::Delimiter {
                    punctuation: close,
                    next,
                })?;
                let body = self.sequence(definition, children, close)?;
                self.state(Instruction::Delimiter {
                    punctuation: open,
                    next: body,
                })
            }
            Kind::Repeat {
                children,
                separator,
                operator,
            } => {
                let end = self.state(Instruction::Action {
                    action: Action::End(id),
                    next,
                })?;
                let repeated = if operator == Repetition::Optional {
                    end
                } else {
                    self.state(Instruction::Split {
                        first: end,
                        second: end,
                    })?
                };
                let body = self.sequence(definition, children, repeated)?;
                let iteration = self.state(Instruction::Action {
                    action: Action::Iteration(id),
                    next: body,
                })?;
                if operator != Repetition::Optional {
                    let again = if let Some(atom) = separator {
                        self.state(Instruction::Atom {
                            atom,
                            next: iteration,
                        })?
                    } else {
                        iteration
                    };
                    self.states[repeated.0 as usize] = Instruction::Split {
                        first: again,
                        second: end,
                    };
                }
                let first = if operator == Repetition::OneOrMore {
                    iteration
                } else {
                    self.state(Instruction::Split {
                        first: iteration,
                        second: end,
                    })?
                };
                self.state(Instruction::Action {
                    action: Action::Begin(id),
                    next: first,
                })
            }
            Kind::Substitute { .. } => {
                Err(error(node.span.start, "substitution in a macro matcher"))
            }
        }
    }
}
