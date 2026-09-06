// Parser behavior owned by the Workshop actions domain.

use super::*;

impl Parser<'_> {
    pub(super) fn conditions_section(&mut self) -> Result<Vec<wir::ValueId>> {
        self.expect_keyword("conditions")?;
        self.expect(TokenKind::LBrace, "expected '{' after 'conditions'")?;
        let mut conditions = Vec::new();
        loop {
            match self.peek() {
                Some(Token {
                    kind: TokenKind::RBrace,
                    ..
                }) => {
                    self.pos += 1;
                    break;
                }
                Some(Token {
                    kind: TokenKind::Semi,
                    ..
                }) => {
                    self.pos += 1;
                }
                Some(_) => {
                    if matches!(
                        self.peek(),
                        Some(Token {
                            kind: TokenKind::String(_),
                            ..
                        })
                    ) {
                        self.pos += 1;
                        continue;
                    }
                    if let Some(Token {
                        kind: TokenKind::Word(word),
                        ..
                    }) = self.peek()
                    {
                        if self.settings_name_matches("tokens", "disabled", &word) {
                            self.pos += 1;
                            let _disabled_condition = self.value()?;
                            self.expect(TokenKind::Semi, "expected ';' after condition")?;
                            continue;
                        }
                    }
                    let condition = self.value()?;
                    self.expect(TokenKind::Semi, "expected ';' after condition")?;
                    conditions.push(condition);
                }
                None => {
                    return Err(self.malformed("unexpected end of input in conditions", self.eof()));
                }
            }
        }
        Ok(conditions)
    }

    pub(super) fn actions_section(&mut self) -> Result<Vec<wir::ActionId>> {
        self.expect_keyword("actions")?;
        self.expect(TokenKind::LBrace, "expected '{' after 'actions'")?;
        let mut all_actions = Vec::new();
        loop {
            let (actions, stop) = self.actions_until_end()?;
            all_actions.extend(actions);
            if stop == Stop::SectionClosed {
                return Ok(all_actions);
            }
            // Some exported Workshop artifacts retain an unmatched structural
            // marker after source-side pruning. Preserve it as an opaque raw
            // action so the source remains visible instead of silently
            // discarding it or rejecting the whole project.
            all_actions.push(self.opaque_action()?);
        }
    }

    /// Parse actions until a structural `else`/`elseIf`/`end` terminator
    /// (not consumed; the token position is preserved) or the enclosing `}`
    /// (consumed). Returns where the parse stopped.
    pub(super) fn actions_until_end(&mut self) -> Result<(Vec<wir::ActionId>, Stop)> {
        let mut actions = Vec::new();
        loop {
            match self.peek() {
                Some(Token {
                    kind: TokenKind::RBrace,
                    ..
                }) => {
                    self.pos += 1;
                    return Ok((actions, Stop::SectionClosed));
                }
                Some(Token {
                    kind: TokenKind::Word(_),
                    ..
                }) => {
                    if let Some(action) = self.assignment_action()? {
                        actions.push(action);
                        continue;
                    }
                    let saved = self.pos;
                    let (phrase, start, end) = self.phrase()?;
                    if let Some(rest) = self.disabled_action_rest(&phrase) {
                        if let Some(structural) = self.resolve_entry(Kind::Structural, rest) {
                            match structural.id.as_str() {
                                "while" => {
                                    actions.push(self.while_group()?);
                                    continue;
                                }
                                "if" => {
                                    actions.push(self.if_group()?);
                                    continue;
                                }
                                _ => {}
                            }
                        }
                        if self.resolve_entry(Kind::Action, rest).is_some() {
                            let canonical = rest;
                            actions.push(self.action_call_from_phrase(
                                canonical.to_string(),
                                start,
                                end,
                            )?);
                            continue;
                        }
                    }
                    if canonical_keyword(&phrase) == "disabled"
                        && matches!(
                            self.peek(),
                            Some(Token {
                                kind: TokenKind::Word(word),
                                ..
                            }) if canonical_keyword(&word) == "While"
                        )
                    {
                        self.pos += 1;
                        actions.push(self.while_group()?);
                        continue;
                    }
                    match self.canonical_keyword(&phrase).as_str() {
                        "end" => {
                            self.pos = saved;
                            return Ok((actions, Stop::End));
                        }
                        "elseIf" => {
                            self.pos = saved;
                            return Ok((actions, Stop::ElseIf));
                        }
                        "else" => {
                            self.pos = saved;
                            return Ok((actions, Stop::Else));
                        }
                        "if" => actions.push(self.if_group()?),
                        "forGlobalVariable" => actions.push(self.for_group()?),
                        "forPlayerVariable" => actions.push(self.for_player_group()?),
                        "while" => actions.push(self.while_group()?),
                        "Loop" => actions.push(self.action_call_from_phrase(phrase, start, end)?),
                        "Loop If Condition Is True" => {
                            actions.push(self.action_call_from_phrase(phrase, start, end)?)
                        }
                        "Global" | "Event Player" => {
                            self.pos = saved;
                            actions.push(self.opaque_action()?);
                        }
                        _ => {
                            if self.line_has_assignment() {
                                self.pos = saved;
                                actions.push(self.opaque_action()?);
                            } else {
                                actions.push(self.action_call_from_phrase(phrase, start, end)?);
                            }
                        }
                    }
                }
                Some(Token {
                    kind: TokenKind::String(_),
                    ..
                }) => {
                    // Raw Workshop permits standalone quoted annotations in
                    // generated action blocks. They are inert source text,
                    // not executable actions.
                    self.pos += 1;
                }
                Some(token) => {
                    let saved = self.pos;
                    if let Some(action) = self.member_assignment_action(saved, token.start)? {
                        actions.push(action);
                    } else {
                        return Err(self.malformed("expected an action", &token));
                    }
                }
                None => {
                    return Err(self.malformed("unexpected end of input in actions", self.eof()));
                }
            }
        }
    }

    /// Return the action spelling after the locale-declared disabled
    /// modifier. The modifier itself is settings/catalog data, not a parser
    /// branch for a fixed pair of client locales.
    pub(super) fn disabled_action_rest<'a>(&self, phrase: &'a str) -> Option<&'a str> {
        if let Some(rest) = phrase.strip_prefix("disabled ") {
            return Some(rest);
        }
        let localized = table::localized_name(self.locale.as_str(), "tokens", "disabled")?;
        phrase.strip_prefix(localized)?.strip_prefix(' ')
    }

    pub(super) fn assignment_action(&mut self) -> Result<Option<wir::ActionId>> {
        let saved = self.pos;
        let Some(Token {
            kind: TokenKind::Word(first),
            start,
            ..
        }) = self.peek()
        else {
            return Ok(None);
        };

        if matches!(canonical_keyword(&first), "Global" | "global") {
            self.pos += 1;
            self.expect(TokenKind::Dot, "expected '.' after 'Global'")?;
            let (name, _, target_end) = self.phrase()?;
            if matches!(
                self.peek().map(|token| token.kind),
                Some(TokenKind::LBracket)
            ) {
                self.pos += 1;
                let index = self.value()?;
                self.expect(
                    TokenKind::RBracket,
                    "expected ']' after global variable index",
                )?;
                let operator = self.assignment_operator()?.ok_or_else(|| {
                    self.malformed(
                        "expected assignment after global variable index",
                        self.peek().as_ref().unwrap_or(self.eof()),
                    )
                })?;
                let variable = self.global_by_name(&name)?;
                let target = self.target.values.push(ValueNode::new(
                    Value::GlobalVariable(variable),
                    Some(Span::new(self.file(), start, target_end)),
                ));
                let value = self.value()?;
                self.expect(TokenKind::Semi, "expected ';' after indexed assignment")?;
                return Ok(Some(self.indexed_assignment_action(
                    true, target, index, operator, value, start,
                )));
            }
            let Some(operator) = self.assignment_operator()? else {
                return self.member_assignment_action(saved, start);
            };
            let variable = self.global_by_name(&name)?;
            let value = self.value()?;
            let value = if let AssignmentOperator::Modify(op) = &operator {
                self.normalize_modify_value(*op, value)
            } else {
                value
            };
            self.expect(TokenKind::Semi, "expected ';' after assignment")?;
            let span = Some(Span::new(self.file(), start, self.previous_span().1));
            let target_span = Some(Span::new(self.file(), start, target_end));
            return Ok(Some(self.target.actions.push(match operator {
                AssignmentOperator::Set => Action::SetGlobalVariable {
                    variable,
                    value,
                    span,
                    target_span,
                },
                AssignmentOperator::Modify(op) => Action::ModifyGlobalVariable {
                    variable,
                    op,
                    value,
                    span,
                    target_span,
                },
            })));
        }

        let first_canonical = canonical_keyword(&first);
        let is_event_player = first_canonical == "Event Player"
            || (matches!(first_canonical, "Event" | "event")
                && matches!(
                    self.peek_at(1).map(|token| token.kind),
                    Some(TokenKind::Word(word))
                        if matches!(canonical_keyword(&word), "Player" | "player")
                ));
        if !is_event_player {
            // Object/member assignments use the same value grammar as member
            // reads (`receiver.member` and `receiver.member[index]`). Keep
            // this native member-assignment form distinct from catalog actions: the
            // receiver and member are dynamic Workshop values, not a builtin
            // identity. Global and Event Player assignments are handled by
            // their dedicated variable paths above and below.
            if !matches!(first_canonical, "Event" | "event") {
                return self.member_assignment_action(saved, start);
            }
            return Ok(None);
        }

        if first_canonical == "Event Player" {
            self.pos += 1;
        } else {
            self.pos += 2;
        }
        let event_player = self.target.values.push(ValueNode::new(
            Value::EventPlayer,
            Some(Span::new(self.file(), start, self.previous_span().1)),
        ));
        self.expect(TokenKind::Dot, "expected '.' after 'Event Player'")?;
        let (name, target_start, target_end) = self.phrase()?;
        let variable = self.player_by_name(&name)?;
        if matches!(
            self.peek().map(|token| token.kind),
            Some(TokenKind::LBracket)
        ) {
            self.pos += 1;
            let index = self.value()?;
            self.expect(
                TokenKind::RBracket,
                "expected ']' after player variable index",
            )?;
            let operator = self.assignment_operator()?.ok_or_else(|| {
                self.malformed(
                    "expected assignment after player variable index",
                    self.peek().as_ref().unwrap_or(self.eof()),
                )
            })?;
            let value = self.value()?;
            self.expect(TokenKind::Semi, "expected ';' after indexed assignment")?;
            let variable_value = self.target.values.push(ValueNode::new(
                Value::PlayerVariable {
                    player: event_player,
                    variable,
                },
                Some(Span::new(self.file(), target_start, target_end)),
            ));
            return Ok(Some(self.indexed_assignment_action(
                false,
                variable_value,
                index,
                operator,
                value,
                start,
            )));
        }
        let Some(operator) = self.assignment_operator()? else {
            return self.member_assignment_action(saved, start);
        };
        let value = self.value()?;
        let value = if let AssignmentOperator::Modify(op) = &operator {
            self.normalize_modify_value(*op, value)
        } else {
            value
        };
        self.expect(TokenKind::Semi, "expected ';' after assignment")?;
        let span = Some(Span::new(self.file(), start, self.previous_span().1));
        let target_span = Some(Span::new(self.file(), target_start, target_end));
        Ok(Some(self.target.actions.push(match operator {
            AssignmentOperator::Set => Action::SetPlayerVariable {
                player: event_player,
                variable,
                value,
                span,
                target_span,
            },
            AssignmentOperator::Modify(op) => Action::ModifyPlayerVariable {
                player: event_player,
                variable,
                op,
                value,
                span,
                target_span,
            },
        })))
    }

    pub(super) fn member_assignment_action(
        &mut self,
        saved: usize,
        start: Position,
    ) -> Result<Option<wir::ActionId>> {
        self.pos = saved;
        if !self.line_has_assignment() {
            return Ok(None);
        }
        let target = self.value()?;
        let Some(operator) = self.assignment_operator()? else {
            self.pos = saved;
            return Ok(None);
        };
        let value = self.value()?;
        self.expect(TokenKind::Semi, "expected ';' after member assignment")?;
        let op = match operator {
            AssignmentOperator::Set => None,
            AssignmentOperator::Modify(op) => Some(op),
        };
        Ok(Some(self.target.actions.push(Action::AssignMember {
            target,
            op,
            value,
            span: Some(Span::new(self.file(), start, self.previous_span().1)),
        })))
    }

    pub(super) fn indexed_assignment_action(
        &mut self,
        global: bool,
        variable: wir::ValueId,
        index: wir::ValueId,
        operator: AssignmentOperator,
        value: wir::ValueId,
        start: Position,
    ) -> wir::ActionId {
        let name = match &operator {
            AssignmentOperator::Set => {
                if global {
                    "setGlobalVariableAtIndex"
                } else {
                    "setPlayerVariableAtIndex"
                }
            }
            AssignmentOperator::Modify(_) => {
                if global {
                    "modifyGlobalVariableAtIndex"
                } else {
                    "modifyPlayerVariableAtIndex"
                }
            }
        };
        let index = self.normalize_contextual_argument(name, 1, index);
        let (value, modify_op) = match operator {
            AssignmentOperator::Set => (value, None),
            AssignmentOperator::Modify(op) => (self.normalize_modify_value(op, value), Some(op)),
        };
        let args = match modify_op {
            None => vec![variable, index, value],
            Some(op) => vec![
                variable,
                index,
                self.target.values.push(ValueNode::new(
                    Value::Call {
                        name: op.catalog_id().to_string(),
                        args: Vec::new(),
                    },
                    None,
                )),
                value,
            ],
        };
        self.target.actions.push(Action::Call {
            name: name.to_string(),
            args,
            span: Some(Span::new(self.file(), start, self.previous_span().1)),
        })
    }

    pub(super) fn assignment_operator(&mut self) -> Result<Option<AssignmentOperator>> {
        let Some(token) = self.peek() else {
            return Ok(None);
        };
        if let TokenKind::Word(word) = &token.kind {
            let op = match word.as_str() {
                "min" => ModifyOp::Min,
                "max" => ModifyOp::Max,
                _ => {
                    if matches!(self.peek_at(1).map(|token| token.kind), Some(TokenKind::Op(equal)) if equal == "=")
                    {
                        return Err(WorkshopError::Unsupported {
                            message: format!("unsupported assignment operator '{word}='"),
                            span: Some(Span::new(
                                self.file(),
                                token.start,
                                self.peek_at(1).unwrap().end,
                            )),
                        });
                    }
                    return Ok(None);
                }
            };
            if !matches!(self.peek_at(1).map(|token| token.kind), Some(TokenKind::Op(equal)) if equal == "=")
            {
                return Ok(None);
            }
            self.pos += 2;
            return Ok(Some(AssignmentOperator::Modify(op)));
        }
        let operator = match &token.kind {
            TokenKind::Op(operator) => operator.clone(),
            _ => return Ok(None),
        };
        if operator == "=" {
            self.pos += 1;
            return Ok(Some(AssignmentOperator::Set));
        }
        let op = match operator.as_str() {
            "+" => ModifyOp::Add,
            "-" => ModifyOp::Subtract,
            "*" => ModifyOp::Multiply,
            "/" => ModifyOp::Divide,
            "%" => ModifyOp::Modulo,
            _ => return Ok(None),
        };
        if !matches!(self.peek_at(1).map(|token| token.kind), Some(TokenKind::Op(equal)) if equal == "=")
        {
            return Ok(None);
        }
        self.pos += 2;
        Ok(Some(AssignmentOperator::Modify(op)))
    }

    pub(super) fn opaque_action(&mut self) -> Result<wir::ActionId> {
        let start = self
            .peek()
            .map(|token| token.start)
            .unwrap_or(self.eof().start);
        while let Some(token) = self.peek() {
            self.pos += 1;
            if matches!(token.kind, TokenKind::Semi) {
                break;
            }
        }
        Ok(self.target.actions.push(Action::Call {
            name: "rawWorkshopAction".to_string(),
            args: Vec::new(),
            span: Some(Span::new(self.file(), start, self.previous_span().1)),
        }))
    }

    pub(super) fn if_group(&mut self) -> Result<wir::ActionId> {
        let start = self.previous_span().0;
        self.expect(TokenKind::LParen, "expected '(' after 'If'")?;
        let condition = self.value()?;
        self.expect(TokenKind::RParen, "expected ')' after If condition")?;
        self.expect(TokenKind::Semi, "expected ';' after If condition")?;

        let mut branches = Vec::new();
        let mut stop = {
            let (body, stop) = self.actions_until_end()?;
            branches.push(wir::IfBranch { condition, body });
            stop
        };

        let mut else_body = None;
        loop {
            match stop {
                Stop::End => {
                    self.consume_phrase("End")?;
                    self.expect(TokenKind::Semi, "expected ';' after 'End'")?;
                    break;
                }
                Stop::ElseIf => {
                    self.consume_phrase("Else If")?;
                    self.expect(TokenKind::LParen, "expected '(' after 'Else If'")?;
                    let condition = self.value()?;
                    self.expect(TokenKind::RParen, "expected ')' after Else If condition")?;
                    self.expect(TokenKind::Semi, "expected ';' after Else If condition")?;
                    let (body, next) = self.actions_until_end()?;
                    branches.push(wir::IfBranch { condition, body });
                    stop = next;
                }
                Stop::Else => {
                    self.consume_phrase("Else")?;
                    self.expect(TokenKind::Semi, "expected ';' after 'Else'")?;
                    let (body, next) = self.actions_until_end()?;
                    else_body = Some(body);
                    stop = next;
                }
                Stop::SectionClosed => {
                    // The oracle closes a rule-final if/if-else with the
                    // enclosing actions-section `}` (no trailing `End;`,
                    // #87). Rewind so the enclosing actions section consumes
                    // that `}` and the rule's own `}` stays intact.
                    self.pos -= 1;
                    break;
                }
            }
        }
        let end_span = self.previous_span();
        let action = Action::If {
            branches,
            else_body,
            span: Some(Span::new(self.file(), start, end_span.1)),
        };
        Ok(self.target.actions.push(action))
    }

    pub(super) fn for_group(&mut self) -> Result<wir::ActionId> {
        let start = self.previous_span().0;
        self.expect(
            TokenKind::LParen,
            "expected '(' after 'For Global Variable'",
        )?;
        let (name, _, _) = self.phrase()?;
        let variable = self.global_by_name(&name)?;
        self.expect(TokenKind::Comma, "expected ',' after loop variable")?;
        let start_value = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after start")?;
        let stop = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after stop")?;
        let step = self.value()?;
        self.expect(TokenKind::RParen, "expected ')' after For bounds")?;
        self.expect(TokenKind::Semi, "expected ';' after For Global Variable")?;
        let (body, loop_stop) = self.actions_until_end()?;
        if loop_stop != Stop::End {
            return Err(self.malformed(
                "'For Global Variable' requires a matching 'End'",
                self.previous(),
            ));
        }
        self.consume_phrase("End")?;
        self.expect(TokenKind::Semi, "expected ';' after 'End'")?;
        let end_span = self.previous_span();
        let action = Action::ForGlobalVariable {
            variable,
            start: start_value,
            stop,
            step,
            body,
            span: Some(Span::new(self.file(), start, end_span.1)),
            target_span: None,
        };
        Ok(self.target.actions.push(action))
    }

    pub(super) fn while_group(&mut self) -> Result<wir::ActionId> {
        let start = self.previous_span().0;
        self.expect(TokenKind::LParen, "expected '(' after 'While'")?;
        let condition = self.value()?;
        self.expect(TokenKind::RParen, "expected ')' after While condition")?;
        self.expect(TokenKind::Semi, "expected ';' after While condition")?;
        let (body, stop) = self.actions_until_end()?;
        if stop != Stop::End {
            return Err(self.malformed("'While' requires a matching 'End'", self.previous()));
        }
        self.consume_phrase("End")?;
        self.expect(TokenKind::Semi, "expected ';' after 'End'")?;
        let end_span = self.previous_span();
        let action = Action::While {
            condition,
            body,
            span: Some(Span::new(self.file(), start, end_span.1)),
        };
        Ok(self.target.actions.push(action))
    }

    /// `For Player Variable(player, name, start, stop, step)` — the
    /// reference's per-player loop form (parsed from pinned reference
    /// evidence; the differential gate normalizes it to the declared global
    /// form, #119).
    pub(super) fn for_player_group(&mut self) -> Result<wir::ActionId> {
        let start = self.previous_span().0;
        self.expect(
            TokenKind::LParen,
            "expected '(' after 'For Player Variable'",
        )?;
        let player = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after loop player")?;
        let (name, _, _) = self.phrase()?;
        let variable = self.player_by_name(&name)?;
        self.expect(TokenKind::Comma, "expected ',' after loop variable")?;
        let start_value = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after start")?;
        let stop = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after stop")?;
        let step = self.value()?;
        self.expect(TokenKind::RParen, "expected ')' after For bounds")?;
        self.expect(TokenKind::Semi, "expected ';' after For Player Variable")?;
        let (body, loop_stop) = self.actions_until_end()?;
        if loop_stop != Stop::End {
            return Err(self.malformed(
                "'For Player Variable' requires a matching 'End'",
                self.previous(),
            ));
        }
        self.consume_phrase("End")?;
        self.expect(TokenKind::Semi, "expected ';' after 'End'")?;
        let end_span = self.previous_span();
        let action = Action::ForPlayerVariable {
            player,
            variable,
            start: start_value,
            stop,
            step,
            body,
            span: Some(Span::new(self.file(), start, end_span.1)),
        };
        Ok(self.target.actions.push(action))
    }

    pub(super) fn action_call_from_phrase(
        &mut self,
        phrase: String,
        start: Position,
        end: Position,
    ) -> Result<wir::ActionId> {
        match self
            .catalog
            .resolve(Kind::Structural, &self.locale, &phrase)
        {
            Some(entry) => match entry.id.as_str() {
                "setGlobalVariable" => {
                    self.expect(
                        TokenKind::LParen,
                        "expected '(' after 'Set Global Variable'",
                    )?;
                    let (name, _, _) = self.phrase()?;
                    let variable = self.global_by_name(&name)?;
                    self.expect(TokenKind::Comma, "expected ',' after variable")?;
                    let value = self.value()?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect(TokenKind::Semi, "expected ';'")?;
                    Ok(self.target.actions.push(Action::SetGlobalVariable {
                        variable,
                        value,
                        span: Some(Span::new(self.file(), start, end)),
                        target_span: None,
                    }))
                }
                "modifyGlobalVariable" => {
                    self.expect(TokenKind::LParen, "expected '('")?;
                    let (name, _, _) = self.phrase()?;
                    let variable = self.global_by_name(&name)?;
                    self.expect(TokenKind::Comma, "expected ',' after variable")?;
                    let op = self.modify_op()?;
                    self.expect(TokenKind::Comma, "expected ',' after modify operator")?;
                    let value = self.value()?;
                    let value = self.normalize_modify_value(op, value);
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect(TokenKind::Semi, "expected ';'")?;
                    Ok(self.target.actions.push(Action::ModifyGlobalVariable {
                        variable,
                        op,
                        value,
                        span: Some(Span::new(self.file(), start, end)),
                        target_span: None,
                    }))
                }
                "setPlayerVariable" => {
                    self.expect(TokenKind::LParen, "expected '('")?;
                    let player = self.value()?;
                    self.expect(TokenKind::Comma, "expected ',' after player")?;
                    let (name, _, _) = self.phrase()?;
                    let variable = self.player_by_name(&name)?;
                    self.expect(TokenKind::Comma, "expected ',' after variable")?;
                    let value = self.value()?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect(TokenKind::Semi, "expected ';'")?;
                    Ok(self.target.actions.push(Action::SetPlayerVariable {
                        player,
                        variable,
                        value,
                        span: Some(Span::new(self.file(), start, end)),
                        target_span: None,
                    }))
                }
                "modifyPlayerVariable" => {
                    self.expect(TokenKind::LParen, "expected '('")?;
                    let player = self.value()?;
                    self.expect(TokenKind::Comma, "expected ',' after player")?;
                    let (name, _, _) = self.phrase()?;
                    let variable = self.player_by_name(&name)?;
                    self.expect(TokenKind::Comma, "expected ',' after variable")?;
                    let op = self.modify_op()?;
                    self.expect(TokenKind::Comma, "expected ',' after modify operator")?;
                    let value = self.value()?;
                    let value = self.normalize_modify_value(op, value);
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect(TokenKind::Semi, "expected ';'")?;
                    Ok(self.target.actions.push(Action::ModifyPlayerVariable {
                        player,
                        variable,
                        op,
                        value,
                        span: Some(Span::new(self.file(), start, end)),
                        target_span: None,
                    }))
                }
                "forGlobalVariable" => self.for_group(),
                "forPlayerVariable" => self.for_player_group(),
                "callSubroutine" => {
                    self.expect(TokenKind::LParen, "expected '('")?;
                    let (name, _, _) = self.phrase()?;
                    let subroutine = self.subroutine_by_name(&name)?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect(TokenKind::Semi, "expected ';'")?;
                    Ok(self.target.actions.push(Action::CallSubroutine {
                        subroutine,
                        span: Some(Span::new(self.file(), start, end)),
                        callee_span: None,
                    }))
                }
                other => Err(WorkshopError::Unsupported {
                    message: format!(
                        "structural action '{other}' is not supported in action position"
                    ),
                    span: Some(Span::new(self.file(), start, end)),
                }),
            },
            None => {
                // Generic action call; the argument list is optional.
                let Some(action) = self
                    .resolve_entry(Kind::Action, &phrase)
                    .or_else(|| self.resolve_entry(Kind::Action, &format!("{phrase} ")))
                    .or_else(|| {
                        let alias = match phrase.as_str() {
                            "Set Player Allowed Heroes" => "Set Allowed Heroes",
                            "设置技能充能" => "设置终极技能充能",
                            _ => return None,
                        };
                        self.resolve_entry(Kind::Action, alias)
                    })
                else {
                    return Err(WorkshopError::Unknown {
                        kind: "action",
                        spelling: phrase,
                        locale: self.locale.clone(),
                        span: Some(Span::new(self.file(), start, end)),
                    });
                };
                // The player-variable chase forms lay the variable out as
                // `player, name` leading arguments (the pinned oracle's
                // spelling, #110); the name is not a value, so the action is
                // parsed like `Set Player Variable` and reconstructed as the
                // canonical `chaseAtRate`/`chaseOverTime` call with a
                // player-variable first argument (the shape the emitter
                // dispatches on).
                match action.id.as_str() {
                    "chasePlayerVariableAtRate" | "chasePlayerVariableOverTime" => {
                        self.expect(TokenKind::LParen, "expected '('")?;
                        let player = self.value()?;
                        self.expect(TokenKind::Comma, "expected ',' after player")?;
                        let (name, _, _) = self.phrase()?;
                        let variable = self.player_by_name(&name)?;
                        let mut args = Vec::with_capacity(4);
                        args.push(self.target.values.push(wir::ValueNode::new(
                            wir::Value::PlayerVariable { player, variable },
                            None,
                        )));
                        // The remaining arguments sit at overall argument
                        // indexes 2.. (player and name consumed indexes 0-1),
                        // so the signature context resolves their expected
                        // domains at the shifted positions.
                        let mut arg_index = 2usize;
                        loop {
                            match self.peek() {
                                Some(Token {
                                    kind: TokenKind::RParen,
                                    ..
                                }) => break,
                                Some(Token {
                                    kind: TokenKind::Comma,
                                    ..
                                }) => {
                                    self.pos += 1;
                                }
                                _ => {}
                            }
                            let saved = self.expected_domain;
                            self.expected_domain =
                                self.context.expected_domain(action.id.as_str(), arg_index);
                            let arg = self.value()?;
                            self.expected_domain = saved;
                            args.push(arg);
                            arg_index += 1;
                        }
                        self.expect(TokenKind::RParen, "expected ')'")?;
                        self.expect(TokenKind::Semi, "expected ';' after action")?;
                        let canonical = if action.id == "chasePlayerVariableAtRate" {
                            "chaseAtRate"
                        } else {
                            "chaseOverTime"
                        };
                        return Ok(self.target.actions.push(Action::Call {
                            name: canonical.to_string(),
                            args,
                            span: Some(Span::new(self.file(), start, end)),
                        }));
                    }
                    "startRule" => {
                        self.expect(TokenKind::LParen, "expected '('")?;
                        let (name, _, _) = self.phrase()?;
                        let subroutine = self.subroutine_by_name(&name)?;
                        self.expect(TokenKind::Comma, "expected ',' after subroutine")?;
                        let saved = self.expected_domain;
                        self.expected_domain = self.context.expected_domain(action.id.as_str(), 1);
                        let behavior = self.value()?;
                        self.expected_domain = saved;
                        self.expect(TokenKind::RParen, "expected ')'")?;
                        self.expect(TokenKind::Semi, "expected ';' after action")?;
                        let subroutine_value = self
                            .target
                            .values
                            .push(ValueNode::new(Value::Subroutine(subroutine), None));
                        return Ok(self.target.actions.push(Action::Call {
                            name: action.id.clone(),
                            args: vec![subroutine_value, behavior],
                            span: Some(Span::new(self.file(), start, end)),
                        }));
                    }
                    "stopChasingPlayerVariable" => {
                        self.expect(TokenKind::LParen, "expected '('")?;
                        let player = self.value()?;
                        self.expect(TokenKind::Comma, "expected ',' after player")?;
                        let (name, _, _) = self.phrase()?;
                        let variable = self.player_by_name(&name)?;
                        self.expect(TokenKind::RParen, "expected ')'")?;
                        self.expect(TokenKind::Semi, "expected ';' after action")?;
                        let player_variable = self.target.values.push(ValueNode::new(
                            Value::PlayerVariable { player, variable },
                            None,
                        ));
                        return Ok(self.target.actions.push(Action::Call {
                            name: action.id.clone(),
                            args: vec![player_variable],
                            span: Some(Span::new(self.file(), start, end)),
                        }));
                    }
                    _ => {}
                }
                let args = if let Some(Token {
                    kind: TokenKind::LParen,
                    ..
                }) = self.peek()
                {
                    self.pos += 1;
                    let args = self.value_args(action.id.as_str())?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    args
                } else {
                    Vec::new()
                };
                self.expect(TokenKind::Semi, "expected ';' after action")?;
                Ok(self.target.actions.push(Action::Call {
                    name: action.id.clone(),
                    args,
                    span: Some(Span::new(self.file(), start, end)),
                }))
            }
        }
    }

    pub(super) fn modify_op(&mut self) -> Result<ModifyOp> {
        let (phrase, start, end) = self.phrase()?;
        if phrase == "根据值从数组中移除" {
            return Ok(ModifyOp::RemoveFromArray);
        }
        if phrase == "根据索引从数组中移除" {
            return Ok(ModifyOp::RemoveFromArrayByIndex);
        }
        let entry = self
            .catalog
            .resolve(Kind::Operator, &self.locale, &phrase)
            .ok_or_else(|| WorkshopError::Unknown {
                kind: "modify operator",
                spelling: phrase.clone(),
                locale: self.locale.clone(),
                span: Some(Span::new(self.file(), start, end)),
            })?;
        let op = match entry.id.as_str() {
            "add" => ModifyOp::Add,
            "subtract" => ModifyOp::Subtract,
            "multiply" => ModifyOp::Multiply,
            "divide" => ModifyOp::Divide,
            "modulo" => ModifyOp::Modulo,
            "min" => ModifyOp::Min,
            "max" => ModifyOp::Max,
            "raiseToPower" => ModifyOp::RaiseToPower,
            "appendToArray" => ModifyOp::AppendToArray,
            "removeFromArray" | "removeFromArrayByValue" => ModifyOp::RemoveFromArray,
            "removeFromArrayByIndex" => ModifyOp::RemoveFromArrayByIndex,
            other => {
                return Err(WorkshopError::Unsupported {
                    message: format!("unsupported modify operator '{other}'"),
                    span: Some(Span::new(self.file(), start, end)),
                });
            }
        };
        Ok(op)
    }
}
