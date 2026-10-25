//! Bounded source-type text memoizes each typed node and charges every copied byte.
use super::{Contracts, Declarations, HashMap, Report};
use crate::types::{Kind, TypeId, arguments::Argument, callable::Convention};

pub(super) struct Formatter<'a> {
    facts: &'a Declarations,
    report: &'a Report,
    budget: &'a Contracts,
    values: HashMap<TypeId, String>,
}
impl<'a> Formatter<'a> {
    pub(super) fn new(facts: &'a Declarations, report: &'a Report, budget: &'a Contracts) -> Self {
        Self {
            facts,
            report,
            budget,
            values: HashMap::default(),
        }
    }
    pub(super) fn append(&self, text: &mut String, value: &str) -> Result<(), &'static str> {
        append(self.budget, text, value)
    }
    pub(super) fn ty(&mut self, root: TypeId) -> Result<&str, &'static str> {
        self.budget.charge(1, 4 * size_of::<(TypeId, bool)>())?;
        let mut pending = vec![(root, false)];
        while let Some((ty, complete)) = pending.pop() {
            self.budget.charge(1, 0)?;
            if self.values.contains_key(&ty) {
                continue;
            }
            if !complete {
                let children = self.children(ty)?;
                self.budget.charge(
                    children.len() + 1,
                    4 * (children.len() + 1) * size_of::<(TypeId, bool)>(),
                )?;
                pending.push((ty, true));
                pending.extend(children.into_iter().rev().map(|child| (child, false)));
                continue;
            }
            let value = self.compose(ty)?;
            self.budget.charge(1, 4 * size_of::<(TypeId, String)>())?;
            self.values.insert(ty, value);
        }
        self.values
            .get(&root)
            .map(String::as_str)
            .ok_or("diagnostic type formatter lost its root")
    }
    fn kind(&self, ty: TypeId) -> Result<Kind, &'static str> {
        self.facts
            .types
            .kind(ty)
            .ok_or("diagnostic type is unknown")
    }
    fn child(&self, ty: TypeId) -> Result<&str, &'static str> {
        self.values
            .get(&ty)
            .map(String::as_str)
            .ok_or("diagnostic child type is unresolved")
    }
    fn children(&self, ty: TypeId) -> Result<Vec<TypeId>, &'static str> {
        let mut children = Vec::new();
        let mut push = |child| -> Result<(), &'static str> {
            self.budget.charge(1, 4 * size_of::<TypeId>())?;
            children.push(child);
            Ok(())
        };
        match self.kind(ty)? {
            Kind::Tuple(list) => {
                for &child in self
                    .facts
                    .types
                    .list(list)
                    .ok_or("diagnostic tuple is unknown")?
                {
                    push(child)?;
                }
            }
            Kind::Array { element, .. } | Kind::Slice(element) => push(element)?,
            Kind::Reference { referent, .. } => push(referent)?,
            Kind::Pointer { pointee, .. } => push(pointee)?,
            Kind::Adt(_, args) => {
                for arg in self
                    .facts
                    .types
                    .arguments(args)
                    .ok_or("diagnostic nominal arguments are unknown")?
                {
                    let Argument::Type(child) = arg else {
                        return Err(
                            "diagnostic nominal lifetime/const formatting is not implemented",
                        );
                    };
                    push(child)?;
                }
            }
            Kind::Function {
                bound_regions: 0,
                parameters,
                result,
                convention: Convention::RUST,
                ..
            } => {
                for &child in self
                    .facts
                    .types
                    .list(parameters)
                    .ok_or("diagnostic function inputs are unknown")?
                {
                    push(child)?;
                }
                push(result)?;
            }
            Kind::FunctionItem { signature, .. } => push(signature)?,
            Kind::Primitive(_) | Kind::Never | Kind::Parameter(_) => {}
            _ => {
                return Err("diagnostic formatting for this typed source shape is not implemented");
            }
        }
        Ok(children)
    }
    fn item(&self, item: crate::types::ItemId) -> Result<&str, &'static str> {
        let definition = self
            .report
            .resolution
            .definitions
            .get(item.0 as usize)
            .ok_or("diagnostic declaration identity is unknown")?;
        if definition.kind != crate::resolve::model::DefKind::Generic && definition.scope.0 != 0 {
            return Err("diagnostic nested declaration path formatting is not implemented");
        }
        Ok(self.report.package.names.get(definition.name))
    }
    fn list(&self, text: &mut String, list: crate::types::ListId) -> Result<(), &'static str> {
        let children = self
            .facts
            .types
            .list(list)
            .ok_or("diagnostic type list is unknown")?;
        for (index, &child) in children.iter().enumerate() {
            if index != 0 {
                self.append(text, ", ")?;
            }
            self.append(text, self.child(child)?)?;
        }
        Ok(())
    }
    fn parameter_name(
        &self,
        id: crate::types::parameters::ParameterId,
    ) -> Result<&str, &'static str> {
        let parameter = self
            .facts
            .types
            .parameter(id)
            .ok_or("diagnostic generic identity is unknown")?;
        let parameters = self
            .facts
            .generics
            .parameters
            .get(&crate::resolve::model::DefId(parameter.owner.0))
            .ok_or("diagnostic generic declaration is not prepared")?;
        self.budget.charge(parameters.len(), 0)?;
        let binding = parameters
            .iter()
            .find_map(|parameter| match parameter.argument {
                Argument::Type(ty) if self.facts.types.kind(ty) == Some(Kind::Parameter(id)) => {
                    Some(parameter.binding)
                }
                _ => None,
            })
            .ok_or("diagnostic generic binding does not match its typed parameter")?;
        self.item(crate::types::ItemId(binding.0))
    }
    fn nominal(
        &self,
        text: &mut String,
        item: crate::types::ItemId,
        args: crate::types::GenericArgs,
    ) -> Result<(), &'static str> {
        self.append(text, self.item(item)?)?;
        let arguments = self
            .facts
            .types
            .arguments(args)
            .ok_or("diagnostic arguments are unknown")?;
        if arguments.len() == 0 {
            return Ok(());
        }
        self.append(text, "<")?;
        for (index, argument) in arguments.enumerate() {
            let Argument::Type(child) = argument else {
                return Err("diagnostic generic argument formatting is not implemented");
            };
            if index != 0 {
                self.append(text, ", ")?;
            }
            self.append(text, self.child(child)?)?;
        }
        self.append(text, ">")
    }
    fn tuple(&self, text: &mut String, list: crate::types::ListId) -> Result<(), &'static str> {
        self.append(text, "(")?;
        self.list(text, list)?;
        if self
            .facts
            .types
            .list(list)
            .is_some_and(|children| children.len() == 1)
        {
            self.append(text, ",")?;
        }
        self.append(text, ")")
    }
    fn array(&self, text: &mut String, element: TypeId, length: u64) -> Result<(), &'static str> {
        self.append(text, "[")?;
        self.append(text, self.child(element)?)?;
        self.budget.charge(1, 64)?;
        self.append(text, &format!("; {length}]"))
    }
    fn function(
        &self,
        text: &mut String,
        unsafe_: bool,
        parameters: crate::types::ListId,
        result: TypeId,
    ) -> Result<(), &'static str> {
        self.append(text, if unsafe_ { "unsafe fn(" } else { "fn(" })?;
        self.list(text, parameters)?;
        self.append(text, ")")?;
        if !matches!(self.kind(result)?, Kind::Tuple(list) if self.facts.types.list(list).is_some_and(<[_]>::is_empty))
        {
            self.append(text, " -> ")?;
            self.append(text, self.child(result)?)?;
        }
        Ok(())
    }
    fn compose(&self, ty: TypeId) -> Result<String, &'static str> {
        let mut text = String::new();
        match self.kind(ty)? {
            Kind::Never => self.append(&mut text, "!")?,
            Kind::Primitive(primitive) => self.append(&mut text, primitive.name())?,
            Kind::Parameter(id) => self.append(&mut text, self.parameter_name(id)?)?,
            Kind::Adt(item, args) => self.nominal(&mut text, item, args)?,
            Kind::Tuple(list) => self.tuple(&mut text, list)?,
            Kind::Array { element, length } => self.array(&mut text, element, length)?,
            Kind::Slice(element) => {
                self.append(&mut text, "[")?;
                self.append(&mut text, self.child(element)?)?;
                self.append(&mut text, "]")?;
            }
            Kind::Reference {
                mutable, referent, ..
            } => {
                self.append(&mut text, if mutable { "&mut " } else { "&" })?;
                self.append(&mut text, self.child(referent)?)?;
            }
            Kind::Pointer { mutable, pointee } => {
                self.append(&mut text, if mutable { "*mut " } else { "*const " })?;
                self.append(&mut text, self.child(pointee)?)?;
            }
            Kind::Function {
                unsafe_,
                parameters,
                result,
                ..
            } => self.function(&mut text, unsafe_, parameters, result)?,
            Kind::FunctionItem {
                item, signature, ..
            } => {
                self.append(&mut text, self.child(signature)?)?;
                self.append(&mut text, " {")?;
                self.append(&mut text, self.item(item)?)?;
                self.append(&mut text, "}")?;
            }
            _ => return Err("diagnostic formatter encountered an unsupported type"),
        }
        Ok(text)
    }
}
pub(super) fn append(
    budget: &Contracts,
    text: &mut String,
    value: &str,
) -> Result<(), &'static str> {
    budget.charge(1, value.len().saturating_mul(2))?;
    if text.len().saturating_add(value.len()) > 64 * 1024 {
        return Err("diagnostic formatted payload exceeds 64 KiB");
    }
    text.push_str(value);
    Ok(())
}
