//! Deterministic localized Workshop emitter.
//!
//! Serializes validated Workshop IR into localized Workshop text with a
//! selectable output locale. Canonical catalog identities resolve to
//! locale-specific spellings; missing target-locale mappings fail explicitly
//! with a [`WorkshopError::MissingMapping`] diagnostic — never a guess, never
//! a silent passthrough of another locale's spelling. Fallback to another
//! declared locale is opt-in ([`EmitOptions`]) and every fell-back identity
//! is recorded in [`EmitOutput::fallback_ids`]. The formatting is fixed and
//! presentation-canonical, so the same WIR/config emits byte-stable text that
//! reparses to equivalent WIR — except for the `settings` section:
//! settings-bearing emissions are deliberately rejected by the Workshop
//! parser (a `.ws` decompiler is a non-goal). Settings names are resolved from
//! the generated locale corpus, with an explicit `en-US` fallback when needed.

pub(crate) use std::fmt::Write;

pub(crate) use crate::catalog::{Catalog, Kind, Locale};
pub(crate) use crate::core::error::{Result, WorkshopError};
pub(crate) use crate::core::format::format_number;
pub(crate) use crate::settings::table::{self, KeyKind, PathPart};
pub(crate) use crate::settings::{Settings as SettingsTree, SettingsNode};
pub(crate) use crate::wir;

/// Emission options: opt-in fallback for missing target-locale mappings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EmitOptions {
    /// When a canonical identity has no spelling for the target locale, its
    /// spelling in this declared locale is used instead. `None` (the default)
    /// keeps missing mappings failing explicitly. The fallback choice is
    /// visible in [`EmitOutput::fallback_ids`].
    pub fallback_locale: Option<Locale>,
}

/// The result of a localized emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitOutput {
    /// The emitted localized Workshop text.
    pub text: String,
    /// Canonical identities (and the `settings` marker) whose spelling came
    /// from the opt-in fallback locale instead of the target locale. Empty
    /// when no fallback occurred.
    pub fallback_ids: Vec<String>,
}

/// Emit a Workshop IR program as localized Workshop text, failing explicitly
/// on any missing target-locale mapping (no fallback).
pub fn emit(program: &wir::Program, catalog: &Catalog, locale: &Locale) -> Result<String> {
    emit_with_options(program, catalog, locale, &EmitOptions::default()).map(|out| out.text)
}

/// Emit a Workshop IR program as localized Workshop text with emission
/// options (opt-in fallback locale).
pub fn emit_with_options(
    program: &wir::Program,
    catalog: &Catalog,
    locale: &Locale,
    options: &EmitOptions,
) -> Result<EmitOutput> {
    emit_with_options_inner(program, catalog, locale, options, false)
}

pub(crate) fn emit_with_options_for_conversion(
    program: &wir::Program,
    catalog: &Catalog,
    locale: &Locale,
    options: &EmitOptions,
) -> Result<EmitOutput> {
    emit_with_options_inner(program, catalog, locale, options, true)
}

fn emit_with_options_inner(
    program: &wir::Program,
    catalog: &Catalog,
    locale: &Locale,
    options: &EmitOptions,
    force_hero_constructors: bool,
) -> Result<EmitOutput> {
    let mut emitter = EmitContext {
        program,
        catalog,
        locale: locale.clone(),
        fallback: options.fallback_locale.clone(),
        force_hero_constructors,
        fallback_ids: Vec::new(),
        out: String::new(),
        line_count: 0,
    };
    emitter.run()?;
    Ok(EmitOutput {
        text: emitter.out,
        fallback_ids: emitter.fallback_ids,
    })
}

pub(crate) struct EmitContext<'a> {
    pub(crate) program: &'a wir::Program,
    pub(crate) catalog: &'a Catalog,
    pub(crate) locale: Locale,
    /// The opt-in fallback locale for missing target-locale mappings.
    pub(crate) fallback: Option<Locale>,
    /// Canonical ids emitted with a fallback-locale spelling.
    pub(crate) fallback_ids: Vec<String>,
    pub(crate) force_hero_constructors: bool,
    pub(crate) out: String,
    pub(crate) line_count: usize,
}

impl EmitContext<'_> {
    pub(crate) fn run(&mut self) -> Result<()> {
        // Section order: settings, variables, subroutines, rules.
        if let Some(settings) = &self.program.settings {
            self.emit_settings(settings)?;
            self.out.push('\n');
        }
        if !self.program.global_variables.is_empty() || !self.program.player_variables.is_empty() {
            let variables = self.structural("variables")?;
            self.line(0, &format!("{variables} {{"))?;
            if !self.program.global_variables.is_empty() {
                let global = self.structural("global")?;
                self.line(1, &format!("{global}:"))?;
                for variable in self.program.global_variables.iter() {
                    self.line(2, &format!("{}: {}", variable.index, variable.name))?;
                }
            }
            if !self.program.player_variables.is_empty() {
                let player = self.structural("player")?;
                self.line(1, &format!("{player}:"))?;
                for variable in self.program.player_variables.iter() {
                    self.line(2, &format!("{}: {}", variable.index, variable.name))?;
                }
            }
            self.line(0, "}")?;
            self.out.push('\n');
        }
        if !self.program.subroutines.is_empty() {
            let subroutines = self.structural("subroutines")?;
            self.line(0, &format!("{subroutines} {{"))?;
            for subroutine in self.program.subroutines.iter() {
                self.line(1, &format!("{}: {}", subroutine.index, subroutine.name))?;
            }
            self.line(0, "}")?;
            self.out.push('\n');
        }
        for (emitted_rules, rule) in self.program.rules.iter().enumerate() {
            if emitted_rules > 0 {
                self.out.push('\n');
            }
            self.rule(rule)?;
        }
        // The oracle's raw artifact ends with a trailing blank line (the
        // committed snapshots strip it via the acquisition normalizer; the
        // pinned oracle's own output keeps it).
        if !self.out.is_empty() && !self.out.ends_with("\n\n") {
            self.out.push('\n');
        }
        Ok(())
    }

    pub(crate) fn malformed(&self, message: impl Into<String>) -> WorkshopError {
        WorkshopError::Malformed {
            message: message.into(),
            span: None,
        }
    }

    pub(crate) fn line(&mut self, level: usize, text: &str) -> Result<()> {
        for _ in 0..level {
            self.out.push_str("    ");
        }
        self.out.push_str(text);
        self.out.push('\n');
        self.line_count += 1;
        Ok(())
    }
}

/// Format a float like the reference frontend: integers print without a
/// decimal point, and non-integers print the shortest round-trip
/// representation truncated to 16 significant digits (OverPy behavior;
/// evidence: the pinned oracle snapshots).
pub(crate) fn is_comparison_operator(name: &str) -> bool {
    matches!(name, "==" | "!=" | "<" | "<=" | ">" | ">=")
}

pub(crate) fn escape_string(value: &str) -> String {
    value.replace('"', "\\\"")
}

/// Re-escape a decoded value string the way the pinned oracle does (#87):
/// `\`, `"`, newline, and carriage return re-escape; tabs pass through raw
/// (byte-measured oracle behavior: `a\tb` emits a real tab, `a\nb` emits the
/// literal two-character `\n`).
pub(crate) fn escape_value_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out
}

/// Split a decoded string per the oracle's long-string rule (#87): when the
/// decoded length exceeds the Workshop 128-char limit, non-final segments
/// hold exactly 125 decoded chars and are emitted with a `{0}` continuation
/// placeholder (128 total text chars), chained as nested `Custom String`
/// arguments; the final segment holds the remainder without a placeholder.
/// Segment texts are re-escaped. Byte-measured basis: chunk sizes are
/// counted on the decoded string (70 escaped newlines — 140 escaped chars,
/// 70 decoded — emit unsplit; 129 decoded newlines split at 125 decoded).
pub(crate) fn split_string(value: &str) -> Vec<String> {
    if value.chars().count() <= 128 {
        return vec![escape_value_string(value)];
    }
    let mut segments = Vec::new();
    let mut rest = value;
    while rest.chars().count() > 125 {
        let chunk: String = rest.chars().take(125).collect();
        let mut text = escape_value_string(&chunk);
        text.push_str("{0}");
        segments.push(text);
        rest = &rest[chunk.len()..];
    }
    if !rest.is_empty() {
        segments.push(escape_value_string(rest));
    }
    segments
}

/// Escape a settings string value the way the pinned oracle does: every
/// decode the JSONC parser performed is re-escaped, so decoded values
/// round-trip to the oracle's spelling. Evidence: the inputhud description
/// (`\n` in the source block) is emitted by the oracle as the literal
/// two-character sequence `\n` in the Workshop settings section.
pub(crate) fn escape_settings_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out
}

/// Emit the nested continuation chain
/// `Custom String(seg0, Custom String(seg1, ...))`; segment texts are
/// pre-escaped, non-final segments carry the `{0}` placeholder. Iterative:
/// every segment except the first opens a `Custom String` level, then all
/// levels close.
pub(crate) fn emit_string_chain(spelling: &str, segments: &[String], out: &mut String) {
    let Some((first, rest)) = segments.split_first() else {
        return;
    };
    out.push_str(spelling);
    out.push('(');
    write!(out, "\"{first}\"").unwrap();
    for segment in rest {
        out.push_str(", ");
        out.push_str(spelling);
        out.push('(');
        write!(out, "\"{segment}\"").unwrap();
    }
    for _ in 0..=rest.len() {
        out.push(')');
    }
}

/// Render a constant format argument the way the oracle folds it: integers
/// without decimals, non-integers with exactly two decimals (JS `toFixed(2)`
/// rounding: `0.5` -> `0.50`, `0.125` -> `0.13`, #87).
pub(crate) fn fold_number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        let scaled = (value * 100.0).round();
        let sign = if scaled < 0.0 { "-" } else { "" };
        let scaled = scaled.abs() as i64;
        format!("{sign}{}.{:02}", scaled / 100, scaled % 100)
    }
}
