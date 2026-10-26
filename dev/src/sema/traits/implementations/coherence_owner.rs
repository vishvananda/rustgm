//! Only one graph can own this registry's pending and sealed specialization work.
use super::Implementations;
use crate::sema::specialization::Graph;
impl Implementations {
    pub(in crate::sema) fn specialization_graph(&self) -> Option<&Graph> {
        self.graph.as_deref()
    }
    pub(in crate::sema) fn specialization_graph_mut(&mut self) -> Option<&mut Graph> {
        self.graph.as_deref_mut()
    }
    pub(in crate::sema) fn install_specialization_graph(
        &mut self,
        graph: Box<Graph>,
    ) -> Result<(), &'static str> {
        if self.graph.is_some() {
            return Err("specialization graph cannot be replaced during registry lifetime");
        }
        self.graph = Some(graph);
        Ok(())
    }
}
