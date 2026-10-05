use super::member::{self, Member};
use crate::core::suggest;
use crate::output::emitter::*;
use crate::source::Span;

impl EmitContext<'_> {
    /// Emit the `settings { ... }` section from the settings carrier,
    /// table-driven (fixture-backed names). Programs that build settings
    /// trees directly (rather than parsing raw Workshop) check table
    /// acceptance through [`crate::settings::check_emission`] first.
    pub(crate) fn emit_settings(&mut self, settings: &SettingsTree) -> Result<()> {
        let settings_keyword = self.structural("settings")?;
        self.line(0, &format!("{settings_keyword} {{"))?;
        for child in &settings.children {
            if let SettingsNode::Workshop { children, .. } = child {
                self.emit_workshop_settings(children, 1)?;
                continue;
            }
            let SettingsNode::Group { name, children, .. } = child else {
                return Err(WorkshopError::malformed(
                    "settings block children must be groups",
                    child.span(),
                ));
            };
            match name.as_str() {
                "main" | "lobby" => {
                    let display =
                        self.setting_name("namespaces", name, &format!("namespace.{name}"))?;
                    self.line(1, &format!("{display} {{"))?;
                    for member in children {
                        self.settings_member(member, 2, &[PathPart::Part(name)], None)?;
                    }
                    self.line(1, "}")?;
                }
                "gamemodes" => self.emit_modes(children)?,
                "heroes" => self.emit_heroes(children)?,
                "extensions" => {
                    let display =
                        self.setting_name("namespaces", "extensions", "namespace.extensions")?;
                    self.line(1, &format!("{display} {{"))?;
                    for member in children {
                        self.settings_member(member, 2, &[PathPart::Part("extensions")], None)?;
                    }
                    self.line(1, "}")?;
                }
                _ => self.emit_opaque_group(children, name, 1)?,
            }
        }
        self.line(0, "}")?;
        Ok(())
    }

    pub(crate) fn emit_workshop_settings(
        &mut self,
        children: &[SettingsNode],
        level: usize,
    ) -> Result<()> {
        let workshop = self.setting_name("namespaces", "workshop", "namespace.workshop")?;
        self.line(level, &format!("{workshop} {{"))?;
        for child in children {
            self.emit_workshop_node(child, level + 1)?;
        }
        self.line(level, "}")?;
        Ok(())
    }

    pub(crate) fn emit_workshop_node(&mut self, node: &SettingsNode, level: usize) -> Result<()> {
        match node {
            SettingsNode::Group { name, children, .. } => {
                self.line(level, &format!("{name} {{"))?;
                for child in children {
                    self.emit_workshop_node(child, level + 1)?;
                }
                self.line(level, "}")?;
                Ok(())
            }
            SettingsNode::Workshop { children, .. } => self.emit_workshop_settings(children, level),
            SettingsNode::Raw { name, value, .. } => {
                if value.is_empty() {
                    self.line(level, name)
                } else {
                    self.line(level, &format!("{name}: {value}"))
                }
            }
            _ => Err(WorkshopError::malformed(
                "settings.workshop contains a typed builtin setting",
                node.span(),
            )),
        }
    }

    /// Emit the `modes { <Mode> { ... } }` block of a gamemodes group.
    pub(crate) fn emit_modes(&mut self, modes: &[SettingsNode]) -> Result<()> {
        let modes_keyword = self.setting_name("namespaces", "modes", "namespace.modes")?;
        self.line(1, &format!("{modes_keyword} {{"))?;
        for mode in modes {
            let SettingsNode::Group { name, children, .. } = mode else {
                return Err(WorkshopError::malformed(
                    "mode entries must be groups",
                    mode.span(),
                ));
            };
            let display = match table::mode_name(name) {
                Some(english) => self.setting_name("modes", english, &format!("mode.{name}"))?,
                None => name.clone(),
            };
            // `enabled: false` prefixes the mode header; true renders with no
            // prefix (only false is source-backed in the corpus, #86).
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
    pub(crate) fn emit_heroes(&mut self, teams: &[SettingsNode]) -> Result<()> {
        let heroes_keyword = self.setting_name("namespaces", "heroes", "namespace.heroes")?;
        self.line(1, &format!("{heroes_keyword} {{"))?;
        for team in teams {
            let SettingsNode::Group { name, children, .. } = team else {
                return Err(WorkshopError::malformed(
                    "team entries must be groups",
                    team.span(),
                ));
            };
            let english = table::team_name(name).ok_or_else(|| {
                suggested(
                    format!("unknown team '{name}'"),
                    suggest::suggest(name, table::team_spellings()),
                    team.span(),
                )
            })?;
            let display = self.setting_name("teams", english, &format!("team.{name}"))?;
            self.line(2, &format!("{display} {{"))?;
            for member in children {
                match member {
                    SettingsNode::Group { name, children, .. } => {
                        let english = table::hero_name(name).ok_or_else(|| {
                            suggested(
                                format!("unknown hero '{name}'"),
                                suggest::suggest(name, table::hero_spellings()),
                                member.span(),
                            )
                        })?;
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
    pub(crate) fn settings_member(
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
        let entry = member::lookup(node, &full).map_err(|diagnostic| diagnostic.error)?;
        let display_name = if let (Some(hero), Some(key)) = (
            hero,
            full.last().and_then(|part| match part {
                PathPart::Part(key) => Some(*key),
                _ => None,
            }),
        ) {
            if let Some(name) = table::hero_setting_name(hero, key, self.locale.as_str()) {
                name.to_string()
            } else if !matches!(
                key,
                "enableAbility1" | "enableAbility2" | "enableAbility3" | "enableSecondaryFire"
            ) {
                if let Some(slot) = table::ability_slot_for_path(&full) {
                    self.gameplay_setting_name(hero, slot, &table::path_string(&full))?
                } else {
                    self.setting_name("labels", entry.workshop_name, &table::path_string(&full))?
                }
            } else {
                self.setting_name("labels", entry.workshop_name, &table::path_string(&full))?
            }
        } else if let (Some(hero), Some(slot)) = (hero, table::ability_slot_for_path(&full)) {
            self.gameplay_setting_name(hero, slot, &table::path_string(&full))?
        } else {
            self.setting_name("labels", entry.workshop_name, &table::path_string(&full))?
        };
        match member::accept(node, entry).map_err(|diagnostic| diagnostic.error)? {
            Member::Flag => self.line(level, &display_name)?,
            Member::String(value) => self.line(
                level,
                &format!("{}: \"{}\"", display_name, escape_settings_string(value)),
            )?,
            Member::Enum {
                domain,
                value,
                english,
            } => {
                let display =
                    self.setting_name("enums", english, &format!("enum.{domain}.{value}"))?;
                self.line(level, &format!("{display_name}: {display}"))?;
            }
            Member::Number(value) => self.line(
                level,
                &format!(
                    "{display_name}: {}",
                    crate::format::format_setting_number(value)
                ),
            )?,
            Member::Percent(value) => self.line(
                level,
                &format!(
                    "{display_name}: {}%",
                    crate::format::format_setting_number(value)
                ),
            )?,
            Member::Bool(value) => {
                let rendered = self.setting_name(
                    "tokens",
                    if value { "On" } else { "Off" },
                    if value { "token.on" } else { "token.off" },
                )?;
                self.line(level, &format!("{display_name}: {rendered}"))?;
            }
            Member::YesNo(value) => {
                let rendered = self.setting_name(
                    "tokens",
                    if value { "Yes" } else { "No" },
                    if value { "token.yes" } else { "token.no" },
                )?;
                self.line(level, &format!("{display_name}: {rendered}"))?;
            }
            Member::List { elements, kind } => {
                self.line(level, &format!("{display_name} {{"))?;
                for element in elements {
                    let english = kind
                        .resolve(element, name)
                        .map_err(|diagnostic| diagnostic.error)?;
                    let display = self.setting_name(
                        kind.section(),
                        english,
                        &format!("{}.{}.name", kind.label(), element.value),
                    )?;
                    self.line(level + 1, &display)?;
                }
                self.line(level, "}")?;
            }
        }
        Ok(())
    }

    pub(crate) fn emit_opaque_group(
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
    pub(crate) fn gameplay_setting_name(
        &mut self,
        hero: &str,
        slot: &str,
        id: &str,
    ) -> Result<String> {
        let resolve = |locale: &Locale| {
            crate::gameplay::data::builtin_ref()
                .ok()
                .and_then(|catalog| {
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
    pub(crate) fn setting_name(
        &mut self,
        section: &str,
        english: &str,
        id: &str,
    ) -> Result<String> {
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
}

/// A `Malformed` rejection whose message names the canonical suggestion when
/// one was identified — the same rendering `check_emission` produces.
fn suggested(message: String, suggestion: Option<String>, span: Option<Span>) -> WorkshopError {
    WorkshopError::malformed(
        suggest::with_suggestion_text(message, suggestion.as_deref()),
        span,
    )
}
