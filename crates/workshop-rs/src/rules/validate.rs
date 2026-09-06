//! Complete-program canonical validation orchestrated by the Workshop rule
//! domain.

use crate::catalog::Catalog;
use crate::core::error::Result;
use crate::wir;

/// Validate every builtin reference in a Workshop-origin WIR program against
/// the canonical catalog: action/value call names must be known canonical ids,
/// and event/enum references must resolve to canonical identities.
pub fn validate_canonical_ids(program: &wir::Program, catalog: &Catalog) -> Result<()> {
    let mut errors = Vec::new();
    for (index, _) in program.rules.iter().enumerate() {
        let rule = wir::RuleId::from_index(index);
        let Some(rule_data) = program.rules.get(rule) else {
            continue;
        };
        crate::events::validate::validate_event(
            &rule_data.event,
            rule_data.span,
            catalog,
            &mut errors,
        );
        for action in &rule_data.actions {
            crate::actions::validate::validate_action(program, catalog, *action, &mut errors);
        }
        for condition in &rule_data.conditions {
            crate::values::validate::validate_value(program, catalog, *condition, &mut errors);
        }
    }
    errors.into_iter().next().map_or(Ok(()), Err)
}
