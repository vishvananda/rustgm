//! A bounded NFA frontier diagnoses local ambiguity before invoking a fragment parser.
use super::{
    capture::{self, CaptureId, Captures},
    input::Input,
    model::{Atom, Definition, Rule, VariableId},
    nfa::{self, Action, Instruction, Program, StateId},
};
use crate::{
    lex::token::{Kind, LexError, Token},
    support::names::Names,
    syntax::{
        cursor::TokenPosition,
        interpolation::{Inputs, Slice, Splice},
        token_tree::error,
    },
};
pub const NO_TRACE: u32 = u32::MAX;
#[derive(Clone, Copy, Debug)]
pub enum Event {
    Action(Action),
    Capture {
        variable: VariableId,
        value: CaptureId,
    },
}
#[derive(Clone, Copy, Debug)]
pub struct Trace {
    pub parent: u32,
    pub event: Event,
}
pub struct Matched {
    pub rule: usize,
    pub captures: Captures,
    pub traces: Vec<Trace>,
    pub last_trace: u32,
    pub work: usize,
}
pub struct Compiled {
    pub definition: Definition,
    programs: Vec<Option<Program>>,
}
impl Compiled {
    #[must_use]
    pub fn new(definition: Definition) -> Self {
        let programs = (0..definition.rules.len()).map(|_| None).collect();
        Self {
            definition,
            programs,
        }
    }
    /// # Errors
    /// Reports no matching rule, local ambiguity, parser failures and work limits.
    pub fn invoke(
        &mut self,
        definition_source: &str,
        source: &str,
        input: &Input,
        names: &mut Names,
    ) -> Result<Matched, LexError> {
        self.invoke_with_scratch(
            definition_source,
            source,
            input,
            names,
            &mut Workspace::default(),
        )
    }
    /// Reuse the NFA frontier across a session's invocations and rule alternatives.
    ///
    /// # Errors
    /// Reports the same language and work-limit errors as `invoke`.
    pub fn invoke_with_scratch(
        &mut self,
        definition_source: &str,
        source: &str,
        input: &Input,
        names: &mut Names,
        scratch: &mut Workspace,
    ) -> Result<Matched, LexError> {
        self.invoke_with_splices(definition_source, source, input, names, scratch, &[])
    }
    /// # Errors
    /// Also rejects incompatible or missing forwarded fragment identities.
    pub fn invoke_with_splices(
        &mut self,
        definition_source: &str,
        source: &str,
        input: &Input,
        names: &mut Names,
        scratch: &mut Workspace,
        splices: &[Splice<'_>],
    ) -> Result<Matched, LexError> {
        self.invoke_with_inputs(
            definition_source,
            source,
            input,
            names,
            scratch,
            &Slice(splices),
        )
    }
    /// # Errors
    /// Reports matching failures or invalid persistent capture identities.
    pub fn invoke_with_inputs(
        &mut self,
        definition_source: &str,
        source: &str,
        input: &Input,
        names: &mut Names,
        scratch: &mut Workspace,
        inputs: &dyn Inputs,
    ) -> Result<Matched, LexError> {
        let mut captures = Captures::default();
        let mut invocation = Invocation {
            source,
            input,
            names,
            captures: &mut captures,
            inputs,
            work: 0,
            budget: 1_000_000_usize.saturating_add(input.tokens.len().saturating_mul(128)),
        };
        for (index, rule) in self.definition.rules.iter().enumerate() {
            if self.programs[index].is_none() {
                self.programs[index] = Some(nfa::build(&self.definition, rule)?);
            }
            let program = self.programs[index]
                .as_ref()
                .ok_or_else(|| error(0, "missing compiled matcher"))?;
            let mut machine =
                Machine::new(program, rule, definition_source, &mut invocation, scratch);
            if let Some(last_trace) = machine.run()? {
                let traces = std::mem::take(&mut machine.scratch.traces);
                let work = machine.invocation.work;
                captures.finish();
                return Ok(Matched {
                    rule: index,
                    captures,
                    traces,
                    last_trace,
                    work,
                });
            }
        }
        Err(error(
            input.tokens.first().map_or(0, |t| t.start),
            "no macro rule matches this invocation",
        ))
    }
}
struct Invocation<'a> {
    source: &'a str,
    input: &'a Input,
    names: &'a mut Names,
    captures: &'a mut Captures,
    inputs: &'a dyn Inputs,
    work: usize,
    budget: usize,
}
#[derive(Clone, Copy)]
struct Thread {
    state: StateId,
    trace: u32,
}
#[derive(Clone, Copy)]
struct FragmentThread {
    variable: VariableId,
    next: StateId,
    trace: u32,
}
struct Machine<'a, 'b> {
    program: &'a Program,
    rule: &'a Rule,
    definition_source: &'a str,
    invocation: &'a mut Invocation<'b>,
    scratch: &'a mut Workspace,
}
#[derive(Default)]
pub struct Workspace {
    active: Vec<Thread>,
    stack: Vec<Thread>,
    ready: Vec<Thread>,
    literal: Vec<Thread>,
    fragments: Vec<FragmentThread>,
    accepted: Vec<u32>,
    traces: Vec<Trace>,
    generation: u32,
    seen: Vec<u32>,
    counts: Vec<u8>,
    seen_traces: Vec<[u32; 2]>,
}
impl<'a, 'b> Machine<'a, 'b> {
    fn new(
        program: &'a Program,
        rule: &'a Rule,
        definition_source: &'a str,
        invocation: &'a mut Invocation<'b>,
        scratch: &'a mut Workspace,
    ) -> Self {
        let states = program.states.len();
        scratch.active.clear();
        scratch.stack.clear();
        scratch.traces.clear();
        scratch.seen.resize(states, 0);
        scratch.counts.resize(states, 0);
        scratch.seen_traces.resize(states, [0; 2]);
        scratch.active.push(Thread {
            state: program.start,
            trace: NO_TRACE,
        });
        Self {
            program,
            rule,
            definition_source,
            invocation,
            scratch,
        }
    }
    fn tick(&mut self, offset: u32) -> Result<(), LexError> {
        if self.invocation.work >= self.invocation.budget {
            return Err(error(offset, "macro matcher work budget exceeded"));
        }
        self.invocation.work += 1;
        Ok(())
    }
    fn trace(&mut self, parent: u32, event: Event) -> Result<u32, LexError> {
        let id = u32::try_from(self.scratch.traces.len())
            .map_err(|_| error(0, "too many macro capture events"))?;
        self.scratch.traces.push(Trace { parent, event });
        Ok(id)
    }
    fn seen(&mut self, thread: Thread) -> bool {
        let index = thread.state.0 as usize;
        if self.scratch.seen[index] != self.scratch.generation {
            self.scratch.seen[index] = self.scratch.generation;
            self.scratch.counts[index] = 0;
        }
        let count = usize::from(self.scratch.counts[index]);
        if self.scratch.seen_traces[index][..count].contains(&thread.trace) || count == 2 {
            return true;
        }
        self.scratch.seen_traces[index][count] = thread.trace;
        self.scratch.counts[index] += 1;
        false
    }
    fn closure(&mut self, offset: u32) -> Result<(), LexError> {
        self.scratch.generation = self.scratch.generation.wrapping_add(1);
        if self.scratch.generation == 0 {
            self.scratch.seen.fill(0);
            self.scratch.generation = 1;
        }
        self.scratch
            .stack
            .extend(self.scratch.active.drain(..).rev());
        self.scratch.ready.clear();
        while let Some(thread) = self.scratch.stack.pop() {
            self.tick(offset)?;
            if self.seen(thread) {
                continue;
            }
            match self.program.states[thread.state.0 as usize] {
                Instruction::Split { first, second } => {
                    self.scratch.stack.push(Thread {
                        state: second,
                        ..thread
                    });
                    self.scratch.stack.push(Thread {
                        state: first,
                        ..thread
                    });
                }
                Instruction::Action { action, next } => {
                    let trace = self.trace(thread.trace, Event::Action(action))?;
                    self.scratch.stack.push(Thread { state: next, trace });
                }
                _ => self.scratch.ready.push(thread),
            }
        }
        Ok(())
    }
    fn candidates(&mut self, token: Option<Token>, offset: u32) -> Result<(), LexError> {
        self.scratch.literal.clear();
        self.scratch.fragments.clear();
        self.scratch.accepted.clear();
        for index in 0..self.scratch.ready.len() {
            self.tick(offset)?;
            let thread = self.scratch.ready[index];
            match self.program.states[thread.state.0 as usize] {
                Instruction::Accept => self.scratch.accepted.push(thread.trace),
                // Source-written tokens cannot carry a definition-site crate identity.
                Instruction::Crate { .. } | Instruction::Doc { .. } => {}
                Instruction::Atom { atom, next } => {
                    if let Some(token) = token
                        && atom_matches(
                            atom,
                            token,
                            self.definition_source,
                            self.invocation.source,
                            self.invocation.names,
                        )?
                    {
                        self.scratch.literal.push(Thread {
                            state: next,
                            ..thread
                        });
                    }
                }
                Instruction::Delimiter { punctuation, next } => {
                    if token.is_some_and(|t| t.kind == Kind::Punctuation(punctuation)) {
                        self.scratch.literal.push(Thread {
                            state: next,
                            ..thread
                        });
                    }
                }
                Instruction::Fragment { variable, next } => {
                    let fragment = self.rule.variables[variable.0 as usize].fragment;
                    if capture::may_begin_with_inputs(
                        fragment,
                        token,
                        self.invocation.source,
                        self.invocation.inputs,
                    )? {
                        self.scratch.fragments.push(FragmentThread {
                            variable,
                            next,
                            trace: thread.trace,
                        });
                    }
                }
                Instruction::Split { .. } | Instruction::Action { .. } => {
                    return Err(error(offset, "unexpanded matcher epsilon state"));
                }
            }
        }
        Ok(())
    }
    fn run(&mut self) -> Result<Option<u32>, LexError> {
        let mut position = TokenPosition::default();
        loop {
            let token = self
                .invocation
                .input
                .token(self.invocation.source, position)?;
            let offset = token.map_or_else(
                || self.invocation.input.tokens.last().map_or(0, |t| t.end),
                |t| t.start,
            );
            self.closure(offset)?;
            self.candidates(token, offset)?;
            if self.scratch.fragments.len() > 1
                || (!self.scratch.fragments.is_empty()
                    && (!self.scratch.literal.is_empty()
                        || (token.is_none() && !self.scratch.accepted.is_empty())))
            {
                return Err(error(offset, "local ambiguity in macro matcher"));
            }
            if token.is_none() && !self.scratch.accepted.is_empty() {
                if self.scratch.accepted.len() != 1 {
                    return Err(error(offset, "multiple successful macro matcher parses"));
                }
                return Ok(Some(self.scratch.accepted[0]));
            }
            if let Some(thread) = self.scratch.fragments.first().copied() {
                let fragment = self.rule.variables[thread.variable.0 as usize].fragment;
                let value = self.invocation.captures.demand_with_inputs(
                    self.invocation.source,
                    self.invocation.input,
                    fragment,
                    position,
                    self.invocation.inputs,
                )?;
                position = self.invocation.captures.values[value.0 as usize].end;
                let trace = self.trace(
                    thread.trace,
                    Event::Capture {
                        variable: thread.variable,
                        value,
                    },
                )?;
                self.scratch.active.push(Thread {
                    state: thread.next,
                    trace,
                });
            } else if !self.scratch.literal.is_empty() {
                position = self
                    .invocation
                    .input
                    .advance(self.invocation.source, position)?;
                self.scratch.active.append(&mut self.scratch.literal);
            } else {
                return Ok(None);
            }
        }
    }
}
fn atom_matches(
    atom: Atom,
    token: Token,
    definition_source: &str,
    source: &str,
    names: &mut Names,
) -> Result<bool, LexError> {
    if atom.token.kind != token.kind {
        return Ok(false);
    }
    let spelling = source
        .get(token.start as usize..token.end as usize)
        .ok_or_else(|| error(token.start, "invalid macro token span"))?;
    if let Some(name) = atom.name {
        return names
            .intern(spelling)
            .map(|other| name == other)
            .map_err(|e| error(token.start, e));
    }
    let expected = definition_source
        .get(atom.token.start as usize..atom.token.end as usize)
        .ok_or_else(|| error(atom.token.start, "invalid matcher token span"))?;
    Ok(expected == spelling)
}
