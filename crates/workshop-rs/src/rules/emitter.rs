use crate::output::emitter::*;

impl<'a> EmitContext<'a> {
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
            wir::Event::Subroutine { subroutine, .. } => {
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
                let condition_value = condition.value;
                // Reference normalization: comparison conditions render
                // infix; other conditions render as `value == True`.
                let comparison = match self
                    .program
                    .values
                    .get(condition_value)
                    .map(|node| &node.value)
                {
                    Some(wir::Value::Call { name, args })
                        if wir::is_comparison_operator(name) && args.len() == 2 =>
                    {
                        Some((name, args))
                    }
                    _ => None,
                };
                let text = if let Some((name, args)) = comparison {
                    let mut text = self.value_text(args[0])?;
                    let operator = self.spelling(Kind::Operator, name)?;
                    write!(text, " {operator} ").unwrap();
                    text.push_str(&self.value_text(args[1])?);
                    text
                } else {
                    let mut text = self.value_text(condition_value)?;
                    let true_spelling = self.spelling(Kind::Value, "true")?;
                    write!(text, " == {true_spelling}").unwrap();
                    text
                };
                let prefix = if condition.disabled {
                    format!("{} ", self.structural("disabled")?)
                } else {
                    String::new()
                };
                self.line(2, &format!("{prefix}{text};"))?;
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

    pub(crate) fn global_name(&self, id: wir::GlobalVarId) -> Result<&'a str> {
        self.variable_name(&self.program.global_variables, id, "global variable")
    }

    pub(crate) fn player_name(&self, id: wir::PlayerVarId) -> Result<&'a str> {
        self.variable_name(&self.program.player_variables, id, "player variable")
    }

    fn variable_name(
        &self,
        table: &'a crate::core::arena::Arena<wir::WorkshopVariable>,
        id: crate::core::ids::Id<wir::WorkshopVariable>,
        kind: &'static str,
    ) -> Result<&'a str> {
        table
            .get(id)
            .map(|variable| variable.name.as_str())
            .ok_or_else(|| WorkshopError::Unknown {
                kind,
                spelling: format!("<{id}>"),
                locale: self.locale.clone(),
                span: None,
            })
    }
}
