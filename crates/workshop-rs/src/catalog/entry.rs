//! One catalog builtin entry and its declared per-parameter and per-entry
//! facts.
//!
//! The structs here are the loaded form of an `actions`/`values`/`structural`
//! catalog object: the signature surface (`params`, `paramNames`,
//! `paramAliases`, `paramDomains`, `paramDefaults`, `paramTypes`,
//! `paramCoercions`, `paramContent`), the reviewed evaluation facts
//! (`reevaluationCoverage`, `nondeterministic`, `perElementParams`,
//! `actionReevaluatedParams`), and the localized spellings that resolve the
//! entry. Unknown or unreviewed facts stay `None` so consumers can tell them
//! apart from a reviewed negative.

use std::collections::{BTreeMap, HashMap};

use super::{Kind, Locale, ParamCoercions, ParamContent};

/// One catalog builtin.
#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub id: String,
    pub kind: Kind,
    pub(crate) callable: bool,
    pub(crate) params: Vec<String>,
    pub(crate) param_names: Option<Vec<String>>,
    pub(crate) param_aliases: Vec<HashMap<Locale, Vec<String>>>,
    pub(crate) param_display_names: Vec<Option<HashMap<Locale, String>>>,
    pub(crate) param_domains: Vec<Option<String>>,
    pub(crate) param_defaults: Vec<Option<String>>,
    pub(crate) param_types: Vec<Option<String>>,
    pub(crate) param_coercions: Vec<Option<ParamCoercions>>,
    pub(crate) param_content: Vec<Option<ParamContent>>,
    pub(crate) return_type: Option<String>,
    pub(crate) reevaluation_coverage: Option<BTreeMap<String, Vec<usize>>>,
    pub(crate) nondeterministic: Option<bool>,
    pub(crate) per_element_params: Option<Vec<usize>>,
    pub(crate) action_reevaluated_params: Option<Vec<usize>>,
    pub(crate) variadic: bool,
    pub(crate) aliases: HashMap<Locale, Vec<String>>,
}

/// A locale-independent identity for a preset used by the Workshop `String`
/// value. Unlike a custom `Value::String`, this identity must resolve through
/// reviewed client-locale aliases before it can be parsed or emitted.
#[derive(Debug, Clone)]
pub struct LocalizedStringEntry {
    pub id: String,
    pub(crate) aliases: HashMap<Locale, Vec<String>>,
}

pub(crate) fn spelling<'a>(
    aliases: &'a HashMap<Locale, Vec<String>>,
    locale: &Locale,
) -> Option<&'a str> {
    aliases
        .get(locale)
        .and_then(|spellings| spellings.first())
        .map(String::as_str)
}

pub(crate) fn spellings_for<'a>(
    aliases: &'a HashMap<Locale, Vec<String>>,
    locale: &Locale,
) -> &'a [String] {
    aliases.get(locale).map(Vec::as_slice).unwrap_or_default()
}

impl LocalizedStringEntry {
    /// The deterministic emitted spelling in `locale`, when mapped.
    pub fn spelling(&self, locale: &Locale) -> Option<&str> {
        spelling(&self.aliases, locale)
    }

    /// All reviewed spellings accepted for this locale.
    pub fn spellings(&self, locale: &Locale) -> &[String] {
        spellings_for(&self.aliases, locale)
    }
}

impl CatalogEntry {
    /// The localized spelling of this builtin in `locale`, when declared.
    pub fn spelling(&self, locale: &Locale) -> Option<&str> {
        spelling(&self.aliases, locale)
    }

    /// Every reviewed localized spelling of this builtin, with the first
    /// spelling reserved for deterministic emission.
    pub fn spellings(&self, locale: &Locale) -> &[String] {
        spellings_for(&self.aliases, locale)
    }

    /// Resolve a canonical or reviewed localized parameter spelling to its
    /// unambiguous declared position.
    pub fn resolve_param(&self, locale: &Locale, spelling: &str) -> Option<usize> {
        let mut matches = self
            .params
            .iter()
            .enumerate()
            .filter(|(index, canonical)| {
                canonical == &spelling
                    || self
                        .param_aliases
                        .get(*index)
                        .and_then(|aliases| aliases.get(locale))
                        .is_some_and(|aliases| aliases.iter().any(|alias| alias == spelling))
            })
            .map(|(index, _)| index);
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    }

    /// The canonical parameter names in declaration order.
    pub fn params(&self) -> &[String] {
        &self.params
    }

    /// The number of declared arguments for this builtin.
    pub fn param_count(&self) -> usize {
        self.params.len()
    }

    /// The reviewed semantic name for an argument position, when declared.
    pub fn param_name(&self, index: usize) -> Option<&str> {
        let names = self.param_names.as_deref().unwrap_or(&self.params);
        param_at(names, index, self.variadic).map(String::as_str)
    }

    /// The localized parameter display name the Workshop editor shows for
    /// the argument at `index` in `locale`, when the catalog records one.
    /// This is a reviewed editor-presentation fact only — it does not
    /// affect parsing or emission — and it stays separate from
    /// [`param_name`], `paramAliases`, and `paramDefaults`.
    ///
    /// [`param_name`]: Self::param_name
    pub fn param_display_name(&self, locale: &Locale, index: usize) -> Option<&str> {
        param_at(&self.param_display_names, index, self.variadic)
            .and_then(Option::as_ref)
            .and_then(|names| names.get(locale))
            .map(String::as_str)
    }

    /// The number of arguments that must be present when trailing defaults
    /// are applied. A missing default in the middle of a signature remains a
    /// required position; defaults only make the suffix optional.
    pub fn required_param_count(&self) -> usize {
        (0..self.params.len())
            .rev()
            .find(|index| {
                self.param_defaults
                    .get(*index)
                    .and_then(Option::as_ref)
                    .is_none()
            })
            .map_or(0, |index| index + 1)
    }

    /// Whether any declared parameter has a default value.
    pub fn has_param_defaults(&self) -> bool {
        self.param_defaults.iter().any(Option::is_some)
    }

    /// The default value for an argument position, when declared.
    pub fn param_default(&self, index: usize) -> Option<&str> {
        param_at(&self.param_defaults, index, self.variadic).and_then(Option::as_deref)
    }

    /// The declared default's enum-member identity `(domain, member)`, when
    /// the catalog records the default as a member of a declared enum domain
    /// (`Team.ALL`, `Wait.IGNORE_CONDITION`). `None` means the position has
    /// no enum-member default — a non-enum default (`0`, `null`, a value
    /// call, a preset string) or no declared default — and does not
    /// distinguish the two; `param_default` still carries the written form.
    pub fn param_default_enum_member(&self, index: usize) -> Option<(&str, &str)> {
        let default = self.param_default(index)?;
        let domain = self.param_domain(index)?;
        let (prefix, member) = default.split_once('.')?;
        (prefix == domain).then_some((domain, member))
    }

    /// The declared enum domain for an argument position, when one exists.
    pub fn param_domain(&self, index: usize) -> Option<&str> {
        param_at(&self.param_domains, index, self.variadic).and_then(Option::as_deref)
    }

    /// The parameter positions the given `*Reeval` enum member keeps
    /// re-evaluating on this action, when the action declares reviewed
    /// reevaluation coverage. `Some(&[])` means the member re-evaluates no
    /// parameter of this action; `None` means the member's coverage is
    /// unknown for this action and callers must stay conservative.
    ///
    /// `member` is the canonical enum member id (e.g. `COLOR`), the same
    /// spelling a parsed `Value::Enum` carries.
    pub fn reevaluation_coverage(&self, member: &str) -> Option<&[usize]> {
        self.reevaluation_coverage
            .as_ref()
            .and_then(|coverage| coverage.get(member).map(Vec::as_slice))
    }

    /// Whether this action declares reviewed per-parameter reevaluation
    /// coverage for its `*Reeval` parameter.
    pub fn has_reevaluation_coverage(&self) -> bool {
        self.reevaluation_coverage.is_some()
    }

    /// Whether this value's result is not a function of its arguments
    /// alone: `Some(true)` marks a reviewed nondeterministic value
    /// (randomness, elapsed time, server state), `Some(false)` a reviewed
    /// deterministic one, and `None` an unreviewed entry — callers must
    /// stay conservative, consistent with `reevaluation_coverage`
    /// returning `None` for unknown coverage.
    ///
    /// The fact is only reviewed for value entries; an action reports
    /// `None`.
    pub fn nondeterministic(&self) -> Option<bool> {
        self.nondeterministic
    }

    /// The parameter positions this value evaluates once per array element
    /// with `Current Array Element` / `Current Array Index` rebound to the
    /// element under consideration. `Some(&[])` marks a value reviewed as
    /// having no per-element positions; `None` is unreviewed, so callers
    /// must stay conservative.
    ///
    /// The fact is only reviewed for value entries.
    pub fn per_element_params(&self) -> Option<&[usize]> {
        self.per_element_params.as_deref()
    }

    /// The parameter positions this action itself re-evaluates while it
    /// runs (`waitUntil[0]`, `loopIf[0]`), independent of the `*Reeval`
    /// enum-parameter coverage `reevaluation_coverage` reports.
    /// `Some(&[])` marks an action reviewed as re-evaluating no argument
    /// itself; `None` is unreviewed, so callers must stay conservative.
    ///
    /// The fact is only reviewed for action entries.
    pub fn action_reevaluated_params(&self) -> Option<&[usize]> {
        self.action_reevaluated_params.as_deref()
    }

    /// The source-backed semantic type for an argument position, when
    /// available. Enum domains remain exposed separately by `param_domain`.
    pub fn param_type(&self, index: usize) -> Option<&str> {
        param_at(&self.param_types, index, self.variadic).and_then(Option::as_deref)
    }

    /// The contextual literal substitutions for an argument position.
    pub fn param_coercions(&self, index: usize) -> Option<&ParamCoercions> {
        param_at(&self.param_coercions, index, self.variadic).and_then(Option::as_ref)
    }

    /// The `rule-content-v1` facts for an argument position: how the
    /// parameter appears in the structured document when the surface
    /// signature and the content layout differ.
    pub fn param_content(&self, index: usize) -> Option<&ParamContent> {
        param_at(&self.param_content, index, self.variadic).and_then(Option::as_ref)
    }

    /// The source-backed return type of a Value, when available.
    pub fn return_type(&self) -> Option<&str> {
        self.return_type.as_deref()
    }

    /// Whether the final declared parameter repeats for additional arguments.
    pub fn is_variadic(&self) -> bool {
        self.variadic
    }

    pub(crate) fn folds_player_variable_pair(&self) -> bool {
        self.param_type(0) == Some("Player|Array") && self.param_type(1) == Some("Variable")
    }

    pub(crate) fn wir_args_folded(
        &self,
        program: &crate::wir::Program,
        args: &[crate::wir::ValueId],
    ) -> bool {
        self.folds_player_variable_pair()
            && matches!(
                args.first()
                    .and_then(|arg| program.values.get(*arg))
                    .map(|node| &node.value),
                Some(crate::wir::Value::PlayerVariable { .. })
            )
    }

    pub(crate) fn wir_arg_param_index(
        &self,
        program: &crate::wir::Program,
        args: &[crate::wir::ValueId],
        wir_index: usize,
    ) -> usize {
        wir_index + usize::from(self.wir_args_folded(program, args))
    }
}

pub(crate) fn param_at<T>(values: &[T], index: usize, variadic: bool) -> Option<&T> {
    values
        .get(index)
        .or_else(|| variadic.then(|| values.last()).flatten())
}
