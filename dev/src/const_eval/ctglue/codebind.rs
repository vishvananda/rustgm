//! A cleanup footer binds once, including when an earlier Code has expired.
use super::{Bundle, Declarations, Method, Rc};
use crate::{
    body::{
        drops::glue,
        ir::{Owner, Place},
    },
    const_eval::code::Code,
};
impl Bundle {
    pub(crate) fn require_unbound(&self) -> Result<(), &'static str> {
        let binding = self
            .binding
            .try_borrow()
            .map_err(|_| "cleanup ownership is being published")?;
        if binding.is_some() {
            return Err("cleanup footer is already bound to an actual Code");
        }
        Ok(())
    }
    pub(crate) fn bind(&self, code: &Rc<Code>) -> Result<(), &'static str> {
        self.header(code)?;
        self.view.validate()?;
        for &method in &self.view.calls {
            self.place(code, method)?;
        }
        let mut binding = self
            .binding
            .try_borrow_mut()
            .map_err(|_| "cleanup ownership is being published")?;
        if binding.is_some() {
            return Err("cleanup footer cannot be rebound or attached to another Code");
        }
        *binding = Some(Rc::downgrade(code));
        Ok(())
    }
    fn header(&self, code: &Code) -> Result<(), &'static str> {
        if !matches!(
            code.owner,
            Owner::Function {
                execution: None,
                ..
            }
        ) || code.owner.key(code.source) != self.key
            || code.source != self.source
            || code.instance.is_some()
            || !code.const_context
            || code.result != self.result
            || code.owner.signatures() != self.signatures
            || self.contract.key != self.key
            || self.contract.source != self.source
            || self.contract.signature != self.signatures.map(|(_, owned)| owned)
        {
            return Err("cleanup footer changed its exact source, owner, instance or signatures");
        }
        Ok(())
    }
    fn place(&self, code: &Code, method: Method) -> Result<Place, &'static str> {
        let Place::Local(local) = method.place else {
            return Err("constant nonlocal cleanup execution is not implemented");
        };
        let recipe = self
            .view
            .recipes
            .get(method.recipe)
            .ok_or("cleanup lost its actual recipe")?;
        if code
            .locals
            .get(local.0 as usize)
            .is_none_or(|actual| actual.ty != method.target)
            || recipe.ty != method.target
            || !matches!(recipe.kind, glue::Shape::Fields(range) if range.start == range.end)
        {
            return Err("reached cleanup changed its complete local target or recipe");
        }
        Ok(method.place)
    }
    pub(super) fn require_code(&self, code: &Code) -> Result<(), &'static str> {
        self.header(code)?;
        let binding = self
            .binding
            .try_borrow()
            .map_err(|_| "cleanup ownership is being published")?;
        let actual = binding
            .as_ref()
            .and_then(std::rc::Weak::upgrade)
            .ok_or("reached cleanup lost its actual live Code owner")?;
        if !std::ptr::eq(actual.as_ref(), code) {
            return Err("reached cleanup belongs to another actual Code");
        }
        Ok(())
    }
    pub(super) fn reached(
        &self,
        facts: &Declarations,
        code: &Code,
        event: usize,
        work: &mut crate::body::defining_cleanups::Work,
    ) -> Result<(Method, Place), &'static str> {
        work.charge(
            // Includes the following independent owning-frame guard lookup.
            64 + self
                .view
                .calls
                .len()
                .checked_ilog2()
                .map_or(0, |n| 2 * (n as usize + 1)),
        )?;
        self.header(code)?;
        if !facts.types.has_identity(&self.types)
            || !facts.aggregates.has_identity(&self.definitions)
        {
            return Err("reached cleanup belongs to another current arena");
        }
        self.require_code(code)?;
        let crate::body::drops::schedule::Effect::Cleanup(index) = self
            .view
            .events
            .get(event)
            .ok_or("invalid reached cleanup cursor")?
            .effect
        else {
            return Err("reached cleanup cursor is not an actual cleanup event");
        };
        let method = self
            .view
            .call(index)?
            .ok_or("reached cleanup has no actual selected method")?;
        self.state()?
            .require_execution_target(&facts.types, method.target, work)?;
        require_method(facts, method)?;
        let place = self.place(code, method)?;
        Ok((method, place))
    }
}
fn require_method(facts: &Declarations, selected: Method) -> Result<(), &'static str> {
    let Some(crate::types::Kind::Adt(item, crate::types::GenericArgs::EMPTY)) =
        facts.types.kind(selected.target)
    else {
        return Err("reached cleanup lost its full closed Struct target");
    };
    let implementation = facts
        .destructors
        .item(item)
        .ok_or("reached cleanup has no actual Drop implementation")?;
    let definition = crate::sema::destructors::cleanup_method(
        facts,
        implementation.method,
        implementation.self_definition,
    )?;
    if facts
        .definitions
        .get(definition.0 as usize)
        .copied()
        .flatten()
        != Some(selected.instance.ty())
        || !facts.is_const_function(definition)
        || facts.const_default_member(selected.instance)?.is_some()
        || facts.recorded_caller_tracking(definition)?
    {
        return Err("reached cleanup changed its actual written const Drop method");
    }
    facts.check_const_member_execution(definition)?;
    facts
        .source_bindings
        .as_deref()
        .ok_or("reached cleanup lost its source catalog")?
        .check_const_call(definition)
}
