//! Authored-extent accessors for values nested in conditions and action
//! arguments. See `docs/source-preservation.md`.

use super::identity::{condition_identity, value_identity};
use super::{Program, ValueProvenance, action_argument_values, value_provenance_at};

impl Program {
    pub(super) fn condition_value_record(
        &self,
        rule: usize,
        condition: usize,
        path: &[usize],
    ) -> Option<&ValueProvenance> {
        let record = self.condition_record(rule, condition)?;
        let value = &self.rules[rule].conditions[condition].value;
        let identity = condition_identity(&self.rules[rule].conditions[condition]);
        value_provenance_at(record, value, path, identity)
    }

    pub(super) fn action_argument_value_record(
        &self,
        rule: usize,
        action: usize,
        argument: usize,
        path: &[usize],
    ) -> Option<&ValueProvenance> {
        let record = self.argument_record(rule, action, argument)?;
        let value = action_argument_values(&self.rules[rule].actions[action])[argument];
        value_provenance_at(record, value, path, value_identity(value))
    }

    /// Return the authored extent of a value nested inside a public rule
    /// condition: the span from its first token through its last token,
    /// covering infix operands and the closing `)` or `]` of calls, arrays,
    /// and vectors. `path` walks the [`Value`](crate::Value) tree exactly as
    /// [`condition_value_span`](Program::condition_value_span) describes.
    ///
    /// Unlike `condition_value_span`, which returns the node's recorded
    /// span — for a variable or subroutine reference its identifier — the
    /// extent always covers the whole authored sub-expression text. Raw
    /// parses record it; a position without recorded provenance returns
    /// `None`, never a neighboring or enclosing span.
    pub fn condition_value_extent(
        &self,
        rule: usize,
        condition: usize,
        path: &[usize],
    ) -> Option<crate::source::Span> {
        let record = self.condition_value_record(rule, condition, path)?;
        record.extent.or(record.span)
    }

    /// Return the authored extent of a value nested inside a direct value
    /// argument of a public action: the span from its first token through
    /// its last token, covering infix operands and the closing `)` or `]`
    /// of calls, arrays, and vectors. `argument` and `path` address the
    /// value exactly as
    /// [`action_argument_value_span`](Program::action_argument_value_span)
    /// describes.
    ///
    /// Unlike `action_argument_value_span`, the extent always covers the
    /// whole authored sub-expression text. Raw parses record it; a position
    /// without recorded provenance returns `None`, never a neighboring or
    /// enclosing span.
    pub fn action_argument_value_extent(
        &self,
        rule: usize,
        action: usize,
        argument: usize,
        path: &[usize],
    ) -> Option<crate::source::Span> {
        let record = self.action_argument_value_record(rule, action, argument, path)?;
        record.extent.or(record.span)
    }
}
