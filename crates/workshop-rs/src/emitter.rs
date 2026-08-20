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

use std::fmt::Write;

use crate::catalog::{Catalog, Kind, Locale};
use crate::error::{Result, WorkshopError};
use crate::format::format_number;
use crate::settings::table::{self, KeyKind, PathPart};
use crate::settings::{Settings as SettingsTree, SettingsNode};
use crate::wir;

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
    let mut emitter = Emitter {
        program,
        catalog,
        locale: locale.clone(),
        fallback: options.fallback_locale.clone(),
        fallback_ids: Vec::new(),
        out: String::new(),
    };
    emitter.run()?;
    Ok(EmitOutput {
        text: emitter.out,
        fallback_ids: emitter.fallback_ids,
    })
}

struct Emitter<'a> {
    program: &'a wir::Program,
    catalog: &'a Catalog,
    locale: Locale,
    /// The opt-in fallback locale for missing target-locale mappings.
    fallback: Option<Locale>,
    /// Canonical ids emitted with a fallback-locale spelling.
    fallback_ids: Vec<String>,
    out: String,
}

impl Emitter<'_> {
    fn run(&mut self) -> Result<()> {
        // Section order: settings, variables, subroutines, rules.
        if let Some(settings) = &self.program.settings {
            self.emit_settings(settings)?;
            self.out.push('\n');
        }
        if !self.program.global_variables.is_empty() || !self.program.player_variables.is_empty() {
            self.line(0, "variables {")?;
            if !self.program.global_variables.is_empty() {
                self.line(1, "global:")?;
                for variable in self.program.global_variables.iter() {
                    self.line(2, &format!("{}: {}", variable.index, variable.name))?;
                }
            }
            if !self.program.player_variables.is_empty() {
                self.line(1, "player:")?;
                for variable in self.program.player_variables.iter() {
                    self.line(2, &format!("{}: {}", variable.index, variable.name))?;
                }
            }
            self.line(0, "}")?;
            self.out.push('\n');
        }
        if !self.program.subroutines.is_empty() {
            self.line(0, "subroutines {")?;
            for subroutine in self.program.subroutines.iter() {
                self.line(1, &format!("{}: {}", subroutine.index, subroutine.name))?;
            }
            self.line(0, "}")?;
            self.out.push('\n');
        }
        // Rules with no actions are dropped, matching the pinned oracle
        // (pass-only and condition-without-actions rules emit nothing).
        let mut emitted_rules = 0;
        for rule in self.program.rules.iter() {
            if rule.actions.is_empty() {
                continue;
            }
            if emitted_rules > 0 {
                self.out.push('\n');
            }
            self.rule(rule)?;
            emitted_rules += 1;
        }
        // The oracle's raw artifact ends with a trailing blank line (the
        // committed snapshots strip it via the acquisition normalizer; the
        // pinned oracle's own output keeps it).
        if !self.out.is_empty() && !self.out.ends_with("\n\n") {
            self.out.push('\n');
        }
        Ok(())
    }

    /// Emit the `settings { ... }` section from the validated settings
    /// carrier, table-driven (fixture-evidenced names). Only runs on
    /// validated programs, so unknown keys cannot reach this point.
    fn emit_settings(&mut self, settings: &SettingsTree) -> Result<()> {
        self.line(0, "settings {")?;
        for child in &settings.children {
            let SettingsNode::Group { name, children, .. } = child else {
                return Err(self.malformed("settings block children must be groups"));
            };
            match name.as_str() {
                "main" | "lobby" => {
                    self.line(1, &format!("{name} {{"))?;
                    for member in children {
                        self.settings_member(member, 2, &[PathPart::Part(name)], None)?;
                    }
                    self.line(1, "}")?;
                }
                "gamemodes" => self.emit_modes(children)?,
                "heroes" => self.emit_heroes(children)?,
                _ => self.emit_opaque_group(children, name, 1)?,
            }
        }
        self.line(0, "}")?;
        Ok(())
    }

    /// Emit the `modes { <Mode> { ... } }` block of a gamemodes group.
    fn emit_modes(&mut self, modes: &[SettingsNode]) -> Result<()> {
        self.line(1, "modes {")?;
        for mode in modes {
            let SettingsNode::Group { name, children, .. } = mode else {
                return Err(self.malformed("mode entries must be groups"));
            };
            let display = match table::mode_name(name) {
                Some(english) => self.setting_name("modes", english, &format!("mode.{name}"))?,
                None => name.clone(),
            };
            // `enabled: false` prefixes the mode header; true renders with no
            // prefix (only false is evidenced in the corpus, #86).
            let disabled = children.iter().any(|member| {
                matches!(
                    member,
                    SettingsNode::Bool { name: n, value: false, .. } if n == "enabled"
                )
            });
            let header = if disabled {
                let disabled_name = self.setting_name("tokens", "disabled", "token.disabled")?;
                format!("{disabled_name} {display}")
            } else {
                display
            };
            self.line(2, &format!("{header} {{"))?;
            for member in children {
                if matches!(member, SettingsNode::Bool { name: n, .. } if n == "enabled") {
                    continue;
                }
                self.settings_member(
                    member,
                    3,
                    &[PathPart::Part("gamemodes"), PathPart::Part(name)],
                    None,
                )?;
            }
            self.line(2, "}")?;
        }
        self.line(1, "}")?;
        Ok(())
    }

    /// Emit the `heroes { <Team> { ... } }` block of a heroes group.
    fn emit_heroes(&mut self, teams: &[SettingsNode]) -> Result<()> {
        self.line(1, "heroes {")?;
        for team in teams {
            let SettingsNode::Group { name, children, .. } = team else {
                return Err(self.malformed("team entries must be groups"));
            };
            let english = table::team_name(name)
                .ok_or_else(|| self.malformed(format!("unknown team '{name}'")))?;
            let display = self.setting_name("teams", english, &format!("team.{name}"))?;
            self.line(2, &format!("{display} {{"))?;
            for member in children {
                match member {
                    SettingsNode::Group { name, children, .. } => {
                        let english = table::hero_name(name)
                            .ok_or_else(|| self.malformed(format!("unknown hero '{name}'")))?;
                        let hero = self.setting_name("heroes", english, &format!("hero.{name}"))?;
                        self.line(3, &format!("{hero} {{"))?;
                        for inner in children {
                            self.settings_member(
                                inner,
                                4,
                                &[PathPart::Part("heroes"), PathPart::Team, PathPart::Hero],
                                Some(name),
                            )?;
                        }
                        self.line(3, "}")?;
                    }
                    other => self.settings_member(
                        other,
                        3,
                        &[PathPart::Part("heroes"), PathPart::Team],
                        None,
                    )?,
                }
            }
            self.line(2, "}")?;
        }
        self.line(1, "}")?;
        Ok(())
    }

    /// Emit one leaf-level settings member (`Name: value`, lists as blocks).
    fn settings_member(
        &mut self,
        node: &SettingsNode,
        level: usize,
        path: &[PathPart],
        hero: Option<&str>,
    ) -> Result<()> {
        if let SettingsNode::Raw { name, value, .. } = node {
            if value.is_empty() {
                self.line(level, name)?;
            } else {
                self.line(level, &format!("{name}: {value}"))?;
            }
            return Ok(());
        }
        let name = node.name();
        let mut full = path.to_vec();
        full.push(PathPart::Part(name));
        let entry = table::lookup(&full).ok_or_else(|| {
            self.malformed(format!(
                "settings key '{}' is outside the emission table",
                table::path_string(&full)
            ))
        })?;
        let display_name =
            if let (Some(hero), Some(slot)) = (hero, table::ability_slot_for_path(&full)) {
                self.gameplay_setting_name(hero, slot, &table::path_string(&full))?
            } else {
                self.setting_name("labels", entry.workshop_name, &table::path_string(&full))?
            };
        match (node, &entry.kind) {
            (SettingsNode::String { value, .. }, KeyKind::String) => {
                self.line(
                    level,
                    &format!("{}: \"{}\"", display_name, escape_settings_string(value)),
                )?;
            }
            (SettingsNode::String { value, .. }, KeyKind::Enum(domain)) => {
                let english = table::enum_name(domain, value).ok_or_else(|| {
                    self.malformed(format!("unknown value '{value}' for settings key '{name}'"))
                })?;
                let display =
                    self.setting_name("enums", english, &format!("enum.{domain}.{value}"))?;
                self.line(level, &format!("{display_name}: {display}"))?;
            }
            (SettingsNode::Number { value, .. }, KeyKind::Number) => {
                self.line(level, &format!("{display_name}: {}", format_number(*value)))?;
            }
            (SettingsNode::Number { value, .. }, KeyKind::Percent) => {
                self.line(
                    level,
                    &format!("{display_name}: {}%", format_number(*value)),
                )?;
            }
            (SettingsNode::Bool { value, .. }, KeyKind::Bool) => {
                let rendered = self.setting_name(
                    "tokens",
                    if *value { "On" } else { "Off" },
                    if *value { "token.on" } else { "token.off" },
                )?;
                self.line(level, &format!("{display_name}: {rendered}"))?;
            }
            (SettingsNode::List { elements, .. }, KeyKind::ListMap) => {
                self.line(level, &format!("{display_name} {{"))?;
                for element in elements {
                    let english = table::map_name(&element.value).ok_or_else(|| {
                        self.malformed(format!(
                            "unknown map '{}' in settings list '{name}'",
                            element.value
                        ))
                    })?;
                    let display =
                        self.setting_name("maps", english, &format!("map.{}.name", element.value))?;
                    self.line(level + 1, &display)?;
                }
                self.line(level, "}")?;
            }
            (SettingsNode::List { elements, .. }, KeyKind::ListHero) => {
                self.line(level, &format!("{display_name} {{"))?;
                for element in elements {
                    let english = table::hero_name(&element.value).ok_or_else(|| {
                        self.malformed(format!(
                            "unknown hero '{}' in settings list '{name}'",
                            element.value
                        ))
                    })?;
                    let display = self.setting_name(
                        "heroes",
                        english,
                        &format!("hero.{}.name", element.value),
                    )?;
                    self.line(level + 1, &display)?;
                }
                self.line(level, "}")?;
            }
            _ => {
                return Err(self.malformed(format!(
                    "settings key '{name}' does not match its table kind"
                )));
            }
        }
        Ok(())
    }

    fn emit_opaque_group(
        &mut self,
        children: &[SettingsNode],
        name: &str,
        level: usize,
    ) -> Result<()> {
        self.line(level, &format!("{name} {{"))?;
        for child in children {
            match child {
                SettingsNode::Group { name, children, .. } => {
                    self.emit_opaque_group(children, name, level + 1)?;
                }
                _ => self.settings_member(child, level + 1, &[], None)?,
            }
        }
        self.line(level, "}")?;
        Ok(())
    }

    /// Resolve a settings spelling from the generated locale corpus. The
    /// English table remains the explicit fallback only when the caller opts
    /// into `en-US`, matching the catalog's missing-mapping contract.
    fn gameplay_setting_name(&mut self, hero: &str, slot: &str, id: &str) -> Result<String> {
        let resolve = |locale: &Locale| {
            crate::gameplay_data::builtin().ok().and_then(|catalog| {
                catalog
                    .query()
                    .ability_name(hero, slot, None, locale.as_str())
                    .ok()
                    .map(str::to_string)
            })
        };
        if let Some(name) = resolve(&self.locale) {
            return Ok(name);
        }
        if let Some(fallback) = &self.fallback {
            if let Some(name) = resolve(fallback) {
                if !self.fallback_ids.iter().any(|value| value == "settings") {
                    self.fallback_ids.push("settings".to_string());
                }
                return Ok(name);
            }
        }
        Err(WorkshopError::MissingMapping {
            kind: "setting",
            id: id.to_string(),
            locale: self.locale.clone(),
        })
    }

    fn setting_name(&mut self, section: &str, english: &str, id: &str) -> Result<String> {
        let en_us = Locale::new("en-US");
        if self.locale == en_us {
            return Ok(english.to_string());
        }
        if let Some(spelling) = table::localized_name(self.locale.as_str(), section, english) {
            return Ok(spelling.to_string());
        }
        if let Some(fallback) = &self.fallback {
            if *fallback == en_us {
                if !self.fallback_ids.iter().any(|value| value == "settings") {
                    self.fallback_ids.push("settings".to_string());
                }
                return Ok(english.to_string());
            }
        }
        Err(WorkshopError::MissingMapping {
            kind: "setting",
            id: id.to_string(),
            locale: self.locale.clone(),
        })
    }

    fn malformed(&self, message: impl Into<String>) -> WorkshopError {
        WorkshopError::Malformed {
            message: message.into(),
            span: None,
        }
    }

    fn rule(&mut self, rule: &wir::Rule) -> Result<()> {
        self.line(0, &format!("rule (\"{}\") {{", escape_string(&rule.name)))?;
        self.line(1, "event {")?;
        match &rule.event {
            wir::Event::Global => {
                let spelling = self.spelling(Kind::Event, "global")?;
                self.line(2, &format!("{spelling};"))?;
            }
            wir::Event::EachPlayer => {
                let spelling = self.spelling(Kind::Event, "eachPlayer")?;
                self.line(2, &format!("{spelling};"))?;
                self.event_filters(wir::EventTeam::All, &wir::EventTarget::All)?;
            }
            wir::Event::EachPlayerWithFilters { team, target } => {
                let spelling = self.spelling(Kind::Event, "eachPlayer")?;
                self.line(2, &format!("{spelling};"))?;
                self.event_filters(*team, target)?;
            }
            wir::Event::Player { kind, team, target } => {
                let spelling = self.spelling(Kind::Event, kind.catalog_id())?;
                self.line(2, &format!("{spelling};"))?;
                self.event_filters(*team, target)?;
            }
            wir::Event::Subroutine(subroutine) => {
                let spelling = self.spelling(Kind::Event, "subroutine")?;
                self.line(2, &format!("{spelling};"))?;
                let name = self
                    .program
                    .subroutines
                    .get(*subroutine)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| "<dangling>".to_string());
                self.line(2, &format!("{name};"))?;
            }
        }
        self.line(1, "}")?;
        if !rule.conditions.is_empty() {
            self.line(1, "conditions {")?;
            for condition in &rule.conditions {
                let mut text = String::new();
                // Reference normalization: comparison conditions render
                // infix; other conditions render as `value == True`.
                if let Some(wir::Value::Call { name, args }) =
                    self.program.values.get(*condition).map(|node| &node.value)
                {
                    if is_comparison_operator(name) && args.len() == 2 {
                        self.value(args[0], &mut text)?;
                        write!(text, " {name} ").unwrap();
                        self.value(args[1], &mut text)?;
                    } else {
                        self.value(*condition, &mut text)?;
                        text.push_str(" == True");
                    }
                } else {
                    self.value(*condition, &mut text)?;
                    text.push_str(" == True");
                }
                self.line(2, &format!("{text};"))?;
            }
            self.line(1, "}")?;
        }
        if !rule.actions.is_empty() {
            self.line(1, "actions {")?;
            for (index, action) in rule.actions.iter().enumerate() {
                let rule_final = index + 1 == rule.actions.len();
                self.action(*action, 2, rule_final)?;
            }
            self.line(1, "}")?;
        }
        self.line(0, "}")?;
        Ok(())
    }

    fn event_filters(&mut self, team: wir::EventTeam, target: &wir::EventTarget) -> Result<()> {
        let team = match team {
            wir::EventTeam::All => "ALL",
            wir::EventTeam::Team1 => "TEAM_1",
            wir::EventTeam::Team2 => "TEAM_2",
        };
        let team = self.enum_spelling("EventTeam", team)?;
        self.line(2, &format!("{team};"))?;
        let target = match target {
            wir::EventTarget::All => self.enum_spelling("EventPlayer", "ALL")?,
            wir::EventTarget::Slot(slot) => {
                self.enum_spelling("EventPlayer", &format!("SLOT_{slot}"))?
            }
            wir::EventTarget::Hero(hero) => self.enum_spelling("Hero", hero)?,
        };
        self.line(2, &format!("{target};"))?;
        Ok(())
    }

    /// Emit one rule action; `rule_final` marks the last action of the rule,
    /// for which an `if`/`if-else` closes without the trailing `End;`
    /// (the pinned oracle's spelling, #87).
    fn action(&mut self, id: wir::ActionId, level: usize, rule_final: bool) -> Result<()> {
        let Some(action) = self.program.actions.get(id) else {
            return Err(WorkshopError::Malformed {
                message: format!("dangling action {id}"),
                span: None,
            });
        };
        match action {
            wir::Action::SetGlobalVariable {
                variable, value, ..
            } => {
                let name = self.global_name(*variable)?;
                let mut value_text = String::new();
                self.value(*value, &mut value_text)?;
                let keyword = self.spelling(Kind::Structural, "setGlobalVariable")?;
                self.line(level, &format!("{keyword}({name}, {value_text});"))?;
            }
            wir::Action::ModifyGlobalVariable {
                variable,
                op,
                value,
                ..
            } => {
                let name = self.global_name(*variable)?;
                let op = self.modify_op_spelling(*op)?;
                let mut value_text = String::new();
                self.value(*value, &mut value_text)?;
                let keyword = self.spelling(Kind::Structural, "modifyGlobalVariable")?;
                self.line(level, &format!("{keyword}({name}, {op}, {value_text});"))?;
            }
            wir::Action::SetPlayerVariable {
                player,
                variable,
                value,
                ..
            } => {
                let mut player_text = String::new();
                self.value(*player, &mut player_text)?;
                let name = self.player_name(*variable)?;
                let mut value_text = String::new();
                self.value(*value, &mut value_text)?;
                let keyword = self.spelling(Kind::Structural, "setPlayerVariable")?;
                self.line(
                    level,
                    &format!("{keyword}({player_text}, {name}, {value_text});"),
                )?;
            }
            wir::Action::ModifyPlayerVariable {
                player,
                variable,
                op,
                value,
                ..
            } => {
                let mut player_text = String::new();
                self.value(*player, &mut player_text)?;
                let name = self.player_name(*variable)?;
                let op = self.modify_op_spelling(*op)?;
                let mut value_text = String::new();
                self.value(*value, &mut value_text)?;
                let keyword = self.spelling(Kind::Structural, "modifyPlayerVariable")?;
                self.line(
                    level,
                    &format!("{keyword}({player_text}, {name}, {op}, {value_text});"),
                )?;
            }
            wir::Action::CallSubroutine { subroutine, .. } => {
                let name = self
                    .program
                    .subroutines
                    .get(*subroutine)
                    .map(|s| s.name.clone())
                    .ok_or_else(|| WorkshopError::Unknown {
                        kind: "subroutine",
                        spelling: format!("<{subroutine}>"),
                        locale: self.locale.clone(),
                        span: None,
                    })?;
                let keyword = self.spelling(Kind::Structural, "callSubroutine")?;
                self.line(level, &format!("{keyword}({name});"))?;
            }
            wir::Action::If {
                branches,
                else_body,
                ..
            } => {
                for (index, branch) in branches.iter().enumerate() {
                    let mut condition = String::new();
                    self.value(branch.condition, &mut condition)?;
                    let keyword =
                        self.spelling(Kind::Structural, if index == 0 { "if" } else { "elseIf" })?;
                    self.line(level, &format!("{keyword}({condition});"))?;
                    for action in &branch.body {
                        self.action(*action, level + 1, false)?;
                    }
                }
                if let Some(else_body) = else_body {
                    let keyword = self.spelling(Kind::Structural, "else")?;
                    self.line(level, &format!("{keyword};"))?;
                    for action in else_body {
                        self.action(*action, level + 1, false)?;
                    }
                }
                // A rule-final if closes the rule without `End;` (oracle
                // spelling); nested and middle-of-rule ifs keep it.
                if !rule_final {
                    let keyword = self.spelling(Kind::Structural, "end")?;
                    self.line(level, &format!("{keyword};"))?;
                }
            }
            wir::Action::While {
                condition, body, ..
            } => {
                let mut text = String::new();
                self.value(*condition, &mut text)?;
                let keyword = self.spelling(Kind::Structural, "while")?;
                self.line(level, &format!("{keyword}({text});"))?;
                for action in body {
                    self.action(*action, level + 1, false)?;
                }
                let end = self.spelling(Kind::Structural, "end")?;
                self.line(level, &format!("{end};"))?;
            }
            wir::Action::ForGlobalVariable {
                variable,
                start,
                stop,
                step,
                body,
                ..
            } => {
                let name = self.global_name(*variable)?;
                let mut start_text = String::new();
                let mut stop_text = String::new();
                let mut step_text = String::new();
                self.value(*start, &mut start_text)?;
                self.value(*stop, &mut stop_text)?;
                self.value(*step, &mut step_text)?;
                let keyword = self.spelling(Kind::Structural, "forGlobalVariable")?;
                self.line(
                    level,
                    &format!("{keyword}({name}, {start_text}, {stop_text}, {step_text});"),
                )?;
                for action in body {
                    self.action(*action, level + 1, false)?;
                }
                let end = self.spelling(Kind::Structural, "end")?;
                self.line(level, &format!("{end};"))?;
            }
            wir::Action::ForPlayerVariable {
                player,
                variable,
                start,
                stop,
                step,
                body,
                ..
            } => {
                let mut player_text = String::new();
                let mut start_text = String::new();
                let mut stop_text = String::new();
                let mut step_text = String::new();
                self.value(*player, &mut player_text)?;
                self.value(*start, &mut start_text)?;
                self.value(*stop, &mut stop_text)?;
                self.value(*step, &mut step_text)?;
                let name = self.player_name(*variable)?;
                self.line(
                    level,
                    &format!(
                        "For Player Variable({player_text}, {name}, {start_text}, {stop_text}, {step_text});"
                    ),
                )?;
                for action in body {
                    self.action(*action, level + 1, false)?;
                }
                let end = self.spelling(Kind::Structural, "end")?;
                self.line(level, &format!("{end};"))?;
            }
            wir::Action::Debug { value, .. } => {
                // `debug(value)` displays the value as HUD text. The
                // reference formats values with type-aware machinery; Wright
                // emits a semantically equivalent but presentation-simpler
                // Create HUD Text (documented intentional difference).
                self.emit_hud_text(*value, level, true)?;
            }
            wir::Action::Print { message, .. } => {
                self.emit_hud_text(*message, level, false)?;
            }
            wir::Action::AssignMember {
                target, op, value, ..
            } => {
                let mut target_text = String::new();
                let mut value_text = String::new();
                self.value(*target, &mut target_text)?;
                self.value(*value, &mut value_text)?;
                let operator = match op {
                    None => "=".to_string(),
                    Some(op) => {
                        let token = match op {
                            wir::ModifyOp::Add => "+",
                            wir::ModifyOp::Subtract => "-",
                            wir::ModifyOp::Multiply => "*",
                            wir::ModifyOp::Divide => "/",
                            wir::ModifyOp::Modulo => "%",
                            _ => {
                                return Err(WorkshopError::Unsupported {
                                    message: format!(
                                        "unsupported member assignment operator {op:?}"
                                    ),
                                    span: None,
                                });
                            }
                        };
                        format!("{token}=")
                    }
                };
                self.line(level, &format!("{target_text} {operator} {value_text};"))?;
            }
            wir::Action::Call { name, args, .. } => {
                // The chase family dispatches on the first argument's
                // variable kind, mirroring the pinned reference: a global
                // variable emits the global form with the argument list
                // unchanged; a player variable emits the player form with
                // the receiver split into `player, name` leading arguments
                // (the frontend guarantees a variable first argument,
                // issue #110).
                if matches!(name.as_str(), "chaseAtRate" | "chaseOverTime") {
                    let player_var = args.first().and_then(|id| {
                        self.program
                            .values
                            .get(*id)
                            .and_then(|node| match &node.value {
                                wir::Value::PlayerVariable { player, variable } => {
                                    Some((*player, *variable))
                                }
                                _ => None,
                            })
                    });
                    let spelling = if let Some((player, variable)) = player_var {
                        let id = if name == "chaseAtRate" {
                            "chasePlayerVariableAtRate"
                        } else {
                            "chasePlayerVariableOverTime"
                        };
                        let spelling = self.spelling(Kind::Action, id)?;
                        // `Chase Player Variable At Rate(player, name, …)`:
                        // the receiver splits into `player, name` leading
                        // arguments (the pinned oracle's spelling).
                        let mut text = String::new();
                        self.value(player, &mut text)?;
                        let mut parts = vec![text, self.player_name(variable)?];
                        for arg in args.iter().skip(1) {
                            let mut part = String::new();
                            self.value(*arg, &mut part)?;
                            parts.push(part);
                        }
                        return self.line(level, &format!("{spelling}({});", parts.join(", ")));
                    } else {
                        self.spelling(Kind::Action, name)?
                    };
                    let mut args_text = String::new();
                    self.args(args, &mut args_text)?;
                    return self.line(level, &format!("{spelling}({args_text});"));
                }
                // Native `.opy` action names map to canonical catalog ids at
                // emission (presentation concern).
                let canonical = match name.as_str() {
                    "createBeam" => Some("createBeamEffect"),
                    _ => None,
                };
                let spelling = if let Some(canonical) = canonical {
                    self.spelling(Kind::Action, canonical)?
                } else {
                    self.spelling(Kind::Action, name)?
                };
                if args.is_empty() {
                    self.line(level, &format!("{spelling};"))?;
                } else {
                    let mut args_text = String::new();
                    self.args(args, &mut args_text)?;
                    self.line(level, &format!("{spelling}({args_text});"))?;
                }
            }
        }
        Ok(())
    }

    /// Emit a `debug`/`print` action as a `Create HUD Text` effect.
    ///
    /// `debug` renders the value into the HUD body; `print` renders the
    /// message directly (a `format` value already carries the text). Every
    /// fixed token resolves through the catalog, so the effect is
    /// locale-correct by data and fails explicitly on missing target-locale
    /// mappings.
    fn emit_hud_text(&mut self, value: wir::ValueId, level: usize, is_debug: bool) -> Result<()> {
        let mut body = String::new();
        if is_debug {
            // Display the value in the HUD body: Custom String("{0}", value).
            body.push_str(&self.spelling(Kind::Value, "customString")?);
            body.push_str("(\"{0}\", ");
            self.value(value, &mut body)?;
            body.push(')');
        } else {
            self.value(value, &mut body)?;
        }
        // Create HUD Text(All Players(All Teams), Null, header, body, text,
        // location, sort order, header color, subheader color, text color,
        // reevaluation, spectators) — the canonical catalog layout (probe P6
        // emission), so the emitted text reparses against the catalog's
        // expected enum domains at the canonical positions.
        let mut line = String::new();
        line.push_str(&self.spelling(Kind::Action, "createHudText")?);
        line.push('(');
        line.push_str(&self.spelling(Kind::Value, "allPlayers")?);
        line.push('(');
        line.push_str(&self.enum_spelling("Team", "ALL")?);
        line.push_str("), Null, ");
        line.push_str(&body);
        line.push_str(", Null, ");
        line.push_str(&self.enum_spelling("HudPosition", "LEFT")?);
        line.push_str(", -9999, Color(");
        line.push_str(&self.enum_spelling("Color", "WHITE")?);
        line.push_str("), Color(");
        line.push_str(&self.enum_spelling("Color", "WHITE")?);
        line.push_str("), Color(");
        line.push_str(&self.enum_spelling("Color", "WHITE")?);
        line.push_str("), ");
        line.push_str(&self.enum_spelling("HudReeval", "VISIBILITY_AND_STRING")?);
        line.push_str(", ");
        line.push_str(&self.enum_spelling("SpecVisibility", "VISIBLE_ALWAYS")?);
        line.push_str(");");
        self.line(level, &line)?;
        Ok(())
    }

    fn args(&mut self, args: &[wir::ValueId], out: &mut String) -> Result<()> {
        for (index, arg) in args.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            self.value(*arg, out)?;
        }
        Ok(())
    }

    fn value(&mut self, id: wir::ValueId, out: &mut String) -> Result<()> {
        let Some(node) = self.program.values.get(id) else {
            return Err(WorkshopError::Malformed {
                message: format!("dangling value {id}"),
                span: None,
            });
        };
        match &node.value {
            wir::Value::Number { text, .. } => {
                // Literal spellings carry through (the oracle preserves the
                // source spelling, e.g. `0.0`; computed values carry the
                // formatted spelling, #87).
                out.push_str(text);
            }
            wir::Value::String(value) => {
                // Value-position strings wrap in `Custom String("...")` with
                // re-escaped content and long-string splitting, the pinned
                // oracle's spelling (evidence: array elements, initializers,
                // assignments, call arguments, comparisons — #87). The only
                // bare string value is the `Custom String` text argument,
                // handled in the call arm below.
                self.emit_string_value(value, out)?;
            }
            wir::Value::Bool(true) => out.push_str("True"),
            wir::Value::Bool(false) => out.push_str("False"),
            wir::Value::Null => out.push_str("Null"),
            wir::Value::Array(elements) => {
                if elements.is_empty() {
                    // The canonical empty-array constant (reference emission).
                    out.push_str(&self.spelling(Kind::Value, "emptyArray")?);
                } else {
                    out.push_str(&self.spelling(Kind::Value, "array")?);
                    out.push('(');
                    self.args(elements, out)?;
                    out.push(')');
                }
            }
            wir::Value::Vector { x, y, z } => {
                out.push_str(&self.spelling(Kind::Value, "vector")?);
                out.push('(');
                self.value(*x, out)?;
                out.push_str(", ");
                self.value(*y, out)?;
                out.push_str(", ");
                self.value(*z, out)?;
                out.push(')');
            }
            wir::Value::Enum { value_type, value } => {
                // The `.opy`-layer `EffectReeval` domain is the same Workshop
                // reevaluation domain as `HudReeval` (member ids align); map
                // it at emission to avoid catalog domain collisions.
                let catalog_domain = if value_type == "EffectReeval" {
                    "HudReeval"
                } else {
                    value_type
                };
                let spelling = self.enum_spelling(catalog_domain, value)?;
                // Color values use the constructor form; other domains use
                // bare member spellings (the canonical corpus form). The
                // Team/Color spelling collision (`Team 2` is both a Team and
                // a Team color) is the one ambiguity unpinned by the
                // catalog's paramDomains, so Team members qualify with the
                // constructor form and the emitted text reparses
                // deterministically (round-trip contract; pinned P4
                // evidence).
                if value_type == "Color" || value_type == "Team" {
                    write!(out, "{catalog_domain}({spelling})").unwrap();
                } else {
                    out.push_str(&spelling);
                }
            }
            wir::Value::GlobalVariable(variable) => {
                let name = self.global_name(*variable)?;
                write!(out, "Global.{name}").unwrap();
            }
            wir::Value::PlayerVariable { player, variable } => {
                // The oracle's spelling parenthesizes the receiver:
                // `Set Global Variable(g, (Event Player).p)` (#87).
                out.push('(');
                self.value(*player, out)?;
                out.push(')');
                let name = self.player_name(*variable)?;
                write!(out, ".{name}").unwrap();
            }
            wir::Value::EventPlayer => out.push_str(&self.spelling(Kind::Value, "eventPlayer")?),
            wir::Value::Call { name, args } => {
                if is_comparison_operator(name) {
                    // Canonical form: Compare(a, op, b).
                    if args.len() != 2 {
                        return Err(WorkshopError::Malformed {
                            message: format!("comparison call '{name}' must have 2 args"),
                            span: None,
                        });
                    }
                    out.push_str(&self.spelling(Kind::Value, "compare")?);
                    out.push('(');
                    self.value(args[0], out)?;
                    write!(out, ", {name}, ").unwrap();
                    self.value(args[1], out)?;
                    out.push(')');
                    return Ok(());
                }
                // Unary minus renders as Multiply(-1, x); the reference folds
                // literal negation, handled by the compat constant-fold pass.
                if name == "-" && args.len() == 1 {
                    out.push_str(&self.spelling(Kind::Value, "multiply")?);
                    out.push_str("(-1, ");
                    self.value(args[0], out)?;
                    out.push(')');
                    return Ok(());
                }
                // `getAllPlayers()` is OverPy's All Players(All Teams).
                if name == "getAllPlayers" && args.is_empty() {
                    out.push_str(&self.spelling(Kind::Value, "allPlayers")?);
                    out.push('(');
                    out.push_str(&self.enum_spelling("Team", "ALL")?);
                    out.push(')');
                    return Ok(());
                }
                // Binary arithmetic operators and native `.opy` source names
                // map to canonical catalog ids at emission (presentation
                // concern; the compat pass folds constants to match the
                // reference exactly).
                let canonical = match name.as_str() {
                    "+" => Some("add"),
                    "-" => Some("subtract"),
                    "*" => Some("multiply"),
                    "/" => Some("divide"),
                    "len" => Some("countOf"),
                    "abs" => Some("absoluteValue"),
                    "sqrt" => Some("squareRoot"),
                    "createBeam" => Some("createBeamEffect"),
                    "random.uniform" => Some("randomReal"),
                    "random.choice" => Some("randomValueInArray"),
                    "format" => Some("customString"),
                    _ => None,
                };
                let spelling = if let Some(canonical) = canonical {
                    self.spelling(Kind::Value, canonical)?
                } else {
                    self.spelling(Kind::Value, name)?
                };
                // `format` (frontend) and `customString` (parsed ws text) are
                // the same node.
                let is_custom_string = canonical == Some("customString") || name == "customString";
                if args.is_empty() {
                    // Constants (e.g. Empty Array) emit as bare spellings.
                    out.push_str(&spelling);
                } else if is_custom_string {
                    // `.format()` calls canonicalize: constant numeric
                    // arguments fold into the substituted text, implicit
                    // `{}` placeholders renumber to the oracle's explicit
                    // form, and remaining variable arguments wrap (the
                    // oracle spelling, #87). The canonical text feeds the
                    // value-string path (re-escaping/splitting) when no
                    // arguments remain.
                    match self.canonicalize_format_call(args)? {
                        Some((text, variable_args)) => {
                            if variable_args.is_empty() {
                                self.emit_string_value(&text, out)?;
                            } else {
                                out.push_str(&spelling);
                                out.push('(');
                                write!(out, "\"{}\"", escape_value_string(&text)).unwrap();
                                if !variable_args.is_empty() {
                                    out.push_str(", ");
                                }
                                self.args(&variable_args, out)?;
                                out.push(')');
                            }
                        }
                        None => {
                            // The `Custom String` text argument stays bare
                            // (the oracle spelling); the remaining arguments
                            // are values and wrap (#87).
                            out.push_str(&spelling);
                            out.push('(');
                            self.bare_string_value(args[0], out)?;
                            if args.len() > 1 {
                                out.push_str(", ");
                            }
                            self.args(&args[1..], out)?;
                            out.push(')');
                        }
                    }
                } else {
                    out.push_str(&spelling);
                    out.push('(');
                    self.args(args, out)?;
                    out.push(')');
                }
            }
        }
        Ok(())
    }

    /// Fold a `Custom String` call whose text argument and constant numeric
    /// arguments are all literals into the substituted text (the oracle's
    /// Canonicalize a `Custom String`/`.format()` call (#87): constant
    /// numeric arguments fold into the substituted text (the oracle's
    /// spelling), implicit `{}` placeholders renumber positionally to the
    /// explicit `{N}` form, and the remaining variable arguments are
    /// returned in placeholder order. Returns `None` (rendered unchanged)
    /// when nothing canonicalizes: explicit-only texts without constants,
    /// texts mixing implicit and explicit placeholders (the oracle rejects
    /// those), out-of-range placeholders, or non-String text arguments.
    fn canonicalize_format_call(
        &self,
        args: &[wir::ValueId],
    ) -> Result<Option<(String, Vec<wir::ValueId>)>> {
        if args.len() < 2 {
            return Ok(None);
        }
        let Some(text) = self.program.values.get(args[0]) else {
            return Ok(None);
        };
        let wir::Value::String(text) = &text.value else {
            return Ok(None);
        };
        let format_args = &args[1..];
        // Classify the placeholders: implicit `{}` consumes the next
        // argument, explicit `{N}` references argument N.
        let mut has_implicit = false;
        let mut has_explicit = false;
        let mut out_of_range = false;
        let mut cursor = 0usize;
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut inner = String::new();
                let mut closed = false;
                for next in chars.by_ref() {
                    if next == '}' {
                        closed = true;
                        break;
                    }
                    inner.push(next);
                }
                if !closed {
                    break; // unterminated brace: literal text
                }
                if inner.is_empty() {
                    if cursor >= format_args.len() {
                        out_of_range = true;
                    }
                    cursor += 1;
                    has_implicit = true;
                } else if inner.chars().all(|c| c.is_ascii_digit()) {
                    match inner.parse::<usize>() {
                        Ok(index) if index < format_args.len() => has_explicit = true,
                        _ => out_of_range = true,
                    }
                } else {
                    out_of_range = true;
                }
            }
        }
        if out_of_range || (has_implicit && has_explicit) {
            return Ok(None);
        }
        let mut any_constant = false;
        for id in format_args {
            let Some(node) = self.program.values.get(*id) else {
                return Ok(None);
            };
            if matches!(node.value, wir::Value::Number { .. }) {
                any_constant = true;
            }
        }
        if !has_implicit && !any_constant {
            return Ok(None);
        }
        // Canonicalize: fold constants inline at their placeholder, renumber
        // variable placeholders positionally, keep variable arguments in
        // placeholder order.
        let mut canonical = String::with_capacity(text.len());
        let mut variable_args = Vec::new();
        let mut variable_index = 0usize;
        let mut cursor = 0usize;
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut inner = String::new();
                let mut closed = false;
                for next in chars.by_ref() {
                    if next == '}' {
                        closed = true;
                        break;
                    }
                    inner.push(next);
                }
                if !closed {
                    canonical.push('{');
                    canonical.push_str(&inner);
                    break;
                }
                let index = if inner.is_empty() {
                    let index = cursor;
                    cursor += 1;
                    index
                } else {
                    match inner.parse::<usize>() {
                        Ok(index) => index,
                        Err(_) => {
                            canonical.push('{');
                            canonical.push_str(&inner);
                            canonical.push('}');
                            continue;
                        }
                    }
                };
                let Some(arg) = format_args.get(index).copied() else {
                    canonical.push('{');
                    canonical.push_str(&inner);
                    canonical.push('}');
                    continue;
                };
                let node = self.program.values.get(arg);
                if let Some(wir::Value::Number { value, .. }) = node.map(|node| &node.value) {
                    canonical.push_str(&fold_number(*value));
                } else {
                    write!(canonical, "{{{variable_index}}}").unwrap();
                    variable_index += 1;
                    variable_args.push(arg);
                }
            } else {
                canonical.push(ch);
            }
        }
        Ok(Some((canonical, variable_args)))
    }

    /// The localized spelling of a modify operator, resolved through the
    /// catalog (fallback-aware).
    fn modify_op_spelling(&mut self, op: wir::ModifyOp) -> Result<String> {
        let id = match op {
            wir::ModifyOp::Add => "add",
            wir::ModifyOp::Subtract => "subtract",
            wir::ModifyOp::Multiply => "multiply",
            wir::ModifyOp::Divide => "divide",
            wir::ModifyOp::Modulo => "modulo",
            wir::ModifyOp::RaiseToPower => "raiseToPower",
            wir::ModifyOp::AppendToArray => "appendToArray",
            wir::ModifyOp::RemoveFromArray => "removeFromArray",
            wir::ModifyOp::RemoveFromArrayByIndex => "removeFromArrayByIndex",
        };
        self.spelling(Kind::Operator, id)
    }

    /// The localized spelling of a canonical builtin id, resolving through
    /// the catalog: a dangling id is `Unknown`, an id without a target-locale
    /// mapping is `MissingMapping` unless an opt-in fallback locale declares
    /// one (recorded in [`Emitter::fallback_ids`]).
    fn spelling(&mut self, kind: Kind, id: &str) -> Result<String> {
        let Some(entry) = self.catalog.entry(kind, id) else {
            return Err(WorkshopError::Unknown {
                kind: kind.as_str(),
                spelling: id.to_string(),
                locale: self.locale.clone(),
                span: None,
            });
        };
        if let Some(spelling) = entry.spelling(&self.locale) {
            return Ok(spelling.to_string());
        }
        if let Some(fallback) = &self.fallback {
            if let Some(spelling) = entry.spelling(fallback) {
                self.fallback_ids.push(id.to_string());
                return Ok(spelling.to_string());
            }
        }
        Err(WorkshopError::MissingMapping {
            kind: kind.as_str(),
            id: id.to_string(),
            locale: self.locale.clone(),
        })
    }

    /// The localized spelling of a canonical enum member, resolving through
    /// the catalog (fallback-aware; see [`Emitter::spelling`]).
    fn enum_spelling(&mut self, domain: &str, member: &str) -> Result<String> {
        let Some(domain_entry) = self.catalog.enum_domain(domain) else {
            return Err(WorkshopError::Unknown {
                kind: "enum domain",
                spelling: domain.to_string(),
                locale: self.locale.clone(),
                span: None,
            });
        };
        let Some(member_entry) = domain_entry.members.iter().find(|m| m.member == member) else {
            return Err(WorkshopError::Unknown {
                kind: "enum member",
                spelling: format!("{domain}.{member}"),
                locale: self.locale.clone(),
                span: None,
            });
        };
        if let Some(spelling) = member_entry.spelling(&self.locale) {
            return Ok(spelling.to_string());
        }
        if let Some(fallback) = &self.fallback {
            if let Some(spelling) = member_entry.spelling(fallback) {
                self.fallback_ids.push(format!("{domain}.{member}"));
                return Ok(spelling.to_string());
            }
        }
        Err(WorkshopError::MissingMapping {
            kind: "enum member",
            id: format!("{domain}.{member}"),
            locale: self.locale.clone(),
        })
    }

    /// Render a value that must stay a bare string (the `Custom String` text
    /// argument). Any non-string value falls back to the normal renderer.
    fn bare_string_value(&mut self, id: wir::ValueId, out: &mut String) -> Result<()> {
        let Some(node) = self.program.values.get(id) else {
            return Err(WorkshopError::Malformed {
                message: format!("dangling value {id}"),
                span: None,
            });
        };
        if let wir::Value::String(value) = &node.value {
            write!(out, "\"{}\"", escape_value_string(value)).unwrap();
            return Ok(());
        }
        self.value(id, out)
    }

    /// Emit a value-position string as `Custom String("...")`, splitting it
    /// into a continuation chain when it exceeds the Workshop 128-char limit.
    fn emit_string_value(&mut self, value: &str, out: &mut String) -> Result<()> {
        let spelling = self.spelling(Kind::Value, "customString")?;
        let segments = split_string(value);
        emit_string_chain(&spelling, &segments, out);
        Ok(())
    }

    fn global_name(&self, id: wir::GlobalVarId) -> Result<String> {
        self.program
            .global_variables
            .get(id)
            .map(|variable| variable.name.clone())
            .ok_or_else(|| WorkshopError::Unknown {
                kind: "global variable",
                spelling: format!("<{id}>"),
                locale: self.locale.clone(),
                span: None,
            })
    }

    fn player_name(&self, id: wir::PlayerVarId) -> Result<String> {
        self.program
            .player_variables
            .get(id)
            .map(|variable| variable.name.clone())
            .ok_or_else(|| WorkshopError::Unknown {
                kind: "player variable",
                spelling: format!("<{id}>"),
                locale: self.locale.clone(),
                span: None,
            })
    }

    fn line(&mut self, level: usize, text: &str) -> Result<()> {
        for _ in 0..level {
            self.out.push_str("    ");
        }
        self.out.push_str(text);
        self.out.push('\n');
        Ok(())
    }
}

/// Format a float like the reference frontend: integers print without a
/// decimal point, and non-integers print the shortest round-trip
/// representation truncated to 16 significant digits (OverPy behavior;
/// evidence: the pinned oracle snapshots).
fn is_comparison_operator(name: &str) -> bool {
    matches!(name, "==" | "!=" | "<" | "<=" | ">" | ">=")
}

fn escape_string(value: &str) -> String {
    value.replace('"', "\\\"")
}

/// Re-escape a decoded value string the way the pinned oracle does (#87):
/// `\`, `"`, newline, and carriage return re-escape; tabs pass through raw
/// (byte-measured oracle behavior: `a\tb` emits a real tab, `a\nb` emits the
/// literal two-character `\n`).
fn escape_value_string(value: &str) -> String {
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
fn split_string(value: &str) -> Vec<String> {
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
fn escape_settings_string(value: &str) -> String {
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
fn emit_string_chain(spelling: &str, segments: &[String], out: &mut String) {
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
fn fold_number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        let scaled = (value * 100.0).round();
        let sign = if scaled < 0.0 { "-" } else { "" };
        let scaled = scaled.abs() as i64;
        format!("{sign}{}.{:02}", scaled / 100, scaled % 100)
    }
}
