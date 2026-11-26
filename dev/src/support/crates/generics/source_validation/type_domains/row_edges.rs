//! The shared type/application recipe graph used by shape and ownership validation.
use super::{Work, g, s};

#[derive(Clone, Copy)]
pub(super) struct Edges<'a> {
    rows: &'a s::SourceRows,
    declaration_policies: bool,
}

type Visit<'a> = &'a mut dyn FnMut(usize, &mut Work) -> Result<(), &'static str>;

impl<'a> Edges<'a> {
    pub(super) const fn new(rows: &'a s::SourceRows, declaration_policies: bool) -> Self {
        Self {
            rows,
            declaration_policies,
        }
    }

    pub(super) fn visit(
        self,
        index: usize,
        work: &mut Work,
        mut visit: impl FnMut(usize, &mut Work) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let Some(&ty) = self.rows.types.get(index) else {
            return self.application(index - self.rows.types.len(), work, &mut visit);
        };
        self.ty(ty, work, &mut visit)
    }

    fn ty(self, ty: s::Type, work: &mut Work, visit: Visit<'_>) -> Result<(), &'static str> {
        let rows = self.rows;
        match ty {
            s::Type::Nominal { arguments, .. } => self.arguments(arguments, work, visit),
            s::Type::Dynamic(reference) => {
                if !self.declaration_policies {
                    return Err("source trait object requires version fourteen");
                }
                let object = rows
                    .cold()
                    .dynamics
                    .objects
                    .get(reference.0 as usize)
                    .ok_or("source trait object has no dynamic recipe")?;
                self.arguments(object.arguments, work, visit)
            }
            s::Type::Projection(reference) => self.projection(reference, work, visit),
            s::Type::OpaqueFuture(reference) => {
                let row = rows
                    .opaque_futures
                    .get(reference.0 as usize)
                    .ok_or("unknown source opaque future recipe")?;
                self.child(row.output, work, visit)?;
                self.arguments(row.captures, work, visit)
            }
            s::Type::OpaqueResult(reference) => {
                let row = rows
                    .cold()
                    .opaques
                    .applications
                    .get(reference.0 as usize)
                    .ok_or("unknown source opaque result recipe")?;
                self.arguments(row.captures, work, visit)
            }
            s::Type::Closure(reference) => {
                let row = rows
                    .closures()
                    .and_then(|closures| closures.types.get(reference.0 as usize))
                    .ok_or("unknown source closure type recipe")?;
                self.child(row.environment, work, visit)?;
                self.child(row.signature, work, visit)?;
                self.arguments(row.arguments, work, visit)
            }
            s::Type::Reference {
                referent: child, ..
            }
            | s::Type::RawPointer { pointee: child, .. }
            | s::Type::Slice(child)
            | s::Type::Pattern { base: child, .. } => self.child(child, work, visit),
            s::Type::Array { element, length } => {
                self.term(rows.array_lengths().get(length.0 as usize), work, visit)?;
                self.child(element, work, visit)
            }
            s::Type::Tuple(elements) => self.type_range(elements, work, visit),
            s::Type::Function(signature) => {
                self.child(signature.result, work, visit)?;
                self.type_range(signature.parameters, work, visit)
            }
            s::Type::FunctionItem(application) => {
                if application.0 as usize >= rows.function_applications.len() {
                    return Err("source function item crosses its application table namespace");
                }
                visit(rows.types.len() + application.0 as usize, work)
            }
            s::Type::ApplicationSignature(_) => {
                Err("source application signature recipes require a complete wider profile")
            }
            s::Type::Scalar(_)
            | s::Type::Foreign(_)
            | s::Type::F16
            | s::Type::F128
            | s::Type::Never
            | s::Type::Parameter(_)
            | s::Type::Str => Ok(()),
        }
    }

    fn projection(
        self,
        reference: s::ProjectionRef,
        work: &mut Work,
        visit: Visit<'_>,
    ) -> Result<(), &'static str> {
        let rows = self.rows;
        let row = rows
            .projections
            .get(reference.0 as usize)
            .ok_or("unknown source projection recipe")?;
        self.child(row.subject, work, visit)?;
        let application = rows
            .trait_applications
            .get(row.trait_application.0 as usize)
            .ok_or("unknown projection trait application")?;
        for range in [application.arguments, row.arguments] {
            for argument in range.slice(&rows.arguments)?.iter().rev() {
                if let s::Argument::Type(ty) = *argument {
                    self.child(ty, work, visit)?;
                }
            }
        }
        Ok(())
    }

    fn application(
        self,
        offset: usize,
        work: &mut Work,
        visit: Visit<'_>,
    ) -> Result<(), &'static str> {
        let rows = self.rows;
        let arguments = if let Some(application) = rows.function_applications.get(offset) {
            application.arguments
        } else {
            let application = rows
                .trait_applications
                .get(offset - rows.function_applications.len())
                .ok_or("source trait application is unknown")?;
            self.child(application.subject, work, visit)?;
            application.arguments
        };
        work.charge(arguments.slice(&rows.arguments)?.len())?;
        self.arguments(arguments, work, visit)
    }

    fn type_range(
        self,
        range: g::Range,
        work: &mut Work,
        visit: Visit<'_>,
    ) -> Result<(), &'static str> {
        for &ty in range.slice(&self.rows.roots)?.iter().rev() {
            self.child(ty, work, visit)?;
        }
        Ok(())
    }

    fn arguments(
        self,
        range: g::Range,
        work: &mut Work,
        visit: Visit<'_>,
    ) -> Result<(), &'static str> {
        for argument in range.slice(&self.rows.arguments)?.iter().rev() {
            match *argument {
                s::Argument::Type(ty) => self.child(ty, work, visit)?,
                s::Argument::Const(term) => {
                    self.term(self.rows.consts().get(term.0 as usize), work, visit)?;
                }
                s::Argument::Region(_) => {}
            }
        }
        Ok(())
    }

    fn term(
        self,
        term: Option<&s::ConstTerm>,
        work: &mut Work,
        visit: Visit<'_>,
    ) -> Result<(), &'static str> {
        match term {
            Some(&s::ConstTerm::Value(value)) => self.child(value.ty, work, visit),
            Some(&s::ConstTerm::Variant(value)) => self.child(value.ty, work, visit),
            _ => Ok(()),
        }
    }

    fn child(self, ty: g::TypeRef, work: &mut Work, visit: Visit<'_>) -> Result<(), &'static str> {
        work.charge(1)?;
        let index = ty.0 as usize;
        if index >= self.rows.types.len() {
            return Err("source type reference crosses its typed table namespace");
        }
        visit(index, work)
    }
}
