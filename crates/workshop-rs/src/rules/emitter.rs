// EmitContext behavior owned by the Workshop rules domain.

use crate::output::emitter::*;

impl EmitContext<'_> {
    pub(crate) fn rule(&mut self, rule: &wir::Rule) -> Result<()> {
        let disabled = if rule.disabled {
            format!("{} ", self.structural("disabled")?)
        } else {
            String::new()
        };
        let rule_keyword = self.structural("rule")?;
        self.line(
            0,
            &format!(
                "{disabled}{rule_keyword} (\"{}\") {{",
                escape_string(&rule.name)
            ),
        )?;
        let event = self.structural("event")?;
        self.line(1, &format!("{event} {{"))?;
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
            let conditions = self.structural("conditions")?;
            self.line(1, &format!("{conditions} {{"))?;
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
            let actions = self.structural("actions")?;
            self.line(1, &format!("{actions} {{"))?;
            for (index, action) in rule.actions.iter().enumerate() {
                let rule_final = index + 1 == rule.actions.len();
                self.action(*action, 2, rule_final)?;
            }
            self.line(1, "}")?;
        }
        self.line(0, "}")?;
        Ok(())
    }

    pub(crate) fn global_name(&self, id: wir::GlobalVarId) -> Result<String> {
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
    pub(crate) fn player_name(&self, id: wir::PlayerVarId) -> Result<String> {
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
}
