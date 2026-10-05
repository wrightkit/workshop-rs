//! Complete-program canonical validation orchestrated by the Workshop rule
//! domain.

use crate::catalog::Catalog;
use crate::core::error::Result;
use crate::wir;

/// Validate every builtin reference in a Workshop-origin WIR program against
/// the canonical catalog: action/value call names must be known canonical ids,
/// and event/enum references must resolve to canonical identities.
///
/// Calls must supply every declared parameter that carries no catalog default;
/// a trailing parameter with a declared default (for example `Wait`'s
/// `waitBehavior`) is optional and may be omitted. Variable names are bound at
/// parse time: an undeclared name allocates an implicit variable slot rather
/// than failing validation.
pub fn validate_canonical_ids(program: &crate::Program, catalog: &Catalog) -> Result<()> {
    let storage = program.to_wir()?;
    validate_wir(&storage, catalog, false)
}

/// Same contract as [`validate_canonical_ids`], except that catalog-unknown
/// action and value calls — the constructs [`crate::Program::semantic_issues`]
/// reports as completeness residuals — are tolerated rather than rejected.
/// An unknown call's signature is unknowable and is not checked, but its
/// argument values are still validated recursively and validation continues
/// to later siblings, so a residual cannot mask a canonical violation
/// elsewhere in the program. Every other canonical check is unchanged and
/// the first real violation still aborts validation.
///
/// Consumers that report residuals separately (through `semantic_issues`)
/// use this entry point so that completeness handling does not weaken
/// canonical validation; producer artifacts should still use the strict
/// [`validate_canonical_ids`].
pub fn validate_canonical_ids_tolerating_residuals(
    program: &crate::Program,
    catalog: &Catalog,
) -> Result<()> {
    let storage = program.to_wir()?;
    validate_wir(&storage, catalog, true)
}

#[cfg(test)]
pub(crate) fn validate_canonical_ids_wir(program: &wir::Program, catalog: &Catalog) -> Result<()> {
    validate_wir(program, catalog, false)
}

pub(crate) fn validate_wir(
    program: &wir::Program,
    catalog: &Catalog,
    tolerate_residuals: bool,
) -> Result<()> {
    for (index, _) in program.rules.iter().enumerate() {
        let rule = wir::RuleId::from_index(index);
        let Some(rule_data) = program.rules.get(rule) else {
            continue;
        };
        crate::events::validate::validate_event(&rule_data.event, rule_data.span, catalog)?;
        for action in &rule_data.actions {
            crate::actions::validate::validate_action(
                program,
                catalog,
                *action,
                tolerate_residuals,
            )?;
        }
        for condition in &rule_data.conditions {
            crate::values::validate::validate_value(
                program,
                catalog,
                condition.value,
                tolerate_residuals,
            )?;
        }
    }
    Ok(())
}
