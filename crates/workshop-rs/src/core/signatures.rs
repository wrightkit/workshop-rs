/// Supplies the expected enum domain for a call argument during parsing.
///
/// The parser asks for the expected domain of argument `arg_index` (0-based)
/// of the call whose Workshop catalog id is `catalog_id`. Implementations
/// must return the domain only when the canonical signature pins exactly one;
/// returning `None` leaves an ambiguous bare member unresolved.
pub trait ExpectedDomain {
    /// The expected enum domain for `arg_index` of the call with catalog id
    /// `catalog_id`, or `None` when the signature does not pin one.
    fn expected_domain(&self, catalog_id: &str, arg_index: usize) -> Option<&str>;
}

/// A context with no signature metadata. Ambiguous bare enum members remain
/// structured with their catalog candidates. Used by callers that intentionally
/// need context-free parsing.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoExpectedDomain;

impl ExpectedDomain for NoExpectedDomain {
    fn expected_domain(&self, _catalog_id: &str, _arg_index: usize) -> Option<&str> {
        None
    }
}

/// A context chain: consult `first`, then fall back to `second`.
///
/// Callers that combine signature sources (e.g. a provider manifest followed
/// by the canonical catalog) chain them; neither is authoritative alone.
#[derive(Clone, Copy)]
pub struct ChainedExpectedDomain<'a, 'b> {
    first: &'a dyn ExpectedDomain,
    second: &'b dyn ExpectedDomain,
}

impl<'a, 'b> ChainedExpectedDomain<'a, 'b> {
    /// Chain two contexts, consulting `first` before `second`.
    pub fn new(first: &'a dyn ExpectedDomain, second: &'b dyn ExpectedDomain) -> Self {
        ChainedExpectedDomain { first, second }
    }
}

impl ExpectedDomain for ChainedExpectedDomain<'_, '_> {
    fn expected_domain(&self, catalog_id: &str, arg_index: usize) -> Option<&str> {
        self.first
            .expected_domain(catalog_id, arg_index)
            .or_else(|| self.second.expected_domain(catalog_id, arg_index))
    }
}
