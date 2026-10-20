//! The argument-count rule shared by canonical validation and
//! `rule-content-v1` statement conversion.

use crate::catalog::CatalogEntry;

impl CatalogEntry {
    /// The argument-count range this entry accepts for `supplied` arguments
    /// (a folded player-variable pair counted as two), or `None` when
    /// `supplied` is accepted. The range is `(min, max)` with `max` unset
    /// for a variadic signature.
    ///
    /// An empty signature means the arity is not declared, not that the
    /// builtin takes no arguments (documented variadic calls such as Custom
    /// String), so every count is accepted.
    pub(crate) fn rejected_arity(&self, supplied: usize) -> Option<(usize, Option<usize>)> {
        if self.param_count() == 0 && self.required_param_count() == 0 {
            return None;
        }
        let min = self.required_param_count();
        let max = (!self.is_variadic()).then_some(self.param_count());
        (supplied < min || max.is_some_and(|max| supplied > max)).then_some((min, max))
    }
}
