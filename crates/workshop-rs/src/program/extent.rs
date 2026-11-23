use super::identity::{NodeIdentity, condition_identity, value_identity};
use super::{Program, Value, ValueProvenance, action_argument_values, value_children};

impl Program {
    /// Return the authored `If`, `Else If`, `Else`, or `End` keyword span for
    /// an action in linear rule order, or `None` without matching provenance.
    pub fn action_marker_span(&self, rule: usize, action: usize) -> Option<crate::source::Span> {
        self.action_provenance(rule, action)?.marker_span
    }

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

fn value_provenance_at<'a>(
    record: &'a ValueProvenance,
    value: &'a Value,
    path: &[usize],
    root_identity: NodeIdentity,
) -> Option<&'a ValueProvenance> {
    let mut record = record;
    let mut value = value;
    let mut identity = root_identity;
    for &index in path {
        let children = value_children(value);
        if record.children.len() != children.len() {
            return None;
        }
        record = record.children.get(index)?;
        value = children[index];
        identity = value_identity(value);
    }
    (record.identity == identity).then_some(record)
}
