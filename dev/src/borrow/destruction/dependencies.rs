//! Only selected cleanup roots demand method code; merely retained recipes do not.
use super::{Operation, Plan, glue};

enum Work {
    Node(super::NodeId),
    Recipe(glue::Id),
}
#[derive(Default)]
pub(crate) struct Dependencies {
    recipes: Vec<bool>,
    steps: usize,
}
impl Dependencies {
    pub(crate) fn collect(
        &mut self,
        recipes: &glue::Recipes,
        plan: &Plan,
    ) -> Result<Vec<glue::Id>, &'static str> {
        let mut work = Vec::new();
        for cleanup in &plan.cleanups {
            self.charge(1)?;
            if let Some(node) = cleanup.node {
                work.push(Work::Node(node));
            }
        }
        self.run(recipes, Some(plan), work)
    }
    pub(crate) fn exposed(
        &mut self,
        recipes: &glue::Recipes,
        roots: impl Iterator<Item = glue::Id>,
    ) -> Result<Vec<glue::Id>, &'static str> {
        self.run(recipes, None, roots.map(Work::Recipe).collect())
    }
    fn run(
        &mut self,
        recipes: &glue::Recipes,
        plan: Option<&Plan>,
        mut work: Vec<Work>,
    ) -> Result<Vec<glue::Id>, &'static str> {
        self.recipes.resize(recipes.recipes().len(), false);
        let mut seen = vec![false; plan.map_or(0, |plan| plan.nodes.len())];
        let mut methods = Vec::new();
        while let Some(next) = work.pop() {
            self.charge(1)?;
            match next {
                Work::Node(id) => {
                    let plan = plan.ok_or("cleanup dependency lacks its selected plan")?;
                    let visited = seen
                        .get_mut(id.0 as usize)
                        .ok_or("invalid cleanup dependency node")?;
                    if std::mem::replace(visited, true) {
                        continue;
                    }
                    let node = plan
                        .nodes
                        .get(id.0 as usize)
                        .ok_or("missing cleanup dependency node")?;
                    self.node(plan, node.operation, &mut work)?;
                }
                Work::Recipe(id) => {
                    let visited = self
                        .recipes
                        .get_mut(id.0 as usize)
                        .ok_or("invalid cleanup dependency recipe")?;
                    if std::mem::replace(visited, true) {
                        continue;
                    }
                    let recipe = recipes.get(id).ok_or("missing cleanup dependency recipe")?;
                    if recipe.method.is_some() {
                        methods.push(id);
                    }
                    match recipe.kind {
                        glue::Shape::Array { element, length } if length != 0 => {
                            work.push(Work::Recipe(element));
                        }
                        glue::Shape::Slice { element } => work.push(Work::Recipe(element)),
                        glue::Shape::Fields(fields) => self.fields(recipes, fields, &mut work)?,
                        glue::Shape::Variants(variants) => {
                            for variant in recipes
                                .variants(variants)
                                .ok_or("invalid cleanup dependency variants")?
                                .iter()
                                .rev()
                            {
                                self.charge(1)?;
                                self.fields(recipes, variant.fields, &mut work)?;
                            }
                        }
                        glue::Shape::Array { .. } | glue::Shape::Dynamic => {}
                    }
                }
            }
        }
        Ok(methods)
    }
    fn node(
        &mut self,
        plan: &Plan,
        operation: Operation,
        work: &mut Vec<Work>,
    ) -> Result<(), &'static str> {
        match operation {
            Operation::Empty => {}
            Operation::Glue(id) => work.push(Work::Recipe(id)),
            Operation::Fields(range) => {
                for field in plan
                    .fields
                    .get(range.start as usize..range.end as usize)
                    .ok_or("invalid selected cleanup fields")?
                    .iter()
                    .rev()
                {
                    self.charge(1)?;
                    work.push(Work::Node(field.node));
                }
            }
            Operation::Variants(range) => {
                for variant in plan
                    .variants
                    .get(range.start as usize..range.end as usize)
                    .ok_or("invalid selected cleanup variants")?
                    .iter()
                    .rev()
                {
                    self.charge(1)?;
                    work.push(Work::Node(variant.node));
                }
            }
            Operation::Array(range) => {
                for interval in plan
                    .intervals
                    .get(range.start as usize..range.end as usize)
                    .ok_or("invalid selected cleanup intervals")?
                    .iter()
                    .rev()
                {
                    self.charge(1)?;
                    if interval.start < interval.end {
                        work.push(Work::Node(interval.element));
                    }
                }
            }
        }
        Ok(())
    }
    fn fields(
        &mut self,
        recipes: &glue::Recipes,
        range: glue::Range,
        work: &mut Vec<Work>,
    ) -> Result<(), &'static str> {
        for field in recipes
            .fields(range)
            .ok_or("invalid cleanup dependency fields")?
            .iter()
            .rev()
        {
            self.charge(1)?;
            work.push(Work::Recipe(field.recipe));
        }
        Ok(())
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|n| *n <= 1_000_000)
            .ok_or("destructor dependency work budget exceeded")?;
        Ok(())
    }
}
