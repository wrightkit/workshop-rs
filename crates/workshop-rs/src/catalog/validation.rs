//! Declared-fact validation for loaded catalog entries.
//!
//! These checks run at `Catalog::load` on the parsed entries: positional
//! arrays must not exceed the declared parameter list, enum-typed positions
//! must name a declared domain, `*Reeval` reevaluation coverage must review
//! every member of its domain against real parameter positions, and the
//! per-entry evaluation facts (`nondeterministic`, `perElementParams`,
//! `actionReevaluatedParams`) must sit on the entry kind they describe with
//! in-range, non-duplicated positions. A declared fact that contradicts the
//! signature is a malformed dataset, not a tolerable gap.

use std::collections::{BTreeMap, BTreeSet};

use crate::core::error::{CatalogError, Result};

use super::{Catalog, Kind};

impl Catalog {
    pub(crate) fn validate_param_domains(&self) -> Result<()> {
        for entry in &self.entries {
            for (name, count) in [
                ("parameter alias sets", entry.param_aliases.len()),
                ("param domains", entry.param_domains.len()),
                ("param defaults", entry.param_defaults.len()),
                ("param types", entry.param_types.len()),
                ("param coercions", entry.param_coercions.len()),
                ("param content facts", entry.param_content.len()),
            ] {
                if count > entry.params.len() {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' declares more {name} than params",
                        entry.kind.as_str(),
                        entry.id
                    )));
                }
            }
            if entry.kind != Kind::Value && entry.return_type.is_some() {
                return Err(CatalogError::validation(format!(
                    "{} '{}' declares a return type but is not a value",
                    entry.kind.as_str(),
                    entry.id
                )));
            }
            if entry.kind != Kind::Value && entry.nondeterministic.is_some() {
                return Err(CatalogError::validation(format!(
                    "{} '{}' declares nondeterministic but is not a value",
                    entry.kind.as_str(),
                    entry.id
                )));
            }
            if entry.kind != Kind::Value && entry.per_element_params.is_some() {
                return Err(CatalogError::validation(format!(
                    "{} '{}' declares per-element parameters but is not a value",
                    entry.kind.as_str(),
                    entry.id
                )));
            }
            if entry.kind != Kind::Action && entry.action_reevaluated_params.is_some() {
                return Err(CatalogError::validation(format!(
                    "{} '{}' declares action-reevaluated parameters but is not an action",
                    entry.kind.as_str(),
                    entry.id
                )));
            }
            for (name, positions) in [
                ("per-element", &entry.per_element_params),
                ("action-reevaluated", &entry.action_reevaluated_params),
            ] {
                if let Some(positions) = positions {
                    let mut seen = BTreeSet::new();
                    for position in positions {
                        if *position >= entry.params.len() {
                            return Err(CatalogError::validation(format!(
                                "{} '{}' declares {name} parameter {position}, out of {} params",
                                entry.kind.as_str(),
                                entry.id,
                                entry.params.len()
                            )));
                        }
                        if !seen.insert(position) {
                            return Err(CatalogError::validation(format!(
                                "{} '{}' declares {name} parameter {position} twice",
                                entry.kind.as_str(),
                                entry.id
                            )));
                        }
                    }
                }
            }
            for domain in entry.param_domains.iter().flatten() {
                if !self.enum_by_domain.contains_key(domain) {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' declares undeclared enum domain '{domain}'",
                        entry.kind.as_str(),
                        entry.id
                    )));
                }
            }
            if let Some(coverage) = &entry.reevaluation_coverage {
                let reeval_params: Vec<usize> = (0..entry.params.len())
                    .filter(|index| {
                        entry
                            .param_domains
                            .get(*index)
                            .and_then(Option::as_deref)
                            .is_some_and(|domain| domain.ends_with("Reeval"))
                    })
                    .collect();
                if reeval_params.len() != 1 {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' declares reevaluation coverage but has {} *Reeval parameters",
                        entry.kind.as_str(),
                        entry.id,
                        reeval_params.len()
                    )));
                }
                let domain_name = entry.param_domains[reeval_params[0]]
                    .as_deref()
                    .expect("filtered above");
                let domain = self
                    .enum_by_domain
                    .get(domain_name)
                    .map(|index| &self.enums[*index])
                    .expect("param domain validated above");
                for (member, positions) in coverage {
                    if !domain.members.iter().any(|m| m.member == *member) {
                        return Err(CatalogError::validation(format!(
                            "{} '{}' declares reevaluation coverage for '{member}', which is not a member of '{domain_name}'",
                            entry.kind.as_str(),
                            entry.id
                        )));
                    }
                    for position in positions {
                        if *position >= entry.params.len() {
                            return Err(CatalogError::validation(format!(
                                "{} '{}' reevaluation coverage for '{member}' names parameter {position}, out of {} params",
                                entry.kind.as_str(),
                                entry.id,
                                entry.params.len()
                            )));
                        }
                        if *position == reeval_params[0] {
                            return Err(CatalogError::validation(format!(
                                "{} '{}' reevaluation coverage for '{member}' covers the reevaluation parameter itself",
                                entry.kind.as_str(),
                                entry.id
                            )));
                        }
                    }
                }
                for member in &domain.members {
                    if !coverage.contains_key(&member.member) {
                        return Err(CatalogError::validation(format!(
                            "{} '{}' reevaluation coverage does not review '{domain_name}' member '{}'",
                            entry.kind.as_str(),
                            entry.id,
                            member.member
                        )));
                    }
                }
            }
            for aliases in &entry.param_aliases {
                for locale in aliases.keys() {
                    if !self.locales.contains(locale) {
                        return Err(CatalogError::validation(format!(
                            "{} '{}' declares parameter alias for undeclared locale '{}'",
                            entry.kind.as_str(),
                            entry.id,
                            locale
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_param_content(&self) -> Result<()> {
        for entry in &self.entries {
            let mut groups: BTreeMap<&str, Vec<(usize, usize)>> = BTreeMap::new();
            for (index, content) in entry.param_content.iter().enumerate() {
                let Some(content) = content else {
                    continue;
                };
                if content.fold.is_none() && content.operations.is_empty() {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' parameter {} declares an empty paramContent fact",
                        entry.kind.as_str(),
                        entry.id,
                        index + 1
                    )));
                }
                for operation in &content.operations {
                    if !self.by_id[Kind::Operator.as_index()].contains_key(operation) {
                        return Err(CatalogError::validation(format!(
                            "{} '{}' parameter {} accepts undeclared operator '{operation}'",
                            entry.kind.as_str(),
                            entry.id,
                            index + 1
                        )));
                    }
                }
                let Some(fold) = &content.fold else {
                    continue;
                };
                let (arity, variable_arg) = match fold.call.as_str() {
                    "globalVariable" => (1, 0),
                    "playerVariable" => (2, 1),
                    _ => {
                        return Err(CatalogError::validation(format!(
                            "{} '{}' parameter {} declares a fold into unknown \
                             call '{}'",
                            entry.kind.as_str(),
                            entry.id,
                            index + 1,
                            fold.call
                        )));
                    }
                };
                if fold.arg >= arity {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' parameter {} declares a fold into '{}' \
                         argument {}, out of {}",
                        entry.kind.as_str(),
                        entry.id,
                        index + 1,
                        fold.call,
                        fold.arg,
                        arity
                    )));
                }
                let variable_typed = matches!(
                    entry.param_type(index),
                    Some("Variable") | Some("Global Variable")
                );
                if fold.arg == variable_arg && !variable_typed {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' parameter {} supplies '{}' argument {}, the tagged \
                         variable node, but is not Variable-typed",
                        entry.kind.as_str(),
                        entry.id,
                        index + 1,
                        fold.call,
                        fold.arg
                    )));
                }
                groups
                    .entry(fold.call.as_str())
                    .or_default()
                    .push((index, fold.arg));
            }
            for (call, members) in groups {
                let arity = if call == "globalVariable" { 1 } else { 2 };
                let positions: Vec<usize> = members.iter().map(|(index, _)| *index).collect();
                let contiguous = positions.windows(2).all(|pair| pair[1] == pair[0] + 1);
                let mut args: Vec<usize> = members.iter().map(|(_, arg)| *arg).collect();
                args.sort_unstable();
                if !contiguous || args != (0..arity).collect::<Vec<_>>() {
                    return Err(CatalogError::validation(format!(
                        "{} '{}' declares an incomplete '{}' fold group: members must be \
                         contiguous parameters covering all {} argument(s)",
                        entry.kind.as_str(),
                        entry.id,
                        call,
                        arity
                    )));
                }
            }
        }
        Ok(())
    }
}
