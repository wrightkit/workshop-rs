# Workshop Language Support

This document is the **single authoritative source of truth for declared `workshop-rs` support status** across the Overwatch Workshop language surface.

It answers the question:
> **Does `workshop-rs` support this Workshop capability today?**

## Support Semantics

Every capability in this document uses one of three visible support states:

- `✅ Supported`: The capability is part of the currently supported `workshop-rs` contract for its stated scope.
- `🚧 Coming soon`: The capability is known and intended for support, but the current release does not yet provide the complete user-visible behavior.
- `❌ Unsupported`: The capability is intentionally outside the supported contract or has no planned support under the current project scope.

> [!NOTE]
> `workshop-rs` is a compiler, parser, validator, emitter, and semantic core for the Overwatch Workshop language. Live runtime execution/simulation within the Overwatch game client is an engine concern and is classified as `❌ Unsupported`.

## Contents

1. [Program Structure & Rules](#1-program-structure--rules)
2. [Variables & Subroutines](#2-variables--subroutines)
3. [Events & Event Filters](#3-events--event-filters)
4. [Conditions & Control Flow](#4-conditions--control-flow)
5. [Operators & Variable Modifications](#5-operators--variable-modifications)
6. [Actions Inventory](#6-actions-inventory)
7. [Values Inventory](#7-values-inventory)
8. [Enumerated Domains](#8-enumerated-domains)
9. [Custom-Game Settings](#9-custom-game-settings)
10. [Strings & Localization](#10-strings--localization)
11. [Tooling & Semantic Capabilities](#11-tooling--semantic-capabilities)
12. [Intentionally Out-of-Scope Capabilities](#12-intentionally-out-of-scope-capabilities)

## 1. Program Structure & Rules

| Feature | Status | Notes |
| --- | --- | --- |
| `settings` block | ✅ Supported | Top-level custom game settings block containing lobby, modes, heroes, extensions, and workshop parameters. |
| `variables` block | ✅ Supported | Top-level variable declaration block declaring `global` and `player` variable indices. |
| `subroutines` block | ✅ Supported | Top-level subroutine declaration block declaring subroutine names and indices. |
| `rule` declaration | ✅ Supported | Rule declaration `rule ("Rule Name") { ... }` with event, conditions, and actions blocks. |
| `event` block | ✅ Supported | Rule header block defining triggering event type and applicable filters. |
| `conditions` block | ✅ Supported | Rule precondition block evaluated before rule actions run. |
| `actions` block | ✅ Supported | Rule action sequence executed when event triggers and conditions pass. |
| `disabled` rule modifier | ✅ Supported | `disabled rule ("...")` disables execution of the entire rule. |
| `disabled` condition modifier | ✅ Supported | `disabled <condition>;` skips evaluation of a single condition within a rule. |
| `disabled` action modifier | ✅ Supported | `disabled <action>;` skips execution of a single action within a rule. |

## 2. Variables & Subroutines

| Feature | Status | Notes |
| --- | --- | --- |
| `global` variable declaration | ✅ Supported | Numerical indexed declaration in `variables { global: 0: varName }`. |
| `Global Variable` read | ✅ Supported | Read access via `Global.var` or `Global Variable(var)`. |
| `Set Global Variable` | ✅ Supported | Assigns a value to a global variable: `Set Global Variable(var, value)`. |
| `Modify Global Variable` | ✅ Supported | Modifies a global variable with an operation: `Modify Global Variable(var, operation, value)`. |
| `Set Global Variable At Index` | ✅ Supported | Assigns a value into an array global variable at index: `Set Global Variable At Index(var, index, value)`. |
| `Modify Global Variable At Index` | ✅ Supported | Modifies an array global variable element at index: `Modify Global Variable At Index(var, index, operation, value)`. |
| `player` variable declaration | ✅ Supported | Numerical indexed declaration in `variables { player: 0: varName }`. |
| `Player Variable` read | ✅ Supported | Read access via `Player.var` or `Player Variable(player, var)`. |
| `Set Player Variable` | ✅ Supported | Assigns a value to a player variable: `Set Player Variable(player, var, value)`. |
| `Modify Player Variable` | ✅ Supported | Modifies a player variable with an operation: `Modify Player Variable(player, var, operation, value)`. |
| `Set Player Variable At Index` | ✅ Supported | Assigns a value into an array player variable at index: `Set Player Variable At Index(player, var, index, value)`. |
| `Modify Player Variable At Index` | ✅ Supported | Modifies an array player variable element at index: `Modify Player Variable At Index(player, var, index, operation, value)`. |
| `subroutines` declaration | ✅ Supported | Numerical indexed declaration in `subroutines { 0: SubName }`. |
| `Call Subroutine` | ✅ Supported | Synchronous subroutine call: `Call Subroutine(SubName)`. |
| `Start Rule` | ✅ Supported | Asynchronous subroutine rule invocation: `Start Rule(SubName, RestartBehavior)`. |
| `Subroutine` event rule | ✅ Supported | Rule event block `event { Subroutine; SubName; }` triggered by subroutine calls. |

## 3. Events & Event Filters

| Feature | Status | Notes |
| --- | --- | --- |
| `Ongoing - Each Player` | ✅ Supported | Filters: Team, Player. |
| `Ongoing - Global` | ✅ Supported | Global event (no filters). |
| `Player Dealt Damage` | ✅ Supported | Filters: Team, Player. |
| `Player Dealt Final Blow` | ✅ Supported | Filters: Team, Player. |
| `Player Dealt Healing` | ✅ Supported | Filters: Team, Player. |
| `Player Dealt Knockback` | ✅ Supported | Filters: Team, Player. |
| `Player Died` | ✅ Supported | Filters: Team, Player. |
| `Player Earned Elimination` | ✅ Supported | Filters: Team, Player. |
| `Player Joined Match` | ✅ Supported | Filters: Team, Player. |
| `Player Left Match` | ✅ Supported | Filters: Team, Player. |
| `Player Received Healing` | ✅ Supported | Filters: Team, Player. |
| `Player Took Damage` | ✅ Supported | Filters: Team, Player. |
| `Player Took Knockback` | ✅ Supported | Filters: Team, Player. |
| `Subroutine` | ✅ Supported | Global event (no filters). |
| `EventTeam` (Team filter) | ✅ Supported | Filters player events by team (`All`, `Team 1`, `Team 2`). |
| `EventPlayer` (Player / Hero filter) | ✅ Supported | Filters player events by slot (`All`, `Slot 0`..`Slot 11`) or specific hero. |
| `Subroutine` name filter | ✅ Supported | Identifies the target subroutine name for subroutine event rules. |

## 4. Conditions & Control Flow

| Feature | Status | Notes |
| --- | --- | --- |
| Comparison condition | ✅ Supported | Binary condition expression (`Value OP Value`) with implicit logical AND across rule conditions. |
| `If` | ✅ Supported | Begins a conditional execution block: `If(Condition);`. |
| `Else If` | ✅ Supported | Secondary conditional execution block: `Else If(Condition);`. |
| `Else` | ✅ Supported | Fallback conditional execution block: `Else;`. |
| `End` | ✅ Supported | Terminates an `If`, `While`, or `For` block (including oracle-compatible implicit trailing `End`). |
| `While` | ✅ Supported | Loop block executed while condition evaluates to true: `While(Condition); ... End;`. |
| `For Global Variable` | ✅ Supported | Loop iterating a global variable from start to stop with step: `For Global Variable(var, start, stop, step); ... End;`. |
| `For Player Variable` | ✅ Supported | Loop iterating a player variable from start to stop with step: `For Player Variable(player, var, start, stop, step); ... End;`. |
| `Loop` | ✅ Supported | Restarts rule execution from the first action. |
| `Loop If` | ✅ Supported | Restarts rule execution if condition evaluates to true. |
| `Loop If Condition Is True` | ✅ Supported | Restarts rule execution if all rule conditions are currently true. |
| `Loop If Condition Is False` | ✅ Supported | Restarts rule execution if any rule condition is currently false. |
| `Break` | ✅ Supported | Breaks out of the innermost `While` or `For` loop. |
| `Continue` | ✅ Supported | Advances to the next iteration of the innermost `While` or `For` loop. |
| `Skip` | ✅ Supported | Unconditionally skips the specified number of subsequent actions. |
| `Skip If` | ✅ Supported | Skips subsequent actions if condition evaluates to true. |
| `Wait` | ✅ Supported | Pauses execution for a duration with condition restart/abort behavior. |
| `Wait Until` | ✅ Supported | Pauses execution until condition evaluates to true or timeout expires. |
| `Abort` | ✅ Supported | Immediately terminates rule action execution. |
| `Abort If` | ✅ Supported | Terminates rule action execution if condition evaluates to true. |
| `Abort If Condition Is True` | ✅ Supported | Terminates rule execution if all rule conditions are currently true. |
| `Abort If Condition Is False` | ✅ Supported | Terminates rule execution if any rule condition is currently false. |
| `Return` | ✅ Supported | Returns from a subroutine execution rule. |

## 5. Operators & Variable Modifications

| Feature | Status | Notes |
| --- | --- | --- |
| `!=` | ✅ Supported | Comparison operator in conditions and `Compare` value. |
| `<` | ✅ Supported | Comparison operator in conditions and `Compare` value. |
| `<=` | ✅ Supported | Comparison operator in conditions and `Compare` value. |
| `==` | ✅ Supported | Comparison operator in conditions and `Compare` value. |
| `>` | ✅ Supported | Comparison operator in conditions and `Compare` value. |
| `>=` | ✅ Supported | Comparison operator in conditions and `Compare` value. |
| `Add` | ✅ Supported | Arithmetic operator and variable modification operation. |
| `Append To Array` | ✅ Supported | Array variable modification operation. |
| `Divide` | ✅ Supported | Arithmetic operator and variable modification operation. |
| `Modulo` | ✅ Supported | Arithmetic operator and variable modification operation. |
| `Multiply` | ✅ Supported | Arithmetic operator and variable modification operation. |
| `Raise To Power` | ✅ Supported | Arithmetic operator and variable modification operation. |
| `Remove From Array` | ✅ Supported | Array variable modification operation. |
| `Remove From Array By Index` | ✅ Supported | Array variable modification operation. |
| `Remove From Array By Value` | ✅ Supported | Array variable modification operation. |
| `Subtract` | ✅ Supported | Arithmetic operator and variable modification operation. |

## 6. Actions Inventory

All 219 canonical Workshop actions are supported:

| Feature | Status | Notes |
| --- | --- | --- |
| `Abort` | ✅ Supported | No parameters. |
| `Abort If` | ✅ Supported | Parameters: (Condition: Boolean). |
| `Abort If Condition Is False` | ✅ Supported | No parameters. |
| `Abort If Condition Is True` | ✅ Supported | No parameters. |
| `Add Health Pool To Player` | ✅ Supported | Parameters: (Object: Player|Array, Health: Health, Number: Number, Boolean: Boolean, Boolean: Boolean). |
| `Allow Button` | ✅ Supported | Parameters: (Player: Player|Array, Button: Button). |
| `Apply Impulse` | ✅ Supported | Parameters: (Player: Player|Array, Direction: Vector, Speed: Number, Relativity: Relativity, Impulse: Impulse). |
| `Attach Players` | ✅ Supported | Parameters: (Player: Player, Player: Player, Position: Vector). |
| `Big Message` | ✅ Supported | Parameters: (VisibleTo: Player|Array, Header: Object). |
| `Break` | ✅ Supported | No parameters. |
| `Cancel Primary Action` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Chase Global Variable At Rate` | ✅ Supported | Parameters: (Variable: Variable, Destination: Any, Rate: Number|Boolean, Reevaluation: ChaseRateReeval). |
| `Chase Global Variable Over Time` | ✅ Supported | Parameters: (Variable: Variable, Destination: Any, Duration: Number, Reevaluation: ChaseTimeReeval). |
| `Chase Player Variable At Rate` | ✅ Supported | Parameters: (Player: Player|Array, Variable: Variable, Destination: Any, Rate: Number, Reevaluation: ChaseRateReeval). |
| `Chase Player Variable Over Time` | ✅ Supported | Parameters: (Player: Player|Array, Variable: Variable, Destination: Any, Duration: Number, Reevaluation: ChaseTimeReeval). |
| `Clear Status` | ✅ Supported | Parameters: (Object: Player|Array, Status: Status). |
| `Communicate` | ✅ Supported | Parameters: (Object: Player|Array, Comms: Comms). |
| `Continue` | ✅ Supported | No parameters. |
| `Create Beam Effect` | ✅ Supported | Parameters: (VisibleTo: Player|Array, Type: Beam, StartPosition: Vector, EndPosition: Vector, Color: Color, Reevaluation: EffectReeval). |
| `Create Dummy Bot` | ✅ Supported | Parameters: (Hero: Hero, Team: Team, Slot: Number|Boolean, Position: Vector, Direction: Vector). |
| `Create Effect` | ✅ Supported | Parameters: (VisibleTo: Player|Array, Type: Effect, Color: Color, Position: Vector|Player, Radius: Number, Reevaluation: EffectReeval). |
| `Create HUD Text` | ✅ Supported | Parameters: (VisibleTo: Player|Array, Header: Object|String, Subheader: Object|String, Text: Object|String, Location: HudPosition, SortOrder: Number|Boolean, HeaderColor: Color, SubheaderColor: Color, TextColor: Color, Reevaluation: HudReeval, Spectators: SpecVisibility). |
| `Create Homing Projectile` | ✅ Supported | Parameters: (Projectile: Projectile, Object: Player|Array, Position: Vector, Direction: Vector, Relativity: Relativity, ModifyHealth: ModifyHealth, Team: Team, Number: Number, Number: Number, Number: Number, DynamicEffect: DynamicEffect, DynamicEffect: DynamicEffect, Number: Number, Number: Number, Number: Number, Number: Number, Player: Player, Number: Number). |
| `Create Icon` | ✅ Supported | Parameters: (VisibleTo: Player|Array, Position: Vector|Player, Icon: Icon, Reevaluation: IconReeval, Color: Color, ShowWhenOffscreen: Boolean). |
| `Create In-World Text` | ✅ Supported | Parameters: (VisibleTo: Player|Array, Header: Object, Position: Vector|Player, Scale: Number, Clipping: Clipping, Reevaluation: WorldTextReeval, TextColor: Color, Spectators: SpecVisibility). |
| `Create Progress Bar HUD Text` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Object: Object, HudPosition: HudPosition, Number: Number|Boolean, Color: Color, Color: Color, ProgressHudReeval: ProgressHudReeval, SpecVisibility: SpecVisibility). |
| `Create Progress Bar In-World Text` | ✅ Supported | Parameters: (VisibleTo: Player|Array, Value: Number, Text: Object, Position: Vector|Player, Scale: Number, Clipping: Clipping, HeaderColor: Color, TextColor: Color, Reevaluation: ProgressWorldTextReeval, NonteamSpectators: SpecVisibility). |
| `Create Projectile` | ✅ Supported | Parameters: (Projectile: Projectile, Object: Player|Array, Position: Vector, Direction: Vector, Relativity: Relativity, ModifyHealth: ModifyHealth, Team: Team, Number: Number, Number: Number, Number: Number, DynamicEffect: DynamicEffect, DynamicEffect: DynamicEffect, Number: Number, Number: Number, Number: Number, Number: Number, Number: Number, Number: Number). |
| `Create Projectile Effect` | ✅ Supported | Parameters: (Object: Player|Array, Projectile: Projectile, Object: Player|Array, Object: Vector|Player, Direction: Vector, Number: Number, ProjectileEffectReeval: ProjectileEffectReeval). |
| `Damage` | ✅ Supported | Parameters: (Object: Player|Array, Player: Player, Number: Number). |
| `Declare Match Draw` | ✅ Supported | No parameters. |
| `Declare Player Victory` | ✅ Supported | Parameters: (Player: Player). |
| `Declare Round Draw` | ✅ Supported | No parameters. |
| `Declare Round Victory` | ✅ Supported | Parameters: (Team: Team). |
| `Declare Team Victory` | ✅ Supported | Parameters: (Team: Team). |
| `Destroy All Dummy Bots` | ✅ Supported | No parameters. |
| `Destroy All Effects` | ✅ Supported | No parameters. |
| `Destroy All HUD Text` | ✅ Supported | No parameters. |
| `Destroy All Icons` | ✅ Supported | No parameters. |
| `Destroy All In-World Text` | ✅ Supported | No parameters. |
| `Destroy All Progress Bar HUD Text` | ✅ Supported | No parameters. |
| `Destroy All Progress Bar In-World Text` | ✅ Supported | No parameters. |
| `Destroy Dummy Bot` | ✅ Supported | Parameters: (Team: Team, Number: Number). |
| `Destroy Effect` | ✅ Supported | Parameters: (EffectId: EntityId). |
| `Destroy HUD Text` | ✅ Supported | Parameters: (TextId: TextId). |
| `Destroy Icon` | ✅ Supported | Parameters: (EntityId: EntityId). |
| `Destroy In-World Text` | ✅ Supported | Parameters: (TextId: TextId). |
| `Destroy Progress Bar HUD Text` | ✅ Supported | Parameters: (TextId: TextId). |
| `Destroy Progress Bar In-World Text` | ✅ Supported | Parameters: (TextId: TextId). |
| `Detach Players` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Built-In Game Mode Announcer` | ✅ Supported | No parameters. |
| `Disable Built-In Game Mode Completion` | ✅ Supported | No parameters. |
| `Disable Built-In Game Mode Music` | ✅ Supported | No parameters. |
| `Disable Built-In Game Mode Respawning` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Built-In Game Mode Scoring` | ✅ Supported | No parameters. |
| `Disable Death Spectate All Players` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Death Spectate Target HUD` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Game Mode HUD` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Disable Game Mode In-World UI` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Disable Hero HUD` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Disable Inspector Recording` | ✅ Supported | No parameters. |
| `Disable Kill Feed` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Messages` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Movement Collision With Environment` | ✅ Supported | Parameters: (Player: Player|Array, IncludeFloors: Boolean). |
| `Disable Movement Collision With Players` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Disable Nameplates` | ✅ Supported | Parameters: (Object: Player|Array, Object: Player|Array). |
| `Disable Scoreboard` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Disable Text Chat` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Voice Chat` | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean, Boolean: Boolean, Boolean: Boolean). |
| `Disallow Button` | ✅ Supported | Parameters: (Player: Player|Array, Button: Button). |
| `Enable Built-In Game Mode Announcer` | ✅ Supported | No parameters. |
| `Enable Built-In Game Mode Completion` | ✅ Supported | No parameters. |
| `Enable Built-In Game Mode Music` | ✅ Supported | No parameters. |
| `Enable Built-In Game Mode Respawning` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Enable Built-In Game Mode Scoring` | ✅ Supported | No parameters. |
| `Enable Death Spectate All Players` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Enable Death Spectate Target HUD` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Enable Game Mode HUD` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Enable Game Mode In-World UI` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Enable Hero HUD` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Enable Inspector Recording` | ✅ Supported | No parameters. |
| `Enable Kill Feed` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Enable Messages` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Enable Movement Collision With Environment` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Enable Movement Collision With Players` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Enable Nameplates` | ✅ Supported | Parameters: (Object: Player|Array, Object: Player|Array). |
| `Enable Scoreboard` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Enable Text Chat` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Enable Voice Chat` | ✅ Supported | Parameters: (Object: Player|Array). |
| `For Player Variable` | ✅ Supported | Parameters: (Player: Player, PlayerVariable: Variable, Number: Number, Number: Number, Number: Number). |
| `Force Player Hero` | ✅ Supported | Parameters: (Player, Hero). |
| `Force Throttle` | ✅ Supported | Parameters: (Player, MoveSpeed, InAirSpeed, SpectatorSpeed, GrappleBoost, JumpPower, MoveSpeed). |
| `Go To Assemble Heroes` | ✅ Supported | No parameters. |
| `Heal` | ✅ Supported | Parameters: (Object: Player|Array, Player: Player, Number: Number). |
| `Kill` | ✅ Supported | Parameters: (Object: Player|Array, Player: Player). |
| `Log To Inspector` | ✅ Supported | Parameters: (Object: Object). |
| `Loop` | ✅ Supported | No parameters. |
| `Loop If` | ✅ Supported | Parameters: (Condition: Boolean). |
| `Loop If Condition Is False` | ✅ Supported | No parameters. |
| `Loop If Condition Is True` | ✅ Supported | No parameters. |
| `Modify Global Variable` | ✅ Supported | Parameters: (Variable: Variable, Operation: Operation, Value: Any). |
| `Modify Global Variable At Index` | ✅ Supported | Parameters: (Variable: Global Variable, Index: Number|Boolean, Operation: Operation, Value: Object|Array). |
| `Modify Player Score` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Modify Player Variable At Index` | ✅ Supported | Parameters: (Variable: Player Variable, Index: Number|Boolean, Operation: Operation, Value: Object|Array). |
| `Modify Team Score` | ✅ Supported | Parameters: (Team: Team, Number: Number). |
| `Move Player to Team` | ✅ Supported | Parameters: (Object: Player|Array, Team: Team, Number: Number). |
| `Pause Match Time` | ✅ Supported | No parameters. |
| `Play Effect` | ✅ Supported | Parameters: (VisibleTo: Player|Array, Type: DynamicEffect, Color: Color, Position: Vector, Radius: Number). |
| `Preload Hero` | ✅ Supported | Parameters: (Object: Player|Array, Object: Hero|Array). |
| `Press Button` | ✅ Supported | Parameters: (Object: Player|Array, Button: Button). |
| `Remove All Health Pools From Player` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Remove Health Pool From Player` | ✅ Supported | Parameters: (HealthPoolId: HealthPoolId). |
| `Remove Player` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Reset Player Hero Availability` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Respawn` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Restart Match` | ✅ Supported | No parameters. |
| `Resurrect` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Return To Lobby` | ✅ Supported | No parameters. |
| `Set Ability 1 Enabled` | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Ability 2 Enabled` | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Ability Charge` | ✅ Supported | Parameters: (Object: Player|Array, Button: Button, Number: Number). |
| `Set Ability Cooldown` | ✅ Supported | Parameters: (Object: Player|Array, Button: Button, Number: Number|Boolean). |
| `Set Ability Resource` | ✅ Supported | Parameters: (Object: Player|Array, Button: Button, Number: Number). |
| `Set Aim Speed` | ✅ Supported | Parameters: (player: Player|Array, turnSpeedPercent: Number). |
| `Set Allowed Heroes` | ✅ Supported | Parameters: (Player: Player|Array, Heroes: Hero|Array). |
| `Set Ammo` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Number: Number). |
| `Set Crouch Enabled` | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Damage Dealt` | ✅ Supported | Parameters: (player: Player|Array, damageDealtPercent: Number). |
| `Set Damage Received` | ✅ Supported | Parameters: (player: Player|Array, damageReceivedPercent: Number). |
| `Set Environment Credit Player` | ✅ Supported | Parameters: (Player: Player|Array, Player: Player|Array). |
| `Set Facing` | ✅ Supported | Parameters: (Object: Player|Array, Direction: Vector, Relativity: Relativity). |
| `Set Global Variable At Index` | ✅ Supported | Parameters: (Variable: Global Variable, Index: Number|Boolean, Value: Object|Array). |
| `Set Gravity` | ✅ Supported | Parameters: (Player: Player|Array, Gravity: Number|Boolean). |
| `Set Healing Dealt` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Healing Received` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Invisible` | ✅ Supported | Parameters: (Player: Player|Array, InvisibleTo: Invis). |
| `Set Jump Enabled` | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Jump Vertical Speed` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Knockback Dealt` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Knockback Received` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Match Time` | ✅ Supported | Parameters: (Number: Number|Boolean). |
| `Set Max Ammo` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Number: Number). |
| `Set Max Health` | ✅ Supported | Parameters: (player: Player|Array, healthPercent: Number). |
| `Set Melee Enabled` | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Move Speed` | ✅ Supported | Parameters: (player: Player|Array, moveSpeedPercent: Number). |
| `Set Objective Description` | ✅ Supported | Parameters: (Object: Player|Array, Object: Object, HudReeval: HudReeval). |
| `Set Player Health` | ✅ Supported | Parameters: (player: Player|Array, amount: Number). |
| `Set Player Score` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Player Variable At Index` | ✅ Supported | Parameters: (Variable: Player Variable, Index: Number|Boolean, Value: Object|Array). |
| `Set Primary Fire Enabled` | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Projectile Gravity` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Projectile Speed` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Reload Enabled` | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Respawn Max Time` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Secondary Fire Enabled` | ✅ Supported | Parameters: (Player: Player|Array, bool: Boolean). |
| `Set Slow Motion` | ✅ Supported | Parameters: (Number: Number). |
| `Set Status` | ✅ Supported | Parameters: (player: Player|Array, assister: Player, status: Status, duration: Number). |
| `Set Team Score` | ✅ Supported | Parameters: (Team: Team, Number: Number). |
| `Set Ultimate Ability Enabled` | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Ultimate Charge` | ✅ Supported | Parameters: (player: Player|Array, chargePercent: Number|Boolean). |
| `Set Weapon` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Skip` | ✅ Supported | Parameters: (Value: Number|Boolean). |
| `Skip If` | ✅ Supported | Parameters: (Condition: Boolean, Number: Number|Boolean). |
| `Small Message` | ✅ Supported | Parameters: (VisibleTo: Player|Array, Header: Object). |
| `Start Accelerating` | ✅ Supported | Parameters: (Object: Player|Array, Direction: Vector, Number: Number, Number: Number, Relativity: Relativity, AccelReeval: AccelReeval). |
| `Start Assist` | ✅ Supported | Parameters: (Object: Player|Array, Object: Player|Array, AssistReeval: AssistReeval). |
| `Start Camera` | ✅ Supported | Parameters: (Player: Player|Array, EyePosition: Vector, LookAtPosition: Vector, Facing: Number). |
| `Start Damage Modification` | ✅ Supported | Parameters: (Object: Player|Array, Object: Player|Array, Number: Number, DamageReeval: DamageReeval). |
| `Start Damage Over Time` | ✅ Supported | Parameters: (Object: Player|Array, Player: Player, Number: Number, Number: Number). |
| `Start Facing` | ✅ Supported | Parameters: (Player: Player|Array, Direction: Vector, Turn Rate: Number, Relativity: Relativity, Reevaluation: FacingReeval). |
| `Start Forcing Dummy Bot Name` | ✅ Supported | Parameters: (Object: Player|Array, String: String). |
| `Start Forcing Player Outlines` | ✅ Supported | Parameters: (ViewedPlayers: Player|Array, ViewingPlayers: Player|Array, Visible: Boolean, Color: Color, Visibility: OutlineVisibility). |
| `Start Forcing Player Position` | ✅ Supported | Parameters: (Player: Player, Position: Vector, Boolean: Boolean). |
| `Start Forcing Player To Be Hero` | ✅ Supported | Parameters: (Object: Player|Array, Hero: Hero). |
| `Start Forcing Spawn Room` | ✅ Supported | Parameters: (Team: Team, Number: Number|Boolean). |
| `Start Forcing Throttle` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number|Boolean, Number: Number|Boolean, Number: Number|Boolean, Number: Number|Boolean, Number: Number|Boolean, Number: Number|Boolean). |
| `Start Game Mode` | ✅ Supported | No parameters. |
| `Start Heal Over Time` | ✅ Supported | Parameters: (Object: Player|Array, Player: Player, Number: Number, Number: Number). |
| `Start Healing Modification` | ✅ Supported | Parameters: (Object: Player|Array, Object: Player|Array, Number: Number, HealingReeval: HealingReeval). |
| `Start Holding Button` | ✅ Supported | Parameters: (Object: Player|Array, Button: Button). |
| `Start Modifying Hero Voice Lines` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Boolean: Boolean). |
| `Start Rule` | ✅ Supported | Parameters: (Subroutine: Subroutine, IfAlreadyExecuting: StartRuleBehavior). |
| `Start Scaling Barriers` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Boolean: Boolean). |
| `Start Scaling Player` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Boolean: Boolean). |
| `Start Throttle In Direction` | ✅ Supported | Parameters: (Player: Player|Array, Direction: Vector, Magnitude: Number, Relativity: Relativity, Throttle: Throttle, ThrottleReeval: ThrottleReeval). |
| `Start Transforming Throttle` | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Number: Number, Direction: Vector). |
| `Stop Accelerating` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop All Assists` | ✅ Supported | No parameters. |
| `Stop All Damage Modifications` | ✅ Supported | No parameters. |
| `Stop All Damage Over Time` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop All Heal Over Time` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop All Healing Modifications` | ✅ Supported | No parameters. |
| `Stop Assist` | ✅ Supported | Parameters: (AssistId: AssistId). |
| `Stop Camera` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Stop Chasing Global Variable` | ✅ Supported | Parameters: (Variable: Variable). |
| `Stop Chasing Player Variable` | ✅ Supported | Parameters: (Player: Player|Array, PlayerVariable: Variable). |
| `Stop Chasing Variable` | ✅ Supported | Parameters: (Variable). |
| `Stop Damage Modification` | ✅ Supported | Parameters: (DamageModificationId: DamageModificationId). |
| `Stop Damage Over Time` | ✅ Supported | Parameters: (DotId: DotId). |
| `Stop Facing` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Forcing Dummy Bot Name` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Forcing Hero` | ✅ Supported | Parameters: (Player). |
| `Stop Forcing Player Outlines` | ✅ Supported | Parameters: (Object: Player|Array, Object: Player|Array). |
| `Stop Forcing Player Position` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Forcing Player To Be Hero` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Forcing Spawn Room` | ✅ Supported | Parameters: (Team: Team). |
| `Stop Forcing Throttle` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Stop Heal Over Time` | ✅ Supported | Parameters: (HotId: HotId). |
| `Stop Healing Modification` | ✅ Supported | Parameters: (HealingModificationId: HealingModificationId). |
| `Stop Holding Button` | ✅ Supported | Parameters: (Object: Player|Array, Button: Button). |
| `Stop Modifying Hero Voice Lines` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Scaling Barriers` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Scaling Player` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Throttle In Direction` | ✅ Supported | Parameters: (Player: Player|Array). |
| `Stop Transforming Throttle` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Teleport` | ✅ Supported | Parameters: (Player: Player|Array, Position: Vector). |
| `Unpause Match Time` | ✅ Supported | No parameters. |
| `Wait` | ✅ Supported | Parameters: (Duration: Any, WaitBehavior: Wait). |
| `Wait Until` | ✅ Supported | Parameters: (Condition: Any, Timeout: Number). |

## 7. Values Inventory

All 255 canonical Workshop values and expressions are supported:

| Feature | Status | Notes |
| --- | --- | --- |
| `Ability Charge` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Button: Button). |
| `Ability Cooldown` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Button: Button). |
| `Ability Icon String` | ✅ Supported | Returns: `String`; Parameters: (Hero: Hero, Button: Button). |
| `Ability Resource` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Button: Button). |
| `Absolute Value` | ✅ Supported | Returns: `Number`; Parameters: (Value: Number). |
| `Add` | ✅ Supported | Returns: `Number|Vector`; Parameters: (a: Number|Boolean|Vector, b: Number|Boolean|Vector). |
| `All Damage Heroes` | ✅ Supported | Returns: `Array`. |
| `All Dead Players` | ✅ Supported | Returns: `Array`; Parameters: (Team: Team). |
| `All Heroes` | ✅ Supported | Returns: `Array`. |
| `All Living Players` | ✅ Supported | Returns: `Array`; Parameters: (Team: Team). |
| `All Players` | ✅ Supported | Returns: `Array`; Parameters: (Team: Team). |
| `All Players Not On Objective` | ✅ Supported | Returns: `Array`; Parameters: (Team: Team). |
| `All Players On Objective` | ✅ Supported | Returns: `Array`; Parameters: (Team: Team). |
| `All Support Heroes` | ✅ Supported | Returns: `Array`. |
| `All Tank Heroes` | ✅ Supported | Returns: `Array`. |
| `Allowed Heroes` | ✅ Supported | Returns: `Array`; Parameters: (Player: Player). |
| `Altitude Of` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Ammo` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Clip: Number). |
| `And` | ✅ Supported | Returns: `Boolean`; Parameters: (A: Boolean|Number|Object|Array, B: Boolean|Number|Object|Array). |
| `Angle Between Vectors` | ✅ Supported | Returns: `Number`; Parameters: (Direction: Vector, Direction: Vector). |
| `Angle Difference` | ✅ Supported | Returns: `Number`; Parameters: (Value: Number, Value: Number). |
| `Append To Array` | ✅ Supported | Returns: `Array`; Parameters: (Array: Object|Array, Value: Object|Array). |
| `Arccosine In Degrees` | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Arccosine In Radians` | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Arcsine In Degrees` | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Arcsine In Radians` | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Arctangent In Degrees` | ✅ Supported | Returns: `Number`; Parameters: (Number: Number, Number: Number). |
| `Arctangent In Radians` | ✅ Supported | Returns: `Number`; Parameters: (Number: Number, Number: Number). |
| `Array` | ✅ Supported | Returns: `Array`; Parameters: (Value: Object|Array). |
| `Array Contains` | ✅ Supported | Returns: `Boolean`; Parameters: (Array: Array, Value: Object|Array). |
| `Array Slice` | ✅ Supported | Returns: `Array`; Parameters: (Array: Array, Start Index: Number|Boolean, Count: Number|Boolean). |
| `Assist Count` | ✅ Supported | Returns: `Number`. |
| `Attacker` | ✅ Supported | Returns: `Player`. |
| `Backward` | ✅ Supported | Returns: `Array`. |
| `Char In String` | ✅ Supported | Returns: `String`; Parameters: (String: Any, Index: Any). |
| `Closest Player To` | ✅ Supported | Returns: `Player`; Parameters: (Position: Vector, Team: Team). |
| `Compare` | ✅ Supported | Returns: `Boolean`; Parameters: (a: Any, operator: __Operator__, b: Any). |
| `Control Mode Scoring Percentage` | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Control Mode Scoring Team` | ✅ Supported | Returns: `Team`. |
| `Cosine From Degrees` | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Cosine From Radians` | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Count Of` | ✅ Supported | Returns: `Number`; Parameters: (Array: Array). |
| `Cross Product` | ✅ Supported | Returns: `Vector`; Parameters: (Vector: Vector, Vector: Vector). |
| `Current Array Element` | ✅ Supported | Returns: `Any`. |
| `Current Array Index` | ✅ Supported | Returns: `Any`. |
| `Current Game Mode` | ✅ Supported | Returns: `Gamemode`. |
| `Current Map` | ✅ Supported | Returns: `Map`. |
| `Custom Color` | ✅ Supported | Returns: `Color`; Parameters: (Red: Number, Green: Number, Blue: Number, Alpha: Number). |
| `Custom String` | ✅ Supported | Returns: `String`; Parameters: (Format: String, Replacement 1: Object|Array, Replacement 2: Object|Array, Replacement 3: Object|Array). |
| `Damage Modification Count` | ✅ Supported | Returns: `Number`. |
| `Damage Over Time Count` | ✅ Supported | Returns: `Number`. |
| `Direction From Angles` | ✅ Supported | Returns: `Vector`; Parameters: (HorizontalAngle: Number, VerticalAngle: Number). |
| `Direction Towards` | ✅ Supported | Returns: `Vector`; Parameters: (Position: Vector|Player|Array, Position: Vector|Player|Array). |
| `Distance Between` | ✅ Supported | Returns: `Number`; Parameters: (Position: Vector|Player|Array, Position: Vector|Player|Array). |
| `Divide` | ✅ Supported | Returns: `Number|Vector`; Parameters: (a: Number|Boolean|Vector, b: Number|Boolean|Vector). |
| `Dot Product` | ✅ Supported | Returns: `Number`; Parameters: (Value: Vector, Value: Vector). |
| `Down` | ✅ Supported | Returns: `Array`. |
| `Empty Array` | ✅ Supported | Returns: `Array`. |
| `Entity Count` | ✅ Supported | Returns: `Number`. |
| `Entity Exists` | ✅ Supported | Returns: `Boolean`; Parameters: (Entity: Player|EntityId). |
| `Evaluate Once` | ✅ Supported | Returns: `Object|Array`; Parameters: (Value: Object|Array). |
| `Event Ability` | ✅ Supported | Returns: `Button`. |
| `Event Damage` | ✅ Supported | Returns: `Number`. |
| `Event Direction` | ✅ Supported | Returns: `Vector`. |
| `Event Healing` | ✅ Supported | Returns: `Number`. |
| `Event Player` | ✅ Supported | Returns: `Player`. |
| `Event Was Critical Hit` | ✅ Supported | Returns: `Boolean`. |
| `Event Was Environment` | ✅ Supported | Returns: `Boolean`. |
| `Event Was Health Pack` | ✅ Supported | Returns: `Boolean`. |
| `Eye Position` | ✅ Supported | Returns: `Vector`; Parameters: (Player: Player). |
| `Facing Direction Of` | ✅ Supported | Returns: `Vector`; Parameters: (Player: Player). |
| `Farthest Player From` | ✅ Supported | Returns: `Player`; Parameters: (Position: Vector, Team: Team). |
| `Filtered Array` | ✅ Supported | Returns: `Array`; Parameters: (Array: Array, Condition: Boolean). |
| `First Of` | ✅ Supported | Returns: `Object|Array`; Parameters: (Array: Array). |
| `Flag Position` | ✅ Supported | Returns: `Vector`; Parameters: (Team: Team). |
| `Forward` | ✅ Supported | Returns: `Array`. |
| `Game Mode` | ✅ Supported | Returns: `Gamemode`; Parameters: (Gamemode: Gamemode). |
| `Has Spawned` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player|Array). |
| `Has Status` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player, Status: Status). |
| `Heal Over Time Count` | ✅ Supported | Returns: `Number`. |
| `Healee` | ✅ Supported | Returns: `Player`. |
| `Healer` | ✅ Supported | Returns: `Player`. |
| `Healing Modification Count` | ✅ Supported | Returns: `Number`. |
| `Health` | ✅ Supported | Returns: `Number`; Parameters: (player: Player). |
| `Health Of Type` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Health: Health). |
| `Hero` | ✅ Supported | Returns: `Hero`; Parameters: (Hero: Hero). |
| `Hero Being Duplicated` | ✅ Supported | Returns: `Hero`; Parameters: (Player: Player). |
| `Hero Icon String` | ✅ Supported | Returns: `String`; Parameters: (Hero: Hero). |
| `Hero Of` | ✅ Supported | Returns: `Hero`; Parameters: (player: Player). |
| `Horizontal Angle From Direction` | ✅ Supported | Returns: `Number`; Parameters: (Direction: Vector). |
| `Horizontal Angle Towards` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Position: Vector). |
| `Horizontal Facing Angle Of` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Horizontal Speed Of` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Host Player` | ✅ Supported | Returns: `Player`. |
| `Icon String` | ✅ Supported | Returns: `String`; Parameters: (Icon: Icon). |
| `If-Then-Else` | ✅ Supported | Returns: `Object|Array`; Parameters: (Condition: Boolean|Number, True Value: Object|Array, False Value: Object|Array). |
| `Index Of Array Value` | ✅ Supported | Returns: `Number`; Parameters: (Array: Array, Value: Object|Array). |
| `Index Of String Char` | ✅ Supported | Returns: `Number`; Parameters: (String: String, Character: String). |
| `Input Binding String` | ✅ Supported | Returns: `String`; Parameters: (Button: Button). |
| `Is Alive` | ✅ Supported | Returns: `Boolean`; Parameters: (player: Player). |
| `Is Assembling Heroes` | ✅ Supported | Returns: `Boolean`. |
| `Is Between Rounds` | ✅ Supported | Returns: `Boolean`. |
| `Is Button Held` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player, Button: Button). |
| `Is CTF Mode In Sudden Death` | ✅ Supported | Returns: `Boolean`. |
| `Is Communicating` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player, Comms: Comms). |
| `Is Communicating Any` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Communicating Any Emote` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Communicating Any Spray` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Communicating Any Voice line` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Control Mode Point Locked` | ✅ Supported | Returns: `Boolean`. |
| `Is Crouching` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Dead` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Dummy Bot` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Duplicating` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Firing Primary` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Firing Secondary` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Firing Secondary Fire` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Flag At Base` | ✅ Supported | Returns: `Boolean`; Parameters: (Team: Team). |
| `Is Flag Being Carried` | ✅ Supported | Returns: `Boolean`; Parameters: (Team: Team). |
| `Is Game In Progress` | ✅ Supported | Returns: `Boolean`. |
| `Is Hero Being Played` | ✅ Supported | Returns: `Boolean`; Parameters: (Hero: Hero, Team: Team). |
| `Is In Air` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is In Alternate Form` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is In Line of Sight` | ✅ Supported | Returns: `Boolean`; Parameters: (Start Position: Vector|Player, End Position: Vector|Player, Barriers: BarrierLos). |
| `Is In Setup` | ✅ Supported | Returns: `Boolean`. |
| `Is In Spawn Room` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is In View Angle` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player, Location: Vector, ViewAngle: Number). |
| `Is Jumping` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Match Complete` | ✅ Supported | Returns: `Boolean`. |
| `Is Meleeing` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Moving` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Objective Complete` | ✅ Supported | Returns: `Boolean`; Parameters: (Number: Number). |
| `Is On Ground` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is On Objective` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is On Wall` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Portrait On Fire` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Reloading` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Standing` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Team On Defense` | ✅ Supported | Returns: `Boolean`; Parameters: (Team: Team). |
| `Is Team On Offense` | ✅ Supported | Returns: `Boolean`; Parameters: (Team: Team). |
| `Is True For All` | ✅ Supported | Returns: `Boolean`; Parameters: (Array: Array, Condition: Boolean). |
| `Is True For Any` | ✅ Supported | Returns: `Boolean`; Parameters: (Array: Array, Condition: Boolean). |
| `Is Using Ability 1` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Using Ability 2` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Using Ultimate` | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player|Array). |
| `Is Waiting For Players` | ✅ Supported | Returns: `Boolean`. |
| `Last Assist ID` | ✅ Supported | Returns: `AssistId`. |
| `Last Created Entity` | ✅ Supported | Returns: `EntityId`. |
| `Last Created Health Pool` | ✅ Supported | Returns: `HealthPoolId`. |
| `Last Damage Modification ID` | ✅ Supported | Returns: `DamageModificationId`. |
| `Last Damage Over Time ID` | ✅ Supported | Returns: `DotId`. |
| `Last Heal Over Time ID` | ✅ Supported | Returns: `HotId`. |
| `Last Healing Modification ID` | ✅ Supported | Returns: `HealingModificationId`. |
| `Last Of` | ✅ Supported | Returns: `Any`; Parameters: (Array: Array). |
| `Last Text ID` | ✅ Supported | Returns: `TextId`. |
| `Left` | ✅ Supported | Returns: `Array`. |
| `Local Player` | ✅ Supported | Returns: `Player`. |
| `Local Vector Of` | ✅ Supported | Returns: `Vector`; Parameters: (Vector: Vector, Player: Player, Transform: Transform). |
| `Magnitude Of` | ✅ Supported | Returns: `Number`; Parameters: (Vector: Vector). |
| `Mapped Array` | ✅ Supported | Returns: `Array`; Parameters: (Array: Array, Map: Object|Array). |
| `Match Round` | ✅ Supported | Returns: `Number`. |
| `Match Time` | ✅ Supported | Returns: `Number`. |
| `Max` | ✅ Supported | Returns: `Number`; Parameters: (Value: Number|Boolean, Value: Number|Boolean). |
| `Max Ammo` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Clip: Number). |
| `Max Health` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Max Health Of Type` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Health: Health). |
| `Min` | ✅ Supported | Returns: `Number`; Parameters: (Value: Number|Boolean, Value: Number|Boolean). |
| `Modulo` | ✅ Supported | Returns: `Number`; Parameters: (Value: Number, Value: Number). |
| `Multiply` | ✅ Supported | Returns: `Number|Vector`; Parameters: (a: Number|Boolean|Vector, b: Number|Boolean|Vector). |
| `Nearest Walkable Position` | ✅ Supported | Returns: `Vector`; Parameters: (Position: Vector). |
| `Normalize` | ✅ Supported | Returns: `Vector`; Parameters: (Vector: Vector). |
| `Normalized Health` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Not` | ✅ Supported | Returns: `Boolean`; Parameters: (Value: Boolean|Number). |
| `Null` | ✅ Supported | Returns: `Player`. |
| `Number Of Dead Players` | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Number Of Deaths` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Number Of Eliminations` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Number Of Final Blows` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Number Of Heroes` | ✅ Supported | Returns: `Number`; Parameters: (Hero: Hero, Team: Team). |
| `Number Of Living Players` | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Number Of Players` | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Number Of Players On Objective` | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Number Of Slots` | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Objective Index` | ✅ Supported | Returns: `Number`. |
| `Objective Position` | ✅ Supported | Returns: `Vector`; Parameters: (Objective: Any). |
| `Opposite Team Of` | ✅ Supported | Returns: `Team`; Parameters: (Team: Team). |
| `Or` | ✅ Supported | Returns: `Boolean`; Parameters: (A: Boolean|Number|Object|Array, B: Boolean|Number|Object|Array). |
| `Payload Position` | ✅ Supported | Returns: `Vector`. |
| `Payload Progress Percentage` | ✅ Supported | Returns: `Number`. |
| `Player Carrying Flag` | ✅ Supported | Returns: `Player`; Parameters: (Team: Team). |
| `Player Closest To Reticle` | ✅ Supported | Returns: `Player`; Parameters: (Player: Player, Team: Team). |
| `Player Hero Stat` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Hero: Hero, Statistic: HeroStat). |
| `Player Stat` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Statistic: Stat). |
| `Players In Slot` | ✅ Supported | Returns: `Player|Array`; Parameters: (Number: Number, Team: Team). |
| `Players On Hero` | ✅ Supported | Returns: `Array`; Parameters: (Hero: Hero, Team: Team). |
| `Players Within Radius` | ✅ Supported | Returns: `Array`; Parameters: (center: Vector, radius: Number, team: Team, losCheck: LosCheck). |
| `Players in View Angle` | ✅ Supported | Returns: `Array`; Parameters: (Player: Player, Team: Team, ViewAngle: Number). |
| `Point Capture Percentage` | ✅ Supported | Returns: `Number`. |
| `Position Of` | ✅ Supported | Returns: `Vector`; Parameters: (player: Player|Array). |
| `Random Integer` | ✅ Supported | Returns: `Number`; Parameters: (Min: Number|Boolean, Max: Number|Boolean). |
| `Random Real` | ✅ Supported | Returns: `Number`; Parameters: (Min: Number, Max: Number). |
| `Raise To Power` (Value) | 🚧 Coming soon | External Value contract: returns `Number`; parameters: (Value: Number, Value: Number). This is separate from the supported operator and variable-modification operation above. |
| `Random Value In Array` | ✅ Supported | Returns: `Any`; Parameters: (Array: Array). |
| `Randomized Array` | 🚧 Coming soon | External Value contract: returns `Array`; parameters: (Array: Array). |
| `Ray Cast Hit Normal` | ✅ Supported | Returns: `Vector`; Parameters: (Position: Vector, Position: Vector, Player: Array, Player: Array, Boolean: Boolean). |
| `Ray Cast Hit Player` | ✅ Supported | Returns: `Player`; Parameters: (Position: Vector, Position: Vector, Player: Array, Player: Array, Boolean: Boolean). |
| `Ray Cast Hit Position` | ✅ Supported | Returns: `Vector`; Parameters: (Start Position: Vector, End Position: Vector, Players To Include: Array, Players To Exclude: Array, Include Player Owned Objects: Boolean). |
| `Remove From Array` | ✅ Supported | Returns: `Array`; Parameters: (Array: Array, Value: Object|Array). |
| `Right` | ✅ Supported | Returns: `Array`. |
| `Round To Integer` | ✅ Supported | Returns: `Number`; Parameters: (Value: Number, Rounding: Rounding). |
| `Score Of` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Server Load` | ✅ Supported | Returns: `Number`. |
| `Server Load Average` | ✅ Supported | Returns: `Number`. |
| `Server Load Peak` | ✅ Supported | Returns: `Number`. |
| `Sine From Degrees` | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Sine From Radians` | ✅ Supported | Returns: `Number`; Parameters: (Value: Number). |
| `Slot Of` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Sorted Array` | ✅ Supported | Returns: `Array`; Parameters: (Array: Array, Sort: Object|Array). |
| `Spawn Points` | ✅ Supported | Returns: `Array`; Parameters: (Team: Team). |
| `Speed Of` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Speed Of In Direction` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Direction: Vector). |
| `Square Root` | ✅ Supported | Returns: `Number`; Parameters: (value: Number). |
| `String Contains` | ✅ Supported | Returns: `Boolean`; Parameters: (String: String, String: String). |
| `String Length` | ✅ Supported | Returns: `Number`; Parameters: (String: String). |
| `String Replace` | ✅ Supported | Returns: `String`; Parameters: (String: String|Array, Search: String|Array, Replacement: String|Array). |
| `String Slice` | ✅ Supported | Returns: `String`; Parameters: (String: String, Start Index: Number, Count: Number). |
| `String Split` | ✅ Supported | Returns: `Array`; Parameters: (String: String|Array, Separator: String|Array). |
| `String` | 🚧 Coming soon | External Value contract: `String(String, {0}, {1}, {2})`; `Custom String` remains supported. |
| `Subtract` | ✅ Supported | Returns: `Number|Vector`; Parameters: (a: Number|Boolean|Vector, b: Number|Boolean|Vector). |
| `Tangent From Degrees` | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Tangent From Radians` | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Team Of` | ✅ Supported | Returns: `Team`; Parameters: (Player: Player). |
| `Team Score` | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Text Count` | ✅ Supported | Returns: `Number`. |
| `Throttle Of` | ✅ Supported | Returns: `Vector`; Parameters: (player: Player). |
| `Total Time Elapsed` | ✅ Supported | Returns: `Number`. |
| `Ultimate Charge Percent` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Up` | ✅ Supported | Returns: `Array`. |
| `Update Every Frame` | ✅ Supported | Returns: `Object|Array`; Parameters: (Value: Object|Array). |
| `Value In Array` | ✅ Supported | Returns: `Any`; Parameters: (array: Any, index: Any). |
| `Vector` | ✅ Supported | Returns: `Vector`; Parameters: (X: Number|Boolean, Y: Number|Boolean, Z: Number|Boolean). |
| `Vector Towards` | ✅ Supported | Returns: `Vector`; Parameters: (Position: Any, Position: Any). |
| `Velocity Of` | ✅ Supported | Returns: `Vector`; Parameters: (Player: Player). |
| `Vertical Angle From Direction` | ✅ Supported | Returns: `Number`; Parameters: (Direction: Vector). |
| `Vertical Angle Towards` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Position: Vector). |
| `Vertical Facing Angle Of` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Vertical Speed Of` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Victim` | ✅ Supported | Returns: `Player`. |
| `Weapon` | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Workshop Setting Combo` | ✅ Supported | Returns: `Number`; Parameters: (Category: String, Name: String, Default: Number, Options: Array, SortOrder: Number). |
| `Workshop Setting Hero` | ✅ Supported | Returns: `Hero`; Parameters: (CustomStringLiteral: String, CustomStringLiteral: String, HeroLiteral: Hero, IntLiteral: Number). |
| `Workshop Setting Integer` | ✅ Supported | Returns: `Number`; Parameters: (Category: String, Name: String, Default: Number, MinValue: Number, MaxValue: Number, SortOrder: Number). |
| `Workshop Setting Real` | ✅ Supported | Returns: `Number`; Parameters: (CustomStringLiteral: String, CustomStringLiteral: String, FloatLiteral: Number, FloatLiteral: Number, FloatLiteral: Number, IntLiteral: Number). |
| `Workshop Setting Toggle` | ✅ Supported | Returns: `Boolean`; Parameters: (Category: String, Name: String, Default: Boolean, SortOrder: Number). |
| `World Vector Of` | ✅ Supported | Returns: `Vector`; Parameters: (localVector: Vector, relativePlayer: Player, transformation: Transform). |
| `X Component Of` | ✅ Supported | Returns: `Number`; Parameters: (Vector: Vector). |
| `Y Component Of` | ✅ Supported | Returns: `Number`; Parameters: (Vector: Vector). |
| `Z Component Of` | ✅ Supported | Returns: `Number`; Parameters: (Vector: Vector). |

## 8. Enumerated Domains

All 52 canonical Workshop enumerated domains are supported:

| Feature | Status | Notes |
| --- | --- | --- |
| `AccelReeval` | ✅ Supported | 2 members, including `Direction Rate and Max Speed`, `None` (2 total members). |
| `AssistReeval` | ✅ Supported | 2 members, including `Assisters and Targets`, `None` (2 total members). |
| `BarrierLos` | ✅ Supported | 3 members, including `Barriers Do Not Block LOS`, `Enemy Barriers Block LOS`, `All Barriers Block LOS` (3 total members). |
| `Beam` | ✅ Supported | 20 members, including `Grapple Beam`, `Good Beam`, `Bad Beam`, `Brigitte Flail Chain Beam`, … (20 total members). |
| `Button` | ✅ Supported | 10 members, including `Primary Fire`, `Secondary Fire`, `Ability 1`, `Ability 2`, … (10 total members). |
| `ChaseRateReeval` | ✅ Supported | 2 members, including `None`, `Destination and Rate` (2 total members). |
| `ChaseTimeReeval` | ✅ Supported | 2 members, including `None`, `Destination and Duration` (2 total members). |
| `Clipping` | ✅ Supported | 2 members, including `Do Not Clip`, `Clip Against Surfaces` (2 total members). |
| `Color` | ✅ Supported | 17 members, including `Yellow`, `White`, `Red`, `Orange`, … (17 total members). |
| `Comms` | ✅ Supported | 36 members, including `Acknowledge`, `Attacking`, `Countdown`, `Defending`, … (36 total members). |
| `DamageReeval` | ✅ Supported | 3 members, including `None`, `Receivers and Damagers`, `Receivers Damagers and Damage Percent` (3 total members). |
| `DynamicEffect` | ✅ Supported | 100 members, including `Bad Explosion`, `Buff Impact Sound`, `Debuff Impact Sound`, `Buff Explosion Sound`, … (100 total members). |
| `Effect` | ✅ Supported | 71 members, including `Orb`, `Bad Aura`, `Ring`, `Baptiste Immortality Field Protected Effect`, … (71 total members). |
| `EffectReeval` | ✅ Supported | 8 members, including `None`, `Visible To Position and Radius`, `Visible To`, `Color`, … (8 total members). |
| `EventPlayer` | ✅ Supported | 13 members, including `All`, `Slot 0`, `Slot 1`, `Slot 2`, … (13 total members). |
| `EventTeam` | ✅ Supported | 3 members, including `All`, `Team 1`, `Team 2` (3 total members). |
| `FacingReeval` | ✅ Supported | 2 members, including `Direction and Turn Rate`, `None` (2 total members). |
| `Gamemode` | ✅ Supported | 35 members, including `Assault`, `Bounty Hunter`, `Clash`, `Control`, … (35 total members). |
| `HealingReeval` | ✅ Supported | 3 members, including `None`, `Receivers and Healers`, `Receivers Healers and Healing Percent` (3 total members). |
| `Health` | ✅ Supported | 3 members, including `Armor`, `Shields`, `Health` (3 total members). |
| `Hero` | ✅ Supported | 53 members, including `D.Va`, `Orisa`, `Reinhardt`, `Roadhog`, … (53 total members). |
| `HeroStat` | ✅ Supported | 33 members, including `Healing Dealt`, `All Damage Dealt`, `Barrier Damage Dealt`, `Critical Hit Accuracy`, … (33 total members). |
| `HudPosition` | ✅ Supported | 3 members, including `Left`, `Right`, `Top` (3 total members). |
| `HudReeval` | ✅ Supported | 16 members, including `Visible To Sort Order String and Color`, `Visible To and String`, `Visible To`, `Visible To String and Color`, … (16 total members). |
| `Icon` | ✅ Supported | 36 members, including `No`, `Question Mark`, `Skull`, `Checkmark`, … (36 total members). |
| `IconReeval` | ✅ Supported | 8 members, including `Visible To and Position`, `Color`, `None`, `Position`, … (8 total members). |
| `Impulse` | ✅ Supported | 3 members, including `Cancel Contrary Motion`, `Cancel Contrary Motion XYZ`, `Incorporate Contrary Motion` (3 total members). |
| `Invis` | ✅ Supported | 3 members, including `All`, `Enemies`, `None` (3 total members). |
| `InworldTextReeval` | ✅ Supported | 9 members, including `Visible To`, `Visible To and Color`, `Visible To and Position`, `Visible To and String`, … (9 total members). |
| `LosCheck` | ✅ Supported | 4 members, including `Off`, `Surfaces`, `Surfaces And All Barriers`, `Surfaces And Enemy Barriers` (4 total members). |
| `Map` | ✅ Supported | 91 members, including `Ayutthaya`, `Black Forest`, `Castillo`, `Château Guillard`, … (91 total members). |
| `ModifyHealth` | ✅ Supported | 2 members, including `Damage`, `Heal` (2 total members). |
| `Operation` | ✅ Supported | 3 members, including `Append To Array`, `Remove From Array By Value`, `Remove From Array By Index` (3 total members). |
| `OutlineVisibility` | ✅ Supported | 3 members, including `Always`, `Default`, `Occluded` (3 total members). |
| `ProgressBarWorldReeval` | ✅ Supported | 1 members, including `Visible To And Values` (1 total members). |
| `ProgressHudReeval` | ✅ Supported | 8 members, including `Color`, `None`, `Values`, `Values and Color`, … (8 total members). |
| `ProgressWorldTextReeval` | ✅ Supported | 16 members, including `Color`, `None`, `Position`, `Position and Color`, … (16 total members). |
| `Projectile` | ✅ Supported | 20 members, including `Orb Projectile`, `Baptiste Biotic Launcher`, `Bastion A-36 Tactical Grenade`, `Echo Sticky Bomb`, … (20 total members). |
| `ProjectileEffectReeval` | ✅ Supported | 8 members, including `Visible To Position Direction and Size`, `Position Direction and Size`, `Visible To`, `None`, … (8 total members). |
| `Relativity` | ✅ Supported | 2 members, including `To World`, `To Player` (2 total members). |
| `Rounding` | ✅ Supported | 3 members, including `Up`, `Down`, `Nearest` (3 total members). |
| `SpecVisibility` | ✅ Supported | 3 members, including `Default Visibility`, `Visible Always`, `Visible Never` (3 total members). |
| `StartRuleBehavior` | ✅ Supported | 2 members, including `Restart Rule`, `Do Nothing` (2 total members). |
| `Stat` | ✅ Supported | 20 members, including `Healing Dealt`, `All Damage Dealt`, `Barrier Damage Dealt`, `Damage Blocked`, … (20 total members). |
| `Status` | ✅ Supported | 10 members, including `Asleep`, `Burning`, `Frozen`, `Hacked`, … (10 total members). |
| `Team` | ✅ Supported | 3 members, including `All Teams`, `Team 1`, `Team 2` (3 total members). |
| `Throttle` | ✅ Supported | 2 members, including `Replace existing throttle`, `Add to existing throttle` (2 total members). |
| `ThrottleReeval` | ✅ Supported | 2 members, including `Direction and Magnitude`, `None` (2 total members). |
| `Transform` | ✅ Supported | 2 members, including `Rotation`, `Rotation And Translation` (2 total members). |
| `Vector` | ✅ Supported | 6 members, including `Up`, `Down`, `Left`, `Right`, … (6 total members). |
| `Wait` | ✅ Supported | 3 members, including `Ignore Condition`, `Abort When False`, `Restart When True` (3 total members). |
| `WorldTextReeval` | ✅ Supported | 12 members, including `Color`, `None`, `String`, `String and Color`, … (12 total members). |

## 9. Custom-Game Settings

| Feature | Status | Notes |
| --- | --- | --- |
| `main` (Main settings) | ✅ Supported | Custom game mode name and description strings. |
| `lobby` (Lobby settings) | ✅ Supported | Team size, match start rules, spectator settings, map rotation, and lobby options. |
| `modes` (Mode settings) | ✅ Supported | General mode parameters and individual game modes (Assault, Control, Escort, Hybrid, Push, Flashpoint, Clash, Deathmatch, Team Deathmatch, CTF, Elimination, etc.) and map pools (`enabled maps` / `disabled maps`). |
| `heroes` (Hero settings) | ✅ Supported | Global hero rules, roster toggles (`enabled heroes` / `disabled heroes`), role limits, and per-hero ability/weapon/cooldown parameters. |
| `extensions` (Workshop extensions) | ✅ Supported | Extension flags (`Beam Effects`, `Buff Status Effects`, `Debuff Status Effects`, `Buff and Debuff Sounds`, `Energy Explosion Effects`, `Kinetic Explosion Effects`, `Play More Effects`, `Spawn More Dummy Bots`). |
| `workshop` (Custom workshop settings) | ✅ Supported | User-defined custom settings defined via `Workshop Setting ...` values in rules. |

## 10. Strings & Localization

| Feature | Status | Notes |
| --- | --- | --- |
| `Custom String` format strings | ✅ Supported | Format strings with up to 3 interpolation placeholders (`{0}`, `{1}`, `{2}`) and recursive formatting. |
| Built-in localized `String` values | ✅ Supported | Standard localized Workshop preset string identifiers. |
| `en-US` client locale | ✅ Supported | Primary locale with 100% complete catalog and syntax coverage for parsing, emission, and conversion. |
| `zh-CN` client locale | ✅ Supported | Reviewed canonical localization with high coverage for parsing, emission, and conversion. |
| Additional client locales (`ko-KR`, `ja-JP`, `de-DE`, `fr-FR`, `es-ES`, etc.) | 🚧 Coming soon | Planned for addition upon ingestion of provenance-reviewed game client datasets. |
| Bidirectional conversion (`en-US` ↔ `zh-CN`) | ✅ Supported | Strict conversion with explicit error on unmapped identities, plus opt-in fallback to primary locale. |

## 11. Tooling & Semantic Capabilities

| Feature | Status | Notes |
| --- | --- | --- |
| Raw Workshop parsing | ✅ Supported | Parses raw text into syntax trees with source location spans. |
| Semantic validation | ✅ Supported | Validates arity, parameter types, enum domains, and variable/subroutine declarations. |
| Deterministic code emission | ✅ Supported | Formats and emits canonical Workshop code deterministically for supported locales. |
| Workshop language conversion | ✅ Supported | Bidirectional conversion between supported client locales (`en-US` ↔ `zh-CN`). |
| Hero gameplay & semantic query API | ✅ Supported | Query hero abilities, slots, variants, custom-game modifiers, and cooldown calculations. |
| Offline feature census & conformance testing | ✅ Supported | Sharded offline census and regression runner for compatibility tracking. |

## 12. Intentionally Out-of-Scope Capabilities

| Feature | Status | Notes |
| --- | --- | --- |
| Live Workshop runtime / VM simulation | ❌ Unsupported | `workshop-rs` is a compiler and semantic analysis engine; executing gameplay simulation in real-time is an Overwatch engine function. |
| Source-language syntax (OverPy / DEL / OSTW) | ❌ Unsupported | OverPy syntax and macros belong to `opy-rs`; DEL/OSTW syntax and features belong to `del-rs`. |
| Dynamic script evaluation (`eval`) | ❌ Unsupported | Workshop language semantics do not include dynamic code evaluation or runtime code generation. |
