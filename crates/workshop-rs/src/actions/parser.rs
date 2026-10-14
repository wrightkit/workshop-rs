use crate::frontend::parser::*;

impl ParseContext<'_> {
    pub(crate) fn conditions_section(&mut self) -> Result<Vec<wir::Condition>> {
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
                    let disabled = self.consume_disabled();
                    self.condition_pending = true;
                    let value = self.value()?;
                    self.expect_semi("expected ';' after condition")?;
                    conditions.push(wir::Condition { value, disabled });
                }
                None => {
                    return Err(self.malformed("unexpected end of input in conditions", self.eof()));
                }
            }
        }
        Ok(conditions)
    }

    /// `player, name` — the leading arguments of the player-variable
    /// assignment/chase forms: a player value and a variable name phrase.
    /// Returns the player value, variable id, and the name span.
    fn player_variable_arg(&mut self) -> Result<(wir::ValueId, wir::PlayerVarId, Span)> {
        let player = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after player")?;
        let (name, name_start, name_end) = self.phrase()?;
        let variable = self.player_by_name(&name)?;
        Ok((
            player,
            variable,
            Span::new(self.file(), name_start, name_end),
        ))
    }

    /// `name` — the single argument of the global-variable forms.
    /// Returns the variable id and the name span.
    fn global_variable_arg(&mut self) -> Result<(wir::GlobalVarId, Span)> {
        let (name, name_start, name_end) = self.phrase()?;
        let variable = self.global_by_name(&name)?;
        Ok((variable, Span::new(self.file(), name_start, name_end)))
    }

    pub(crate) fn actions_section(&mut self) -> Result<Vec<wir::ActionId>> {
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
    pub(crate) fn actions_until_end(&mut self) -> Result<(Vec<wir::ActionId>, Stop)> {
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
                    if let Some(action) = self.disabled_assignment_action()? {
                        actions.push(action);
                        continue;
                    }
                    if let Some(action) = self.assignment_action()? {
                        actions.push(action);
                        continue;
                    }
                    let saved = self.pos;
                    let disabled = self.consume_disabled();
                    let (mut phrase, mut start, mut end) = self.action_spelling()?;
                    if disabled {
                        if let Some(structural) = self.resolve_entry(Kind::Structural, &phrase) {
                            match structural.id.as_str() {
                                "while" => {
                                    let group = self.while_group(self.previous_span().0)?;
                                    actions.push(self.disabled_action(group));
                                    continue;
                                }
                                "if" => {
                                    let group = self.if_group(self.previous_span().0)?;
                                    actions.push(self.disabled_action(group));
                                    continue;
                                }
                                _ => {
                                    let action =
                                        self.action_call_from_phrase(phrase, start, end)?;
                                    actions.push(self.disabled_action(action));
                                    continue;
                                }
                            }
                        }
                        if self.resolve_entry(Kind::Action, &phrase).is_some() {
                            let call = self.action_call_from_phrase(phrase, start, end)?;
                            actions.push(self.disabled_action(call));
                            continue;
                        }
                        // Not a recognized disabled form: rewind so the
                        // modifier stays part of the dispatched phrase.
                        self.pos = saved;
                        let reparsed = self.action_spelling()?;
                        phrase = reparsed.0;
                        start = reparsed.1;
                        end = reparsed.2;
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
                        "if" => actions.push(self.if_group(start)?),
                        "forGlobalVariable" => actions.push(self.for_group(start)?),
                        "forPlayerVariable" => actions.push(self.for_player_group(start)?),
                        "while" => actions.push(self.while_group(start)?),
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
                Some(Token {
                    kind: TokenKind::Number { .. },
                    ..
                }) => {
                    // A catalog action may open with a number (`1.
                    // Kabiliyeti Etkinleştir`, #448); the spelled text must
                    // resolve to a declared action or structural entry,
                    // otherwise the member-assignment path applies.
                    let saved = self.pos;
                    let (phrase, start, end) = self.action_spelling()?;
                    if self.resolve_entry(Kind::Action, &phrase).is_some()
                        || self.resolve_entry(Kind::Structural, &phrase).is_some()
                    {
                        actions.push(self.action_call_from_phrase(phrase, start, end)?);
                        continue;
                    }
                    self.pos = saved;
                    let token = self
                        .peek()
                        .expect("number token was peeked by the dispatch");
                    if let Some(action) = self.member_assignment_action(saved, token.start)? {
                        actions.push(action);
                    } else {
                        return Err(self.malformed("expected an action", &token));
                    }
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

    /// Parse `disabled <assignment>;`, restoring the position when the
    /// modifier is not followed by an assignment form.
    fn disabled_assignment_action(&mut self) -> Result<Option<wir::ActionId>> {
        let saved = self.pos;
        if !self.consume_disabled() {
            return Ok(None);
        }
        match self.assignment_action()? {
            Some(action) => Ok(Some(self.disabled_action(action))),
            None => {
                self.pos = saved;
                Ok(None)
            }
        }
    }

    /// An action-position spelling: the maximal word run, extended through
    /// an adjacent `.`/`:`/`!` tail when the joined text resolves to a
    /// declared action or structural entry (`Bekle:`, `Maks.`, #448). The
    /// tail is consumed only when the whole joined spelling resolves, so a
    /// `:`/`!` that belongs to syntax stays put.
    fn action_spelling(&mut self) -> Result<(String, Position, Position)> {
        let (phrase, start, end) = self.phrase()?;
        if let Some((joined, joined_end)) =
            self.extend_entry_spelling(&phrase, end, &[Kind::Action, Kind::Structural])
        {
            return Ok((joined, start, joined_end));
        }
        Ok((phrase, start, end))
    }

    /// Consume the `disabled` modifier if the cursor spells it: the
    /// structural keyword's catalog/fallback spellings (including multi-word
    /// and dotted forms like tr `devre dışı` and pl `wył.`) or the settings
    /// `tokens.disabled` name.
    pub(crate) fn consume_disabled(&mut self) -> bool {
        if self.keyword_prefix(&["disabled"]).is_some() {
            return true;
        }
        if let Some(Token {
            kind: TokenKind::Word(word),
            ..
        }) = self.peek()
        {
            if self.settings_name_matches("tokens", "disabled", &word) {
                self.pos += 1;
                return true;
            }
        }
        false
    }

    /// Wrap a parsed action in the `disabled` modifier.
    fn disabled_action(&mut self, action: wir::ActionId) -> wir::ActionId {
        let span = self.target.actions.get(action).and_then(Action::span);
        self.target.actions.push(Action::Disabled { action, span })
    }

    /// The token count of an `eventPlayer` value spelling at the cursor when
    /// it is followed by `.` — `Event Player` or any reviewed localized
    /// alias (fr `Joueur exécutant`, tr `Etkinlik Oyuncusu`, zh `事件玩家`).
    fn event_player_spelling_len(&self) -> Option<usize> {
        let mut words = Vec::new();
        let mut offset = 0;
        while let Some(token) = self.peek_at(offset) {
            match &token.kind {
                TokenKind::Word(word) => {
                    words.push(word.clone());
                    offset += 1;
                }
                TokenKind::Dot => break,
                _ => return None,
            }
        }
        if !matches!(
            self.peek_at(offset).map(|token| token.kind),
            Some(TokenKind::Dot)
        ) {
            return None;
        }
        self.resolve_entry(Kind::Value, &words.join(" "))
            .filter(|entry| entry.id == "eventPlayer")
            .map(|_| offset)
    }

    pub(crate) fn assignment_action(&mut self) -> Result<Option<wir::ActionId>> {
        let saved = self.pos;
        let Some(Token {
            kind: TokenKind::Word(first),
            start,
            ..
        }) = self.peek()
        else {
            return Ok(None);
        };

        if matches!(self.canonical_keyword(&first).as_str(), "Global" | "global") {
            self.pos += 1;
            self.expect(TokenKind::Dot, "expected '.' after 'Global'")?;
            let (name, name_start, target_end) = self.phrase()?;
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
                let target = self.target.values.push(
                    ValueNode::new(
                        Value::GlobalVariable(variable),
                        Some(Span::new(self.file(), start, target_end)),
                    )
                    .with_identifier(Some(Span::new(
                        self.file(),
                        name_start,
                        target_end,
                    ))),
                );
                let value = self.value()?;
                self.expect_semi("expected ';' after indexed assignment")?;
                return Ok(Some(self.indexed_assignment_action(
                    true, target, index, operator, value, start,
                )));
            }
            let Some(operator) = self.assignment_operator()? else {
                return self.member_assignment_action(saved, start);
            };
            let variable = self.global_by_name(&name)?;
            let value = self.value()?;
            self.expect_semi("expected ';' after assignment")?;
            let span = Some(Span::new(self.file(), start, self.previous_span().1));
            let target_span = Some(Span::new(self.file(), name_start, target_end));
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

        // An Event Player receiver may carry any catalog spelling, including
        // multi-word aliases (fr `Joueur exécutant`, tr `Etkinlik Oyuncusu`);
        // resolve the word run before `.` against the value catalog rather
        // than matching individual words.
        let Some(event_player_len) = self.event_player_spelling_len() else {
            // Object/member assignments use the same value grammar as member
            // reads (`receiver.member` and `receiver.member[index]`). Keep
            // this native member-assignment form distinct from catalog actions: the
            // receiver and member are dynamic Workshop values, not a builtin
            // identity. Global and Event Player assignments are handled by
            // their dedicated variable paths above and below.
            if !matches!(
                self.canonical_keyword(&first).as_str(),
                "Event" | "event" | "Event Player"
            ) {
                return self.member_assignment_action(saved, start);
            }
            return Ok(None);
        };
        self.pos += event_player_len;
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
            self.expect_semi("expected ';' after indexed assignment")?;
            let variable_value = self.target.values.push(
                ValueNode::new(
                    Value::PlayerVariable {
                        player: event_player,
                        variable,
                    },
                    Some(Span::new(self.file(), target_start, target_end)),
                )
                .with_identifier(Some(Span::new(
                    self.file(),
                    target_start,
                    target_end,
                ))),
            );
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
        self.expect_semi("expected ';' after assignment")?;
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

    pub(crate) fn member_assignment_action(
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
        self.expect_semi("expected ';' after member assignment")?;
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

    pub(crate) fn indexed_assignment_action(
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
        let (value, modify_op) = match operator {
            AssignmentOperator::Set => (value, None),
            AssignmentOperator::Modify(op) => (value, Some(op)),
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

    pub(crate) fn assignment_operator(&mut self) -> Result<Option<AssignmentOperator>> {
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
                        return Err(WorkshopError::unsupported(
                            format!("unsupported assignment operator '{word}='"),
                            Some(Span::new(
                                self.file(),
                                token.start,
                                self.peek_at(1).unwrap().end,
                            )),
                        ));
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

    pub(crate) fn opaque_action(&mut self) -> Result<wir::ActionId> {
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

    pub(crate) fn if_group(&mut self, start: Position) -> Result<wir::ActionId> {
        self.expect(TokenKind::LParen, "expected '(' after 'If'")?;
        self.condition_pending = true;
        let condition = self.value()?;
        self.expect(TokenKind::RParen, "expected ')' after If condition")?;
        self.expect_semi("expected ';' after If condition")?;

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
                    self.consume_keyword("end")?;
                    self.expect_semi("expected ';' after 'End'")?;
                    break;
                }
                Stop::ElseIf => {
                    self.consume_keyword("elseIf")?;
                    self.expect(TokenKind::LParen, "expected '(' after 'Else If'")?;
                    self.condition_pending = true;
                    let condition = self.value()?;
                    self.expect(TokenKind::RParen, "expected ')' after Else If condition")?;
                    self.expect_semi("expected ';' after Else If condition")?;
                    let (body, next) = self.actions_until_end()?;
                    branches.push(wir::IfBranch { condition, body });
                    stop = next;
                }
                Stop::Else => {
                    self.consume_keyword("else")?;
                    self.expect_semi("expected ';' after 'Else'")?;
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

    pub(crate) fn for_group(&mut self, start: Position) -> Result<wir::ActionId> {
        self.expect(
            TokenKind::LParen,
            "expected '(' after 'For Global Variable'",
        )?;
        let (variable, target_span) = self.global_variable_arg()?;
        self.expect(TokenKind::Comma, "expected ',' after loop variable")?;
        let start_value = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after start")?;
        let stop = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after stop")?;
        let step = self.value()?;
        self.expect(TokenKind::RParen, "expected ')' after For bounds")?;
        self.expect_semi("expected ';' after For Global Variable")?;
        let (body, end) =
            self.actions_until_matching_end("'For Global Variable' requires a matching 'End'")?;
        let action = Action::ForGlobalVariable {
            variable,
            start: start_value,
            stop,
            step,
            body,
            span: Some(Span::new(self.file(), start, end)),
            target_span: Some(target_span),
        };
        Ok(self.target.actions.push(action))
    }

    pub(crate) fn while_group(&mut self, start: Position) -> Result<wir::ActionId> {
        self.expect(TokenKind::LParen, "expected '(' after 'While'")?;
        self.condition_pending = true;
        let condition = self.value()?;
        self.expect(TokenKind::RParen, "expected ')' after While condition")?;
        self.expect_semi("expected ';' after While condition")?;
        let (body, end) = self.actions_until_matching_end("'While' requires a matching 'End'")?;
        let action = Action::While {
            condition,
            body,
            span: Some(Span::new(self.file(), start, end)),
        };
        Ok(self.target.actions.push(action))
    }

    /// `For Player Variable(player, name, start, stop, step)` — the
    /// reference's per-player loop form (parsed from pinned reference
    /// evidence; the differential gate normalizes it to the declared global
    /// form, #119).
    pub(crate) fn for_player_group(&mut self, start: Position) -> Result<wir::ActionId> {
        self.expect(
            TokenKind::LParen,
            "expected '(' after 'For Player Variable'",
        )?;
        let (player, variable, name_span) = self.player_variable_arg()?;
        self.expect(TokenKind::Comma, "expected ',' after loop variable")?;
        let start_value = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after start")?;
        let stop = self.value()?;
        self.expect(TokenKind::Comma, "expected ',' after stop")?;
        let step = self.value()?;
        self.expect(TokenKind::RParen, "expected ')' after For bounds")?;
        self.expect_semi("expected ';' after For Player Variable")?;
        let (body, end) =
            self.actions_until_matching_end("'For Player Variable' requires a matching 'End'")?;
        let action = Action::ForPlayerVariable {
            player,
            variable,
            start: start_value,
            stop,
            step,
            body,
            span: Some(Span::new(self.file(), start, end)),
            target_span: Some(name_span),
        };
        Ok(self.target.actions.push(action))
    }

    fn actions_until_matching_end(
        &mut self,
        missing_end_message: &str,
    ) -> Result<(Vec<wir::ActionId>, Position)> {
        let (body, stop) = self.actions_until_end()?;
        if stop != Stop::End {
            return Err(self.malformed(missing_end_message, self.previous()));
        }
        self.consume_keyword("end")?;
        self.expect_semi("expected ';' after 'End'")?;
        Ok((body, self.previous_span().1))
    }

    pub(crate) fn action_call_from_phrase(
        &mut self,
        phrase: String,
        start: Position,
        end: Position,
    ) -> Result<wir::ActionId> {
        match self.resolve_entry(Kind::Structural, &phrase) {
            Some(entry) => match entry.id.as_str() {
                "setGlobalVariable" | "modifyGlobalVariable" => {
                    let modify = entry.id == "modifyGlobalVariable";
                    self.expect(
                        TokenKind::LParen,
                        "expected '(' after 'Set/Modify Global Variable'",
                    )?;
                    let (variable, target_span) = self.global_variable_arg()?;
                    self.expect(TokenKind::Comma, "expected ',' after variable")?;
                    let op = if modify {
                        let op = self.modify_op()?;
                        self.expect(TokenKind::Comma, "expected ',' after modify operator")?;
                        Some(op)
                    } else {
                        None
                    };
                    let value = self.value()?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect_semi("expected ';'")?;
                    let span = Some(Span::new(self.file(), start, end));
                    let target_span = Some(target_span);
                    Ok(self.target.actions.push(match op {
                        Some(op) => Action::ModifyGlobalVariable {
                            variable,
                            op,
                            value,
                            span,
                            target_span,
                        },
                        None => Action::SetGlobalVariable {
                            variable,
                            value,
                            span,
                            target_span,
                        },
                    }))
                }
                "setPlayerVariable" | "modifyPlayerVariable" => {
                    let modify = entry.id == "modifyPlayerVariable";
                    self.expect(TokenKind::LParen, "expected '('")?;
                    let (player, variable, target_span) = self.player_variable_arg()?;
                    self.expect(TokenKind::Comma, "expected ',' after variable")?;
                    let op = if modify {
                        let op = self.modify_op()?;
                        self.expect(TokenKind::Comma, "expected ',' after modify operator")?;
                        Some(op)
                    } else {
                        None
                    };
                    let value = self.value()?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect_semi("expected ';'")?;
                    let span = Some(Span::new(self.file(), start, end));
                    let target_span = Some(target_span);
                    Ok(self.target.actions.push(match (op, modify) {
                        (Some(op), true) => Action::ModifyPlayerVariable {
                            player,
                            variable,
                            op,
                            value,
                            span,
                            target_span,
                        },
                        _ => Action::SetPlayerVariable {
                            player,
                            variable,
                            value,
                            span,
                            target_span,
                        },
                    }))
                }
                "forGlobalVariable" => self.for_group(start),
                "forPlayerVariable" => self.for_player_group(start),
                "callSubroutine" => {
                    self.expect(TokenKind::LParen, "expected '('")?;
                    let (name, name_start, name_end) = self.phrase()?;
                    let subroutine = self.subroutine_by_name(
                        &name,
                        Some(Span::new(self.file(), name_start, name_end)),
                    )?;
                    self.expect(TokenKind::RParen, "expected ')'")?;
                    self.expect_semi("expected ';'")?;
                    Ok(self.target.actions.push(Action::CallSubroutine {
                        subroutine,
                        span: Some(Span::new(self.file(), start, end)),
                        callee_span: Some(Span::new(self.file(), name_start, name_end)),
                    }))
                }
                other => Err(WorkshopError::unsupported(
                    format!("structural action '{other}' is not supported in action position"),
                    Some(Span::new(self.file(), start, end)),
                )),
            },
            None => {
                // Generic action call; the argument list is optional.
                let Some(action) = self
                    .resolve_entry(Kind::Action, &phrase)
                    .or_else(|| self.resolve_entry(Kind::Action, &format!("{phrase} ")))
                else {
                    let candidates = self.spelling_candidates(Kind::Action, &phrase, true);
                    return Err(WorkshopError::unknown(
                        "action",
                        phrase,
                        self.locale.clone(),
                        Some(Span::new(self.file(), start, end)),
                    )
                    .with_candidates(candidates));
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
                        let (player, variable, name_span) = self.player_variable_arg()?;
                        let name_span = Some(name_span);
                        let mut args = Vec::with_capacity(4);
                        args.push(
                            self.target.values.push(
                                wir::ValueNode::new(
                                    wir::Value::PlayerVariable { player, variable },
                                    None,
                                )
                                .with_identifier(name_span),
                            ),
                        );
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
                            let domain =
                                self.context.expected_domain(action.id.as_str(), arg_index);
                            args.push(self.value_in_domain(domain)?);
                            arg_index += 1;
                        }
                        self.expect(TokenKind::RParen, "expected ')'")?;
                        self.expect_semi("expected ';' after action")?;
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
                        let (name, name_start, name_end) = self.phrase()?;
                        let subroutine = self.subroutine_by_name(
                            &name,
                            Some(Span::new(self.file(), name_start, name_end)),
                        )?;
                        self.expect(TokenKind::Comma, "expected ',' after subroutine")?;
                        let domain = self.context.expected_domain(action.id.as_str(), 1);
                        let behavior = self.value_in_domain(domain)?;
                        self.expect(TokenKind::RParen, "expected ')'")?;
                        self.expect_semi("expected ';' after action")?;
                        let name_span = Some(Span::new(self.file(), name_start, name_end));
                        let subroutine_value = self.target.values.push(
                            ValueNode::new(Value::Subroutine(subroutine), name_span)
                                .with_identifier(name_span),
                        );
                        return Ok(self.target.actions.push(Action::Call {
                            name: action.id.clone(),
                            args: vec![subroutine_value, behavior],
                            span: Some(Span::new(self.file(), start, end)),
                        }));
                    }
                    "stopChasingPlayerVariable" => {
                        self.expect(TokenKind::LParen, "expected '('")?;
                        let (player, variable, name_span) = self.player_variable_arg()?;
                        self.expect(TokenKind::RParen, "expected ')'")?;
                        self.expect_semi("expected ';' after action")?;
                        let player_variable = self.target.values.push(
                            ValueNode::new(Value::PlayerVariable { player, variable }, None)
                                .with_identifier(Some(name_span)),
                        );
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
                self.expect_semi("expected ';' after action")?;
                Ok(self.target.actions.push(Action::Call {
                    name: action.id.clone(),
                    args,
                    span: Some(Span::new(self.file(), start, end)),
                }))
            }
        }
    }

    pub(crate) fn modify_op(&mut self) -> Result<ModifyOp> {
        let (phrase, start, end) = self.phrase()?;
        // Operator spellings carry the same punctuation/trailing-space
        // shapes as other catalog aliases (#448); an adjacent tail is part
        // of the spelling only when the joined text resolves.
        let (phrase, end) = self
            .extend_entry_spelling(&phrase, end, &[Kind::Operator])
            .unwrap_or((phrase, end));
        let entry = self.resolve_entry(Kind::Operator, &phrase).ok_or_else(|| {
            WorkshopError::unknown(
                "modify operator",
                phrase.clone(),
                self.locale.clone(),
                Some(Span::new(self.file(), start, end)),
            )
            .with_candidates(self.spelling_candidates(Kind::Operator, &phrase, false))
        })?;
        ModifyOp::from_catalog_id(&entry.id).ok_or_else(|| {
            WorkshopError::unsupported(
                format!("unsupported modify operator '{}'", entry.id),
                Some(Span::new(self.file(), start, end)),
            )
        })
    }
}
