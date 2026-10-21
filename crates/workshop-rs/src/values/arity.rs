use crate::catalog::CatalogEntry;

impl CatalogEntry {
    pub(crate) fn rejected_arity(&self, supplied: usize) -> Option<(usize, Option<usize>)> {
        if self.param_count() == 0 && self.required_param_count() == 0 {
            return None;
        }
        let min = self.required_param_count();
        let max = (!self.is_variadic()).then_some(self.param_count());
        (supplied < min || max.is_some_and(|max| supplied > max)).then_some((min, max))
    }
}
