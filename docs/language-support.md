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
| `settings` block (`settings`) | ✅ Supported | Top-level custom game settings block containing lobby, modes, heroes, extensions, and workshop parameters. |
| `variables` block (`variables`) | ✅ Supported | Top-level variable declaration block declaring `global` and `player` variable indices. |
| `subroutines` block (`subroutines`) | ✅ Supported | Top-level subroutine declaration block declaring subroutine names and indices. |
| `rule` declaration (`rule`) | ✅ Supported | Rule declaration `rule ("Rule Name") { ... }` with event, conditions, and actions blocks. |
| `event` block (`event`) | ✅ Supported | Rule header block defining triggering event type and applicable filters. |
| `conditions` block (`conditions`) | ✅ Supported | Rule precondition block evaluated before rule actions run. |
| `actions` block (`actions`) | ✅ Supported | Rule action sequence executed when event triggers and conditions pass. |
| `disabled` rule modifier (`disabled`) | ✅ Supported | `disabled rule ("...")` disables execution of the entire rule. |
| `disabled` condition modifier | ✅ Supported | `disabled <condition>;` skips evaluation of a single condition within a rule. |
| `disabled` action modifier | ✅ Supported | `disabled <action>;` skips execution of a single action within a rule. |

## 2. Variables & Subroutines

| Feature | Status | Notes |
| --- | --- | --- |
| `global` variable declaration (`global`) | ✅ Supported | Numerical indexed declaration in `variables { global: 0: varName }`. |
| `global` variable read (`Global Variable`) | ✅ Supported | Read access via `Global.var` or `Global Variable(var)`. |
| `Set Global Variable` (`setGlobalVariable`) | ✅ Supported | Assigns a value to a global variable: `Set Global Variable(var, value)`. |
| `Modify Global Variable` (`modifyGlobalVariable`) | ✅ Supported | Modifies a global variable with an operation: `Modify Global Variable(var, operation, value)`. |
| `Set Global Variable At Index` (`setGlobalVariableAtIndex`) | ✅ Supported | Assigns a value into an array global variable at index: `Set Global Variable At Index(var, index, value)`. |
| `Modify Global Variable At Index` (`modifyGlobalVariableAtIndex`) | ✅ Supported | Modifies an array global variable element at index: `Modify Global Variable At Index(var, index, operation, value)`. |
| `player` variable declaration (`player`) | ✅ Supported | Numerical indexed declaration in `variables { player: 0: varName }`. |
| `player` variable read (`Player Variable`) | ✅ Supported | Read access via `Player.var` or `Player Variable(player, var)`. |
| `Set Player Variable` (`setPlayerVariable`) | ✅ Supported | Assigns a value to a player variable: `Set Player Variable(player, var, value)`. |
| `Modify Player Variable` (`modifyPlayerVariable`) | ✅ Supported | Modifies a player variable with an operation: `Modify Player Variable(player, var, operation, value)`. |
| `Set Player Variable At Index` (`setPlayerVariableAtIndex`) | ✅ Supported | Assigns a value into an array player variable at index: `Set Player Variable At Index(player, var, index, value)`. |
| `Modify Player Variable At Index` (`modifyPlayerVariableAtIndex`) | ✅ Supported | Modifies an array player variable element at index: `Modify Player Variable At Index(player, var, index, operation, value)`. |
| `subroutines` declaration (`subroutines`) | ✅ Supported | Numerical indexed declaration in `subroutines { 0: SubName }`. |
| `Call Subroutine` (`callSubroutine`) | ✅ Supported | Synchronous subroutine call: `Call Subroutine(SubName)`. |
| `Start Rule` (`startRule`) | ✅ Supported | Asynchronous subroutine rule invocation: `Start Rule(SubName, RestartBehavior)`. |
| `Subroutine` event rule (`subroutine`) | ✅ Supported | Rule event block `event { Subroutine; SubName; }` triggered by subroutine calls. |

## 3. Events & Event Filters

| Feature | Status | Notes |
| --- | --- | --- |
| `Ongoing - Each Player` (`eachPlayer`) | ✅ Supported | Filters: Team, Player. |
| `Ongoing - Global` (`global`) | ✅ Supported | Global event (no filters). |
| `Player Dealt Damage` (`playerDealtDamage`) | ✅ Supported | Filters: Team, Player. |
| `Player Dealt Final Blow` (`playerDealtFinalBlow`) | ✅ Supported | Filters: Team, Player. |
| `Player Dealt Healing` (`playerDealtHealing`) | ✅ Supported | Filters: Team, Player. |
| `Player Dealt Knockback` (`playerDealtKnockback`) | ✅ Supported | Filters: Team, Player. |
| `Player Died` (`playerDied`) | ✅ Supported | Filters: Team, Player. |
| `Player Earned Elimination` (`playerEarnedElimination`) | ✅ Supported | Filters: Team, Player. |
| `Player Joined Match` (`playerJoined`) | ✅ Supported | Filters: Team, Player. |
| `Player Left Match` (`playerLeft`) | ✅ Supported | Filters: Team, Player. |
| `Player Received Healing` (`playerReceivedHealing`) | ✅ Supported | Filters: Team, Player. |
| `Player Took Damage` (`playerTookDamage`) | ✅ Supported | Filters: Team, Player. |
| `Player Took Knockback` (`playerReceivedKnockback`) | ✅ Supported | Filters: Team, Player. |
| `Subroutine` (`subroutine`) | ✅ Supported | Global event (no filters). |
| `Team` event filter (`EventTeam`) | ✅ Supported | Filters player events by team (`All`, `Team 1`, `Team 2`). |
| `Player / Hero` event filter (`EventPlayer`) | ✅ Supported | Filters player events by slot (`All`, `Slot 0`..`Slot 11`) or specific hero. |
| `Subroutine` name filter | ✅ Supported | Identifies the target subroutine name for subroutine event rules. |

## 4. Conditions & Control Flow

| Feature | Status | Notes |
| --- | --- | --- |
| `Comparison condition` | ✅ Supported | Binary condition expression (`Value OP Value`) with implicit logical AND across rule conditions. |
| `If` (`if`) | ✅ Supported | Begins a conditional execution block: `If(Condition);`. |
| `Else If` (`elseIf`) | ✅ Supported | Secondary conditional execution block: `Else If(Condition);`. |
| `Else` (`else`) | ✅ Supported | Fallback conditional execution block: `Else;`. |
| `End` (`end`) | ✅ Supported | Terminates an `If`, `While`, or `For` block (including oracle-compatible implicit trailing `End`). |
| `While` (`while`) | ✅ Supported | Loop block executed while condition evaluates to true: `While(Condition); ... End;`. |
| `For Global Variable` (`forGlobalVariable`) | ✅ Supported | Loop iterating a global variable from start to stop with step: `For Global Variable(var, start, stop, step); ... End;`. |
| `For Player Variable` (`forPlayerVariable`) | ✅ Supported | Loop iterating a player variable from start to stop with step: `For Player Variable(player, var, start, stop, step); ... End;`. |
| `Loop` (`loop`) | ✅ Supported | Restarts rule execution from the first action. |
| `Loop If` (`loopIf`) | ✅ Supported | Restarts rule execution if condition evaluates to true. |
| `Loop If Condition Is True` (`loopIfConditionIsTrue`) | ✅ Supported | Restarts rule execution if all rule conditions are currently true. |
| `Loop If Condition Is False` (`loopIfConditionIsFalse`) | ✅ Supported | Restarts rule execution if any rule condition is currently false. |
| `Break` (`break`) | ✅ Supported | Breaks out of the innermost `While` or `For` loop. |
| `Continue` (`continue`) | ✅ Supported | Advances to the next iteration of the innermost `While` or `For` loop. |
| `Skip` (`skip`) | ✅ Supported | Unconditionally skips the specified number of subsequent actions. |
| `Skip If` (`skipIf`) | ✅ Supported | Skips subsequent actions if condition evaluates to true. |
| `Wait` (`wait`) | ✅ Supported | Pauses execution for a duration with condition restart/abort behavior. |
| `Wait Until` (`waitUntil`) | ✅ Supported | Pauses execution until condition evaluates to true or timeout expires. |
| `Abort` (`abort`) | ✅ Supported | Immediately terminates rule action execution. |
| `Abort If` (`abortIf`) | ✅ Supported | Terminates rule action execution if condition evaluates to true. |
| `Abort If Condition Is True` (`abortIfConditionIsTrue`) | ✅ Supported | Terminates rule execution if all rule conditions are currently true. |
| `Abort If Condition Is False` (`abortIfConditionIsFalse`) | ✅ Supported | Terminates rule execution if any rule condition is currently false. |
| `Return` (`return`) | ✅ Supported | Returns from a subroutine execution rule. |

## 5. Operators & Variable Modifications

| Feature | Status | Notes |
| --- | --- | --- |
| `!=` (`!=`) | ✅ Supported | Comparison operator in conditions and `Compare` value. |
| `<` (`<`) | ✅ Supported | Comparison operator in conditions and `Compare` value. |
| `<=` (`<=`) | ✅ Supported | Comparison operator in conditions and `Compare` value. |
| `==` (`==`) | ✅ Supported | Comparison operator in conditions and `Compare` value. |
| `>` (`>`) | ✅ Supported | Comparison operator in conditions and `Compare` value. |
| `>=` (`>=`) | ✅ Supported | Comparison operator in conditions and `Compare` value. |
| `Add` (`add`) | ✅ Supported | Arithmetic operator and variable modification operation. |
| `Append To Array` (`appendToArray`) | ✅ Supported | Array variable modification operation. |
| `Divide` (`divide`) | ✅ Supported | Arithmetic operator and variable modification operation. |
| `Modulo` (`modulo`) | ✅ Supported | Arithmetic operator and variable modification operation. |
| `Multiply` (`multiply`) | ✅ Supported | Arithmetic operator and variable modification operation. |
| `Raise To Power` (`raiseToPower`) | ✅ Supported | Arithmetic operator and variable modification operation. |
| `Remove From Array` (`removeFromArray`) | ✅ Supported | Array variable modification operation. |
| `Remove From Array By Index` (`removeFromArrayByIndex`) | ✅ Supported | Array variable modification operation. |
| `Remove From Array By Value` (`removeFromArrayByValue`) | ✅ Supported | Array variable modification operation. |
| `Subtract` (`subtract`) | ✅ Supported | Arithmetic operator and variable modification operation. |

## 6. Actions Inventory

All 219 canonical Workshop actions are supported:

| Feature | Status | Notes |
| --- | --- | --- |
| `Abort` (`abort`) | ✅ Supported | No parameters. |
| `Abort If` (`abortIf`) | ✅ Supported | Parameters: (Condition: Boolean). |
| `Abort If Condition Is False` (`__abortIfConditionIsFalse__`) | ✅ Supported | No parameters. |
| `Abort If Condition Is True` (`__abortIfConditionIsTrue__`) | ✅ Supported | No parameters. |
| `Add Health Pool To Player` (`addHealthPool`) | ✅ Supported | Parameters: (Object: Player|Array, Health: Health, Number: Number, Boolean: Boolean, Boolean: Boolean). |
| `Allow Button` (`allowButton`) | ✅ Supported | Parameters: (Player: Player|Array, Button: Button). |
| `Apply Impulse` (`applyImpulse`) | ✅ Supported | Parameters: (Player: Player|Array, Direction: Vector, Speed: Number, Relativity: Relativity, Impulse: Impulse). |
| `Attach Players` (`attachTo`) | ✅ Supported | Parameters: (Player: Player, Player: Player, Position: Vector). |
| `Big Message` (`bigMessage`) | ✅ Supported | Parameters: (VisibleTo: Player|Array, Header: Object). |
| `Break` (`break`) | ✅ Supported | No parameters. |
| `Cancel Primary Action` (`cancelPrimaryAction`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Chase Global Variable At Rate` (`chaseAtRate`) | ✅ Supported | Parameters: (Variable: Variable, Destination: Any, Rate: Number|Boolean, Reevaluation: ChaseRateReeval). |
| `Chase Global Variable Over Time` (`chaseOverTime`) | ✅ Supported | Parameters: (Variable: Variable, Destination: Any, Duration: Number, Reevaluation: ChaseTimeReeval). |
| `Chase Player Variable At Rate` (`chasePlayerVariableAtRate`) | ✅ Supported | Parameters: (Player: Player|Array, Variable: Variable, Destination: Any, Rate: Number, Reevaluation: ChaseRateReeval). |
| `Chase Player Variable Over Time` (`chasePlayerVariableOverTime`) | ✅ Supported | Parameters: (Player: Player|Array, Variable: Variable, Destination: Any, Duration: Number, Reevaluation: ChaseTimeReeval). |
| `Clear Status` (`clearStatusEffect`) | ✅ Supported | Parameters: (Object: Player|Array, Status: Status). |
| `Communicate` (`communicate`) | ✅ Supported | Parameters: (Object: Player|Array, Comms: Comms). |
| `Continue` (`continue`) | ✅ Supported | No parameters. |
| `Create Beam Effect` (`createBeamEffect`) | ✅ Supported | Parameters: (VisibleTo: Player|Array, Type: Beam, StartPosition: Vector, EndPosition: Vector, Color: Color, Reevaluation: EffectReeval). |
| `Create Dummy Bot` (`createDummyBot`) | ✅ Supported | Parameters: (Hero: Hero, Team: Team, Slot: Number|Boolean, Position: Vector, Direction: Vector). |
| `Create Effect` (`createEffect`) | ✅ Supported | Parameters: (VisibleTo: Player|Array, Type: Effect, Color: Color, Position: Vector|Player, Radius: Number, Reevaluation: EffectReeval). |
| `Create HUD Text` (`createHudText`) | ✅ Supported | Parameters: (VisibleTo: Player|Array, Header: Object|String, Subheader: Object|String, Text: Object|String, Location: HudPosition, SortOrder: Number|Boolean, HeaderColor: Color, SubheaderColor: Color, TextColor: Color, Reevaluation: HudReeval, Spectators: SpecVisibility). |
| `Create Homing Projectile` (`createHomingProjectile`) | ✅ Supported | Parameters: (Projectile: Projectile, Object: Player|Array, Position: Vector, Direction: Vector, Relativity: Relativity, ModifyHealth: ModifyHealth, Team: Team, Number: Number, Number: Number, Number: Number, DynamicEffect: DynamicEffect, DynamicEffect: DynamicEffect, Number: Number, Number: Number, Number: Number, Number: Number, Player: Player, Number: Number). |
| `Create Icon` (`createIcon`) | ✅ Supported | Parameters: (VisibleTo: Player|Array, Position: Vector|Player, Icon: Icon, Reevaluation: IconReeval, Color: Color, ShowWhenOffscreen: Boolean). |
| `Create In-World Text` (`createInWorldText`) | ✅ Supported | Parameters: (VisibleTo: Player|Array, Header: Object, Position: Vector|Player, Scale: Number, Clipping: Clipping, Reevaluation: WorldTextReeval, TextColor: Color, Spectators: SpecVisibility). |
| `Create Progress Bar HUD Text` (`progressBarHud`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Object: Object, HudPosition: HudPosition, Number: Number|Boolean, Color: Color, Color: Color, ProgressHudReeval: ProgressHudReeval, SpecVisibility: SpecVisibility). |
| `Create Progress Bar In-World Text` (`createProgressBarInWorldText`) | ✅ Supported | Parameters: (VisibleTo: Player|Array, Value: Number, Text: Object, Position: Vector|Player, Scale: Number, Clipping: Clipping, HeaderColor: Color, TextColor: Color, Reevaluation: ProgressWorldTextReeval, NonteamSpectators: SpecVisibility). |
| `Create Projectile` (`createProjectile`) | ✅ Supported | Parameters: (Projectile: Projectile, Object: Player|Array, Position: Vector, Direction: Vector, Relativity: Relativity, ModifyHealth: ModifyHealth, Team: Team, Number: Number, Number: Number, Number: Number, DynamicEffect: DynamicEffect, DynamicEffect: DynamicEffect, Number: Number, Number: Number, Number: Number, Number: Number, Number: Number, Number: Number). |
| `Create Projectile Effect` (`createProjectileEffect`) | ✅ Supported | Parameters: (Object: Player|Array, Projectile: Projectile, Object: Player|Array, Object: Vector|Player, Direction: Vector, Number: Number, ProjectileEffectReeval: ProjectileEffectReeval). |
| `Damage` (`damage`) | ✅ Supported | Parameters: (Object: Player|Array, Player: Player, Number: Number). |
| `Declare Match Draw` (`declareDraw`) | ✅ Supported | No parameters. |
| `Declare Player Victory` (`declarePlayerVictory`) | ✅ Supported | Parameters: (Player: Player). |
| `Declare Round Draw` (`declareRoundDraw`) | ✅ Supported | No parameters. |
| `Declare Round Victory` (`declareRoundVictory`) | ✅ Supported | Parameters: (Team: Team). |
| `Declare Team Victory` (`declareTeamVictory`) | ✅ Supported | Parameters: (Team: Team). |
| `Destroy All Dummy Bots` (`destroyAllDummies`) | ✅ Supported | No parameters. |
| `Destroy All Effects` (`destroyAllEffects`) | ✅ Supported | No parameters. |
| `Destroy All HUD Text` (`destroyAllHudText`) | ✅ Supported | No parameters. |
| `Destroy All Icons` (`destroyAllIcons`) | ✅ Supported | No parameters. |
| `Destroy All In-World Text` (`destroyAllInWorldText`) | ✅ Supported | No parameters. |
| `Destroy All Progress Bar HUD Text` (`destroyAllProgressBarHudText`) | ✅ Supported | No parameters. |
| `Destroy All Progress Bar In-World Text` (`destroyAllProgressBarInWorldText`) | ✅ Supported | No parameters. |
| `Destroy Dummy Bot` (`destroyDummy`) | ✅ Supported | Parameters: (Team: Team, Number: Number). |
| `Destroy Effect` (`destroyEffect`) | ✅ Supported | Parameters: (EffectId: EntityId). |
| `Destroy HUD Text` (`destroyHudText`) | ✅ Supported | Parameters: (TextId: TextId). |
| `Destroy Icon` (`destroyIcon`) | ✅ Supported | Parameters: (EntityId: EntityId). |
| `Destroy In-World Text` (`destroyInWorldText`) | ✅ Supported | Parameters: (TextId: TextId). |
| `Destroy Progress Bar HUD Text` (`destroyProgressBarHud`) | ✅ Supported | Parameters: (TextId: TextId). |
| `Destroy Progress Bar In-World Text` (`destroyProgressBarInWorldText`) | ✅ Supported | Parameters: (TextId: TextId). |
| `Detach Players` (`detach`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Built-In Game Mode Announcer` (`disableBuiltinGameModeAnnouncer`) | ✅ Supported | No parameters. |
| `Disable Built-In Game Mode Completion` (`disableBuiltinGameModeCompletion`) | ✅ Supported | No parameters. |
| `Disable Built-In Game Mode Music` (`disableBuiltinGameModeMusic`) | ✅ Supported | No parameters. |
| `Disable Built-In Game Mode Respawning` (`disableRespawn`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Built-In Game Mode Scoring` (`disableBuiltinGameModeScoring`) | ✅ Supported | No parameters. |
| `Disable Death Spectate All Players` (`disableDeathSpectateAllPlayers`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Death Spectate Target HUD` (`disableDeathSpectateTargetHud`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Game Mode HUD` (`disableGameModeHud`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Disable Game Mode In-World UI` (`disableGameModeInworldUI`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Disable Hero HUD` (`disableHeroHud`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Disable Inspector Recording` (`disableInspector`) | ✅ Supported | No parameters. |
| `Disable Kill Feed` (`disableKillFeed`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Messages` (`disableMessages`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Movement Collision With Environment` (`disableMovementCollisionWithEnvironment`) | ✅ Supported | Parameters: (Player: Player|Array, IncludeFloors: Boolean). |
| `Disable Movement Collision With Players` (`disableMovementCollisionWithPlayers`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Disable Nameplates` (`disableNameplatesFor`) | ✅ Supported | Parameters: (Object: Player|Array, Object: Player|Array). |
| `Disable Scoreboard` (`disableScoreboard`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Disable Text Chat` (`disableTextChat`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Disable Voice Chat` (`disableVoiceChat`) | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean, Boolean: Boolean, Boolean: Boolean). |
| `Disallow Button` (`disallowButton`) | ✅ Supported | Parameters: (Player: Player|Array, Button: Button). |
| `Enable Built-In Game Mode Announcer` (`enableAnnouncer`) | ✅ Supported | No parameters. |
| `Enable Built-In Game Mode Completion` (`enableGamemodeCompletion`) | ✅ Supported | No parameters. |
| `Enable Built-In Game Mode Music` (`enableBuiltinGameModeMusic`) | ✅ Supported | No parameters. |
| `Enable Built-In Game Mode Respawning` (`enableRespawn`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Enable Built-In Game Mode Scoring` (`enableScoring`) | ✅ Supported | No parameters. |
| `Enable Death Spectate All Players` (`enableDeathSpectateAllPlayers`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Enable Death Spectate Target HUD` (`enableDeathSpectateTargetHud`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Enable Game Mode HUD` (`enableGameModeHud`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Enable Game Mode In-World UI` (`enableGameModeInworldUI`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Enable Hero HUD` (`enableHeroHud`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Enable Inspector Recording` (`enableInspectorRecording`) | ✅ Supported | No parameters. |
| `Enable Kill Feed` (`enableKillFeed`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Enable Messages` (`enableMessages`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Enable Movement Collision With Environment` (`enableMovementCollisionWithEnvironment`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Enable Movement Collision With Players` (`enableMovementCollisionWithPlayers`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Enable Nameplates` (`enableNameplatesFor`) | ✅ Supported | Parameters: (Object: Player|Array, Object: Player|Array). |
| `Enable Scoreboard` (`enableScoreboard`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Enable Text Chat` (`enableTextChat`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Enable Voice Chat` (`enableVoiceChat`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `For Player Variable` (`__forPlayerVariable__`) | ✅ Supported | Parameters: (Player: Player, PlayerVariable: Variable, Number: Number, Number: Number, Number: Number). |
| `Force Player Hero` (`forcePlayerHero`) | ✅ Supported | Parameters: (Player, Hero). |
| `Force Throttle` (`forceThrottle`) | ✅ Supported | Parameters: (Player, MoveSpeed, InAirSpeed, SpectatorSpeed, GrappleBoost, JumpPower, MoveSpeed). |
| `Go To Assemble Heroes` (`goToAssembleHeroes`) | ✅ Supported | No parameters. |
| `Heal` (`heal`) | ✅ Supported | Parameters: (Object: Player|Array, Player: Player, Number: Number). |
| `Kill` (`kill`) | ✅ Supported | Parameters: (Object: Player|Array, Player: Player). |
| `Log To Inspector` (`logToInspector`) | ✅ Supported | Parameters: (Object: Object). |
| `Loop` (`loop`) | ✅ Supported | No parameters. |
| `Loop If` (`loopIf`) | ✅ Supported | Parameters: (Condition: Boolean). |
| `Loop If Condition Is False` (`__loopIfConditionIsFalse__`) | ✅ Supported | No parameters. |
| `Loop If Condition Is True` (`loopIfConditionIsTrue`) | ✅ Supported | No parameters. |
| `Modify Global Variable` (`modifyGlobalVariable`) | ✅ Supported | Parameters: (Variable: Variable, Operation: Operation, Value: Any). |
| `Modify Global Variable At Index` (`modifyGlobalVariableAtIndex`) | ✅ Supported | Parameters: (Variable: Global Variable, Index: Number|Boolean, Operation: Operation, Value: Object|Array). |
| `Modify Player Score` (`addToScore`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Modify Player Variable At Index` (`modifyPlayerVariableAtIndex`) | ✅ Supported | Parameters: (Variable: Player Variable, Index: Number|Boolean, Operation: Operation, Value: Object|Array). |
| `Modify Team Score` (`addToTeamScore`) | ✅ Supported | Parameters: (Team: Team, Number: Number). |
| `Move Player to Team` (`moveToTeam`) | ✅ Supported | Parameters: (Object: Player|Array, Team: Team, Number: Number). |
| `Pause Match Time` (`pauseMatchTime`) | ✅ Supported | No parameters. |
| `Play Effect` (`playEffect`) | ✅ Supported | Parameters: (VisibleTo: Player|Array, Type: DynamicEffect, Color: Color, Position: Vector, Radius: Number). |
| `Preload Hero` (`preloadHero`) | ✅ Supported | Parameters: (Object: Player|Array, Object: Hero|Array). |
| `Press Button` (`forceButtonPress`) | ✅ Supported | Parameters: (Object: Player|Array, Button: Button). |
| `Remove All Health Pools From Player` (`removeAllHealthPoolsFromPlayer`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Remove Health Pool From Player` (`removeHealthPool`) | ✅ Supported | Parameters: (HealthPoolId: HealthPoolId). |
| `Remove Player` (`removeFromGame`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Reset Player Hero Availability` (`resetHeroAvailability`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Respawn` (`respawn`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Restart Match` (`restartMatch`) | ✅ Supported | No parameters. |
| `Resurrect` (`resurrect`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Return To Lobby` (`returnToLobby`) | ✅ Supported | No parameters. |
| `Set Ability 1 Enabled` (`setAbility1Enabled`) | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Ability 2 Enabled` (`setAbility2Enabled`) | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Ability Charge` (`setAbilityCharge`) | ✅ Supported | Parameters: (Object: Player|Array, Button: Button, Number: Number). |
| `Set Ability Cooldown` (`setAbilityCooldown`) | ✅ Supported | Parameters: (Object: Player|Array, Button: Button, Number: Number|Boolean). |
| `Set Ability Resource` (`setAbilityResource`) | ✅ Supported | Parameters: (Object: Player|Array, Button: Button, Number: Number). |
| `Set Aim Speed` (`setAimSpeed`) | ✅ Supported | Parameters: (player: Player|Array, turnSpeedPercent: Number). |
| `Set Allowed Heroes` (`setAllowedHeroes`) | ✅ Supported | Parameters: (Player: Player|Array, Heroes: Hero|Array). |
| `Set Ammo` (`setAmmo`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Number: Number). |
| `Set Crouch Enabled` (`setCrouchEnabled`) | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Damage Dealt` (`setDamageDealt`) | ✅ Supported | Parameters: (player: Player|Array, damageDealtPercent: Number). |
| `Set Damage Received` (`setDamageReceived`) | ✅ Supported | Parameters: (player: Player|Array, damageReceivedPercent: Number). |
| `Set Environment Credit Player` (`setEnvironmentCreditPlayer`) | ✅ Supported | Parameters: (Player: Player|Array, Player: Player|Array). |
| `Set Facing` (`setFacing`) | ✅ Supported | Parameters: (Object: Player|Array, Direction: Vector, Relativity: Relativity). |
| `Set Global Variable At Index` (`setGlobalVariableAtIndex`) | ✅ Supported | Parameters: (Variable: Global Variable, Index: Number|Boolean, Value: Object|Array). |
| `Set Gravity` (`setGravity`) | ✅ Supported | Parameters: (Player: Player|Array, Gravity: Number|Boolean). |
| `Set Healing Dealt` (`setHealingDealt`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Healing Received` (`setHealingReceived`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Invisible` (`setInvisibility`) | ✅ Supported | Parameters: (Player: Player|Array, InvisibleTo: Invis). |
| `Set Jump Enabled` (`setJumpEnabled`) | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Jump Vertical Speed` (`setJumpVerticalSpeed`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Knockback Dealt` (`setKnockbackDealt`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Knockback Received` (`setKnockbackReceived`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Match Time` (`setMatchTime`) | ✅ Supported | Parameters: (Number: Number|Boolean). |
| `Set Max Ammo` (`setMaxAmmo`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Number: Number). |
| `Set Max Health` (`setMaxHealth`) | ✅ Supported | Parameters: (player: Player|Array, healthPercent: Number). |
| `Set Melee Enabled` (`setMeleeEnabled`) | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Move Speed` (`setMoveSpeed`) | ✅ Supported | Parameters: (player: Player|Array, moveSpeedPercent: Number). |
| `Set Objective Description` (`setObjectiveDescription`) | ✅ Supported | Parameters: (Object: Player|Array, Object: Object, HudReeval: HudReeval). |
| `Set Player Health` (`setHealth`) | ✅ Supported | Parameters: (player: Player|Array, amount: Number). |
| `Set Player Score` (`setScore`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Player Variable At Index` (`setPlayerVariableAtIndex`) | ✅ Supported | Parameters: (Variable: Player Variable, Index: Number|Boolean, Value: Object|Array). |
| `Set Primary Fire Enabled` (`setPrimaryFireEnabled`) | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Projectile Gravity` (`setProjectileGravity`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Projectile Speed` (`setProjectileSpeed`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Reload Enabled` (`setReloadEnabled`) | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Respawn Max Time` (`setRespawnTime`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Set Secondary Fire Enabled` (`setSecondaryFireEnabled`) | ✅ Supported | Parameters: (Player: Player|Array, bool: Boolean). |
| `Set Slow Motion` (`setSlowMotion`) | ✅ Supported | Parameters: (Number: Number). |
| `Set Status` (`setStatusEffect`) | ✅ Supported | Parameters: (player: Player|Array, assister: Player, status: Status, duration: Number). |
| `Set Team Score` (`setTeamScore`) | ✅ Supported | Parameters: (Team: Team, Number: Number). |
| `Set Ultimate Ability Enabled` (`setUltEnabled`) | ✅ Supported | Parameters: (Object: Player|Array, Boolean: Boolean). |
| `Set Ultimate Charge` (`setUltCharge`) | ✅ Supported | Parameters: (player: Player|Array, chargePercent: Number|Boolean). |
| `Set Weapon` (`setWeapon`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number). |
| `Skip` (`skip`) | ✅ Supported | Parameters: (Value: Number|Boolean). |
| `Skip If` (`skipIf`) | ✅ Supported | Parameters: (Condition: Boolean, Number: Number|Boolean). |
| `Small Message` (`smallMessage`) | ✅ Supported | Parameters: (VisibleTo: Player|Array, Header: Object). |
| `Start Accelerating` (`startAcceleration`) | ✅ Supported | Parameters: (Object: Player|Array, Direction: Vector, Number: Number, Number: Number, Relativity: Relativity, AccelReeval: AccelReeval). |
| `Start Assist` (`startGrantingAssistFor`) | ✅ Supported | Parameters: (Object: Player|Array, Object: Player|Array, AssistReeval: AssistReeval). |
| `Start Camera` (`startCamera`) | ✅ Supported | Parameters: (Player: Player|Array, EyePosition: Vector, LookAtPosition: Vector, Facing: Number). |
| `Start Damage Modification` (`startDamageModification`) | ✅ Supported | Parameters: (Object: Player|Array, Object: Player|Array, Number: Number, DamageReeval: DamageReeval). |
| `Start Damage Over Time` (`startDamageOverTime`) | ✅ Supported | Parameters: (Object: Player|Array, Player: Player, Number: Number, Number: Number). |
| `Start Facing` (`startFacing`) | ✅ Supported | Parameters: (Player: Player|Array, Direction: Vector, Turn Rate: Number, Relativity: Relativity, Reevaluation: FacingReeval). |
| `Start Forcing Dummy Bot Name` (`startForcingName`) | ✅ Supported | Parameters: (Object: Player|Array, String: String). |
| `Start Forcing Player Outlines` (`startForcingOutlineFor`) | ✅ Supported | Parameters: (ViewedPlayers: Player|Array, ViewingPlayers: Player|Array, Visible: Boolean, Color: Color, Visibility: OutlineVisibility). |
| `Start Forcing Player Position` (`startForcingPosition`) | ✅ Supported | Parameters: (Player: Player, Position: Vector, Boolean: Boolean). |
| `Start Forcing Player To Be Hero` (`startForcingHero`) | ✅ Supported | Parameters: (Object: Player|Array, Hero: Hero). |
| `Start Forcing Spawn Room` (`startForcingSpawn`) | ✅ Supported | Parameters: (Team: Team, Number: Number|Boolean). |
| `Start Forcing Throttle` (`startForcingThrottle`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number|Boolean, Number: Number|Boolean, Number: Number|Boolean, Number: Number|Boolean, Number: Number|Boolean, Number: Number|Boolean). |
| `Start Game Mode` (`startGameMode`) | ✅ Supported | No parameters. |
| `Start Heal Over Time` (`startHealingOverTime`) | ✅ Supported | Parameters: (Object: Player|Array, Player: Player, Number: Number, Number: Number). |
| `Start Healing Modification` (`startHealingModification`) | ✅ Supported | Parameters: (Object: Player|Array, Object: Player|Array, Number: Number, HealingReeval: HealingReeval). |
| `Start Holding Button` (`startForcingButton`) | ✅ Supported | Parameters: (Object: Player|Array, Button: Button). |
| `Start Modifying Hero Voice Lines` (`startModifyingVoicelinePitch`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Boolean: Boolean). |
| `Start Rule` (`startRule`) | ✅ Supported | Parameters: (Subroutine: Subroutine, IfAlreadyExecuting: StartRuleBehavior). |
| `Start Scaling Barriers` (`startScalingBarriers`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Boolean: Boolean). |
| `Start Scaling Player` (`startScalingSize`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Boolean: Boolean). |
| `Start Throttle In Direction` (`startThrottleInDirection`) | ✅ Supported | Parameters: (Player: Player|Array, Direction: Vector, Magnitude: Number, Relativity: Relativity, Throttle: Throttle, ThrottleReeval: ThrottleReeval). |
| `Start Transforming Throttle` (`startTransformingThrottle`) | ✅ Supported | Parameters: (Object: Player|Array, Number: Number, Number: Number, Direction: Vector). |
| `Stop Accelerating` (`stopAcceleration`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop All Assists` (`stopAllAssists`) | ✅ Supported | No parameters. |
| `Stop All Damage Modifications` (`stopAllDamageModifications`) | ✅ Supported | No parameters. |
| `Stop All Damage Over Time` (`stopAllDamageOverTime`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop All Heal Over Time` (`stopAllHealingOverTime`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop All Healing Modifications` (`stopAllHealingModifications`) | ✅ Supported | No parameters. |
| `Stop Assist` (`stopAssist`) | ✅ Supported | Parameters: (AssistId: AssistId). |
| `Stop Camera` (`stopCamera`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Stop Chasing Global Variable` (`stopChasingGlobalVariable`) | ✅ Supported | Parameters: (Variable: Variable). |
| `Stop Chasing Player Variable` (`stopChasingPlayerVariable`) | ✅ Supported | Parameters: (Player: Player|Array, PlayerVariable: Variable). |
| `Stop Chasing Variable` (`stopChasingVariable`) | ✅ Supported | Parameters: (Variable). |
| `Stop Damage Modification` (`stopDamageModification`) | ✅ Supported | Parameters: (DamageModificationId: DamageModificationId). |
| `Stop Damage Over Time` (`stopDamageOverTime`) | ✅ Supported | Parameters: (DotId: DotId). |
| `Stop Facing` (`stopFacing`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Forcing Dummy Bot Name` (`stopForcingName`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Forcing Hero` (`stopForcingHero`) | ✅ Supported | Parameters: (Player). |
| `Stop Forcing Player Outlines` (`stopForcingOutlineFor`) | ✅ Supported | Parameters: (Object: Player|Array, Object: Player|Array). |
| `Stop Forcing Player Position` (`stopForcingPosition`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Forcing Player To Be Hero` (`stopForcingCurrentHero`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Forcing Spawn Room` (`stopForcingSpawn`) | ✅ Supported | Parameters: (Team: Team). |
| `Stop Forcing Throttle` (`stopForcingThrottle`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Stop Heal Over Time` (`stopHealingOverTime`) | ✅ Supported | Parameters: (HotId: HotId). |
| `Stop Healing Modification` (`stopHealingModification`) | ✅ Supported | Parameters: (HealingModificationId: HealingModificationId). |
| `Stop Holding Button` (`stopForcingButton`) | ✅ Supported | Parameters: (Object: Player|Array, Button: Button). |
| `Stop Modifying Hero Voice Lines` (`stopModifyingVoicelinePitch`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Scaling Barriers` (`stopScalingBarriers`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Scaling Player` (`stopScalingSize`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Throttle In Direction` (`stopThrottleInDirection`) | ✅ Supported | Parameters: (Player: Player|Array). |
| `Stop Transforming Throttle` (`stopTransformingThrottle`) | ✅ Supported | Parameters: (Object: Player|Array). |
| `Teleport` (`teleport`) | ✅ Supported | Parameters: (Player: Player|Array, Position: Vector). |
| `Unpause Match Time` (`unpauseMatchTime`) | ✅ Supported | No parameters. |
| `Wait` (`wait`) | ✅ Supported | Parameters: (Duration: Any, WaitBehavior: Wait). |
| `Wait Until` (`waitUntil`) | ✅ Supported | Parameters: (Condition: Any, Timeout: Number). |

## 7. Values Inventory

All 255 canonical Workshop values and expressions are supported:

| Feature | Status | Notes |
| --- | --- | --- |
| `Ability Charge` (`getAbilityCharge`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Button: Button). |
| `Ability Cooldown` (`getAbilityCooldown`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Button: Button). |
| `Ability Icon String` (`abilityIconString`) | ✅ Supported | Returns: `String`; Parameters: (Hero: Hero, Button: Button). |
| `Ability Resource` (`getAbilityResource`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Button: Button). |
| `Absolute Value` (`absoluteValue`) | ✅ Supported | Returns: `Number`; Parameters: (Value: Number). |
| `Add` (`add`) | ✅ Supported | Returns: `Number|Vector`; Parameters: (a: Number|Boolean|Vector, b: Number|Boolean|Vector). |
| `All Damage Heroes` (`allDamageHeroes`) | ✅ Supported | Returns: `Array`. |
| `All Dead Players` (`getDeadPlayers`) | ✅ Supported | Returns: `Array`; Parameters: (Team: Team). |
| `All Heroes` (`allHeroes`) | ✅ Supported | Returns: `Array`. |
| `All Living Players` (`getLivingPlayers`) | ✅ Supported | Returns: `Array`; Parameters: (Team: Team). |
| `All Players` (`allPlayers`) | ✅ Supported | Returns: `Array`; Parameters: (Team: Team). |
| `All Players Not On Objective` (`getPlayersNotOnObjective`) | ✅ Supported | Returns: `Array`; Parameters: (Team: Team). |
| `All Players On Objective` (`getPlayersOnObjective`) | ✅ Supported | Returns: `Array`; Parameters: (Team: Team). |
| `All Support Heroes` (`allSupportHeroes`) | ✅ Supported | Returns: `Array`. |
| `All Tank Heroes` (`allTankHeroes`) | ✅ Supported | Returns: `Array`. |
| `Allowed Heroes` (`allowedHeroes`) | ✅ Supported | Returns: `Array`; Parameters: (Player: Player). |
| `Altitude Of` (`getAltitude`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Ammo` (`getAmmo`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Clip: Number). |
| `And` (`and`) | ✅ Supported | Returns: `Boolean`; Parameters: (A: Boolean|Number|Object|Array, B: Boolean|Number|Object|Array). |
| `Angle Between Vectors` (`angleBetweenVectors`) | ✅ Supported | Returns: `Number`; Parameters: (Direction: Vector, Direction: Vector). |
| `Angle Difference` (`angleDifference`) | ✅ Supported | Returns: `Number`; Parameters: (Value: Number, Value: Number). |
| `Append To Array` (`appendToArray`) | ✅ Supported | Returns: `Array`; Parameters: (Array: Object|Array, Value: Object|Array). |
| `Arccosine In Degrees` (`acosDeg`) | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Arccosine In Radians` (`acos`) | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Arcsine In Degrees` (`asinDeg`) | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Arcsine In Radians` (`asin`) | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Arctangent In Degrees` (`atan2Deg`) | ✅ Supported | Returns: `Number`; Parameters: (Number: Number, Number: Number). |
| `Arctangent In Radians` (`atan2`) | ✅ Supported | Returns: `Number`; Parameters: (Number: Number, Number: Number). |
| `Array` (`array`) | ✅ Supported | Returns: `Array`; Parameters: (Value: Object|Array). |
| `Array Contains` (`arrayContains`) | ✅ Supported | Returns: `Boolean`; Parameters: (Array: Array, Value: Object|Array). |
| `Array Slice` (`slice`) | ✅ Supported | Returns: `Array`; Parameters: (Array: Array, Start Index: Number|Boolean, Count: Number|Boolean). |
| `Assist Count` (`getNumberOfAssistIds`) | ✅ Supported | Returns: `Number`. |
| `Attacker` (`attacker`) | ✅ Supported | Returns: `Player`. |
| `Backward` (`backward`) | ✅ Supported | Returns: `Array`. |
| `Char In String` (`charAt`) | ✅ Supported | Returns: `String`; Parameters: (String: Any, Index: Any). |
| `Closest Player To` (`getClosestPlayer`) | ✅ Supported | Returns: `Player`; Parameters: (Position: Vector, Team: Team). |
| `Compare` (`compare`) | ✅ Supported | Returns: `Boolean`; Parameters: (a: Any, operator: __Operator__, b: Any). |
| `Control Mode Scoring Percentage` (`getControlScorePercentage`) | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Control Mode Scoring Team` (`getControlScoringTeam`) | ✅ Supported | Returns: `Team`. |
| `Cosine From Degrees` (`cosDeg`) | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Cosine From Radians` (`cos`) | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Count Of` (`countOf`) | ✅ Supported | Returns: `Number`; Parameters: (Array: Array). |
| `Cross Product` (`crossProduct`) | ✅ Supported | Returns: `Vector`; Parameters: (Vector: Vector, Vector: Vector). |
| `Current Array Element` (`currentArrayElement`) | ✅ Supported | Returns: `Any`. |
| `Current Array Index` (`currentArrayIndex`) | ✅ Supported | Returns: `Any`. |
| `Current Game Mode` (`getCurrentGamemode`) | ✅ Supported | Returns: `Gamemode`. |
| `Current Map` (`currentMap`) | ✅ Supported | Returns: `Map`. |
| `Custom Color` (`customColor`) | ✅ Supported | Returns: `Color`; Parameters: (Red: Number, Green: Number, Blue: Number, Alpha: Number). |
| `Custom String` (`customString`) | ✅ Supported | Returns: `String`; Parameters: (Format: String, Replacement 1: Object|Array, Replacement 2: Object|Array, Replacement 3: Object|Array). |
| `Damage Modification Count` (`getNumberOfDamageModificationIds`) | ✅ Supported | Returns: `Number`. |
| `Damage Over Time Count` (`getNumberOfDamageOverTimeIds`) | ✅ Supported | Returns: `Number`. |
| `Direction From Angles` (`directionFromAngles`) | ✅ Supported | Returns: `Vector`; Parameters: (HorizontalAngle: Number, VerticalAngle: Number). |
| `Direction Towards` (`directionTowards`) | ✅ Supported | Returns: `Vector`; Parameters: (Position: Vector|Player|Array, Position: Vector|Player|Array). |
| `Distance Between` (`distance`) | ✅ Supported | Returns: `Number`; Parameters: (Position: Vector|Player|Array, Position: Vector|Player|Array). |
| `Divide` (`divide`) | ✅ Supported | Returns: `Number|Vector`; Parameters: (a: Number|Boolean|Vector, b: Number|Boolean|Vector). |
| `Dot Product` (`dotProduct`) | ✅ Supported | Returns: `Number`; Parameters: (Value: Vector, Value: Vector). |
| `Down` (`down`) | ✅ Supported | Returns: `Array`. |
| `Empty Array` (`emptyArray`) | ✅ Supported | Returns: `Array`. |
| `Entity Count` (`getNumberOfEntityIds`) | ✅ Supported | Returns: `Number`. |
| `Entity Exists` (`entityExists`) | ✅ Supported | Returns: `Boolean`; Parameters: (Entity: Player|EntityId). |
| `Evaluate Once` (`evaluateOnce`) | ✅ Supported | Returns: `Object|Array`; Parameters: (Value: Object|Array). |
| `Event Ability` (`eventAbility`) | ✅ Supported | Returns: `Button`. |
| `Event Damage` (`eventDamage`) | ✅ Supported | Returns: `Number`. |
| `Event Direction` (`eventDirection`) | ✅ Supported | Returns: `Vector`. |
| `Event Healing` (`eventHealing`) | ✅ Supported | Returns: `Number`. |
| `Event Player` (`eventPlayer`) | ✅ Supported | Returns: `Player`. |
| `Event Was Critical Hit` (`eventWasCriticalHit`) | ✅ Supported | Returns: `Boolean`. |
| `Event Was Environment` (`eventWasEnvironment`) | ✅ Supported | Returns: `Boolean`. |
| `Event Was Health Pack` (`eventWasHealthPack`) | ✅ Supported | Returns: `Boolean`. |
| `Eye Position` (`getEyePosition`) | ✅ Supported | Returns: `Vector`; Parameters: (Player: Player). |
| `Facing Direction Of` (`getFacingDirection`) | ✅ Supported | Returns: `Vector`; Parameters: (Player: Player). |
| `Farthest Player From` (`getFarthestPlayer`) | ✅ Supported | Returns: `Player`; Parameters: (Position: Vector, Team: Team). |
| `Filtered Array` (`filteredArray`) | ✅ Supported | Returns: `Array`; Parameters: (Array: Array, Condition: Boolean). |
| `First Of` (`firstOf`) | ✅ Supported | Returns: `Object|Array`; Parameters: (Array: Array). |
| `Flag Position` (`getFlagPosition`) | ✅ Supported | Returns: `Vector`; Parameters: (Team: Team). |
| `Forward` (`forward`) | ✅ Supported | Returns: `Array`. |
| `Game Mode` (`gameMode`) | ✅ Supported | Returns: `Gamemode`; Parameters: (Gamemode: Gamemode). |
| `Has Spawned` (`hasSpawned`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player|Array). |
| `Has Status` (`hasStatus`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player, Status: Status). |
| `Heal Over Time Count` (`getNumberOfHealingOverTimeIds`) | ✅ Supported | Returns: `Number`. |
| `Healee` (`healee`) | ✅ Supported | Returns: `Player`. |
| `Healer` (`healer`) | ✅ Supported | Returns: `Player`. |
| `Healing Modification Count` (`getNumberOfHealingModificationIds`) | ✅ Supported | Returns: `Number`. |
| `Health` (`getHealth`) | ✅ Supported | Returns: `Number`; Parameters: (player: Player). |
| `Health Of Type` (`getHealthOfType`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Health: Health). |
| `Hero` (`hero`) | ✅ Supported | Returns: `Hero`; Parameters: (Hero: Hero). |
| `Hero Being Duplicated` (`getHeroOfDuplication`) | ✅ Supported | Returns: `Hero`; Parameters: (Player: Player). |
| `Hero Icon String` (`heroIconString`) | ✅ Supported | Returns: `String`; Parameters: (Hero: Hero). |
| `Hero Of` (`getHero`) | ✅ Supported | Returns: `Hero`; Parameters: (player: Player). |
| `Horizontal Angle From Direction` (`horizontalAngleFromDirection`) | ✅ Supported | Returns: `Number`; Parameters: (Direction: Vector). |
| `Horizontal Angle Towards` (`horizontalAngleTowards`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Position: Vector). |
| `Horizontal Facing Angle Of` (`getHorizontalFacingAngle`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Horizontal Speed Of` (`getHorizontalSpeed`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Host Player` (`hostPlayer`) | ✅ Supported | Returns: `Player`. |
| `Icon String` (`iconString`) | ✅ Supported | Returns: `String`; Parameters: (Icon: Icon). |
| `If-Then-Else` (`ifThenElse`) | ✅ Supported | Returns: `Object|Array`; Parameters: (Condition: Boolean|Number, True Value: Object|Array, False Value: Object|Array). |
| `Index Of Array Value` (`indexOfArrayValue`) | ✅ Supported | Returns: `Number`; Parameters: (Array: Array, Value: Object|Array). |
| `Index Of String Char` (`strIndex`) | ✅ Supported | Returns: `Number`; Parameters: (String: String, Character: String). |
| `Input Binding String` (`inputBindingString`) | ✅ Supported | Returns: `String`; Parameters: (Button: Button). |
| `Is Alive` (`isAlive`) | ✅ Supported | Returns: `Boolean`; Parameters: (player: Player). |
| `Is Assembling Heroes` (`isAssemblingHeroes`) | ✅ Supported | Returns: `Boolean`. |
| `Is Between Rounds` (`isMatchBetweenRounds`) | ✅ Supported | Returns: `Boolean`. |
| `Is Button Held` (`isButtonHeld`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player, Button: Button). |
| `Is CTF Mode In Sudden Death` (`isInSuddenDeath`) | ✅ Supported | Returns: `Boolean`. |
| `Is Communicating` (`isCommunicating`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player, Comms: Comms). |
| `Is Communicating Any` (`isCommunicatingAnything`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Communicating Any Emote` (`isCommunicatingEmote`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Communicating Any Spray` (`isCommunicatingSpray`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Communicating Any Voice line` (`isCommunicatingVoiceline`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Control Mode Point Locked` (`isControlPointLocked`) | ✅ Supported | Returns: `Boolean`. |
| `Is Crouching` (`isCrouching`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Dead` (`isDead`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Dummy Bot` (`isDummy`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Duplicating` (`isDuplicatingAHero`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Firing Primary` (`isFiringPrimaryFire`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Firing Secondary` (`isFiringSecondary`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Firing Secondary Fire` (`isFiringSecondaryFire`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Flag At Base` (`isFlagAtBase`) | ✅ Supported | Returns: `Boolean`; Parameters: (Team: Team). |
| `Is Flag Being Carried` (`isFlagBeingCarried`) | ✅ Supported | Returns: `Boolean`; Parameters: (Team: Team). |
| `Is Game In Progress` (`isGameInProgress`) | ✅ Supported | Returns: `Boolean`. |
| `Is Hero Being Played` (`isHeroBeingPlayed`) | ✅ Supported | Returns: `Boolean`; Parameters: (Hero: Hero, Team: Team). |
| `Is In Air` (`isInAir`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is In Alternate Form` (`isInAlternateForm`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is In Line of Sight` (`isInLoS`) | ✅ Supported | Returns: `Boolean`; Parameters: (Start Position: Vector|Player, End Position: Vector|Player, Barriers: BarrierLos). |
| `Is In Setup` (`isInSetup`) | ✅ Supported | Returns: `Boolean`. |
| `Is In Spawn Room` (`isInSpawnRoom`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is In View Angle` (`isInViewAngle`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player, Location: Vector, ViewAngle: Number). |
| `Is Jumping` (`isJumping`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Match Complete` (`isMatchComplete`) | ✅ Supported | Returns: `Boolean`. |
| `Is Meleeing` (`isMeleeing`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Moving` (`isMoving`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Objective Complete` (`isObjectiveComplete`) | ✅ Supported | Returns: `Boolean`; Parameters: (Number: Number). |
| `Is On Ground` (`isOnGround`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is On Objective` (`isOnObjective`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is On Wall` (`isOnWall`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Portrait On Fire` (`isOnFire`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Reloading` (`isReloading`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Standing` (`isStanding`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Team On Defense` (`isTeamOnDefense`) | ✅ Supported | Returns: `Boolean`; Parameters: (Team: Team). |
| `Is Team On Offense` (`isTeamOnOffense`) | ✅ Supported | Returns: `Boolean`; Parameters: (Team: Team). |
| `Is True For All` (`isTrueForAll`) | ✅ Supported | Returns: `Boolean`; Parameters: (Array: Array, Condition: Boolean). |
| `Is True For Any` (`isTrueForAny`) | ✅ Supported | Returns: `Boolean`; Parameters: (Array: Array, Condition: Boolean). |
| `Is Using Ability 1` (`isUsingAbility1`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Using Ability 2` (`isUsingAbility2`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player). |
| `Is Using Ultimate` (`isUsingUltimate`) | ✅ Supported | Returns: `Boolean`; Parameters: (Player: Player|Array). |
| `Is Waiting For Players` (`isWaitingForPlayers`) | ✅ Supported | Returns: `Boolean`. |
| `Last Assist ID` (`getLastAssistId`) | ✅ Supported | Returns: `AssistId`. |
| `Last Created Entity` (`lastCreatedEntity`) | ✅ Supported | Returns: `EntityId`. |
| `Last Created Health Pool` (`getLastCreatedHealthPool`) | ✅ Supported | Returns: `HealthPoolId`. |
| `Last Damage Modification ID` (`getLastDamageModification`) | ✅ Supported | Returns: `DamageModificationId`. |
| `Last Damage Over Time ID` (`getLastDamageOverTimeId`) | ✅ Supported | Returns: `DotId`. |
| `Last Heal Over Time ID` (`getLastHealingOverTimeId`) | ✅ Supported | Returns: `HotId`. |
| `Last Healing Modification ID` (`getLastHealingModification`) | ✅ Supported | Returns: `HealingModificationId`. |
| `Last Of` (`lastOf`) | ✅ Supported | Returns: `Any`; Parameters: (Array: Array). |
| `Last Text ID` (`lastTextId`) | ✅ Supported | Returns: `TextId`. |
| `Left` (`left`) | ✅ Supported | Returns: `Array`. |
| `Local Player` (`localPlayer`) | ✅ Supported | Returns: `Player`. |
| `Local Vector Of` (`localVector`) | ✅ Supported | Returns: `Vector`; Parameters: (Vector: Vector, Player: Player, Transform: Transform). |
| `Magnitude Of` (`magnitude`) | ✅ Supported | Returns: `Number`; Parameters: (Vector: Vector). |
| `Mapped Array` (`mappedArray`) | ✅ Supported | Returns: `Array`; Parameters: (Array: Array, Map: Object|Array). |
| `Match Round` (`getMatchRound`) | ✅ Supported | Returns: `Number`. |
| `Match Time` (`getMatchTime`) | ✅ Supported | Returns: `Number`. |
| `Max` (`max`) | ✅ Supported | Returns: `Number`; Parameters: (Value: Number|Boolean, Value: Number|Boolean). |
| `Max Ammo` (`getMaxAmmo`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Clip: Number). |
| `Max Health` (`getMaxHealth`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Max Health Of Type` (`getMaxHealthOfType`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Health: Health). |
| `Min` (`min`) | ✅ Supported | Returns: `Number`; Parameters: (Value: Number|Boolean, Value: Number|Boolean). |
| `Modulo` (`modulo`) | ✅ Supported | Returns: `Number`; Parameters: (Value: Number, Value: Number). |
| `Multiply` (`multiply`) | ✅ Supported | Returns: `Number|Vector`; Parameters: (a: Number|Boolean|Vector, b: Number|Boolean|Vector). |
| `Nearest Walkable Position` (`nearestWalkablePosition`) | ✅ Supported | Returns: `Vector`; Parameters: (Position: Vector). |
| `Normalize` (`normalize`) | ✅ Supported | Returns: `Vector`; Parameters: (Vector: Vector). |
| `Normalized Health` (`getNormalizedHealth`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Not` (`not`) | ✅ Supported | Returns: `Boolean`; Parameters: (Value: Boolean|Number). |
| `Null` (`null`) | ✅ Supported | Returns: `Player`. |
| `Number Of Dead Players` (`getNumberOfDeadPlayers`) | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Number Of Deaths` (`getNumberOfDeaths`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Number Of Eliminations` (`getNumberOfElims`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Number Of Final Blows` (`getNumberOfFinalBlows`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Number Of Heroes` (`getNumberOfHeroes`) | ✅ Supported | Returns: `Number`; Parameters: (Hero: Hero, Team: Team). |
| `Number Of Living Players` (`getNumberOfLivingPlayers`) | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Number Of Players` (`numberOfPlayers`) | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Number Of Players On Objective` (`getNumberOfPlayersOnObjective`) | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Number Of Slots` (`getNumberOfSlots`) | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Objective Index` (`getCurrentObjective`) | ✅ Supported | Returns: `Number`. |
| `Objective Position` (`getObjectivePosition`) | ✅ Supported | Returns: `Vector`; Parameters: (Objective: Any). |
| `Opposite Team Of` (`oppositeTeamOf`) | ✅ Supported | Returns: `Team`; Parameters: (Team: Team). |
| `Or` (`or`) | ✅ Supported | Returns: `Boolean`; Parameters: (A: Boolean|Number|Object|Array, B: Boolean|Number|Object|Array). |
| `Payload Position` (`getPayloadPosition`) | ✅ Supported | Returns: `Vector`. |
| `Payload Progress Percentage` (`getPayloadProgressPercentage`) | ✅ Supported | Returns: `Number`. |
| `Player Carrying Flag` (`getFlagCarrier`) | ✅ Supported | Returns: `Player`; Parameters: (Team: Team). |
| `Player Closest To Reticle` (`getPlayerClosestToReticle`) | ✅ Supported | Returns: `Player`; Parameters: (Player: Player, Team: Team). |
| `Player Hero Stat` (`getHeroStatistic`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Hero: Hero, Statistic: HeroStat). |
| `Player Stat` (`getStatistic`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Statistic: Stat). |
| `Players In Slot` (`getPlayersInSlot`) | ✅ Supported | Returns: `Player|Array`; Parameters: (Number: Number, Team: Team). |
| `Players On Hero` (`getPlayersOnHero`) | ✅ Supported | Returns: `Array`; Parameters: (Hero: Hero, Team: Team). |
| `Players Within Radius` (`getPlayersInRadius`) | ✅ Supported | Returns: `Array`; Parameters: (center: Vector, radius: Number, team: Team, losCheck: LosCheck). |
| `Players in View Angle` (`getPlayersInViewAngle`) | ✅ Supported | Returns: `Array`; Parameters: (Player: Player, Team: Team, ViewAngle: Number). |
| `Point Capture Percentage` (`getCapturePercentage`) | ✅ Supported | Returns: `Number`. |
| `Position Of` (`getPosition`) | ✅ Supported | Returns: `Vector`; Parameters: (player: Player|Array). |
| `Random Integer` (`randomInteger`) | ✅ Supported | Returns: `Number`; Parameters: (Min: Number|Boolean, Max: Number|Boolean). |
| `Random Real` (`randomReal`) | ✅ Supported | Returns: `Number`; Parameters: (Min: Number, Max: Number). |
| `Random Value In Array` (`randomValueInArray`) | ✅ Supported | Returns: `Any`; Parameters: (Array: Array). |
| `Ray Cast Hit Normal` (`raycastHitNormal`) | ✅ Supported | Returns: `Vector`; Parameters: (Position: Vector, Position: Vector, Player: Array, Player: Array, Boolean: Boolean). |
| `Ray Cast Hit Player` (`raycastHitPlayer`) | ✅ Supported | Returns: `Player`; Parameters: (Position: Vector, Position: Vector, Player: Array, Player: Array, Boolean: Boolean). |
| `Ray Cast Hit Position` (`raycastHitPosition`) | ✅ Supported | Returns: `Vector`; Parameters: (Start Position: Vector, End Position: Vector, Players To Include: Array, Players To Exclude: Array, Include Player Owned Objects: Boolean). |
| `Remove From Array` (`removeFromArray`) | ✅ Supported | Returns: `Array`; Parameters: (Array: Array, Value: Object|Array). |
| `Right` (`right`) | ✅ Supported | Returns: `Array`. |
| `Round To Integer` (`roundToInteger`) | ✅ Supported | Returns: `Number`; Parameters: (Value: Number, Rounding: Rounding). |
| `Score Of` (`getScore`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Server Load` (`getServerLoad`) | ✅ Supported | Returns: `Number`. |
| `Server Load Average` (`getAverageServerLoad`) | ✅ Supported | Returns: `Number`. |
| `Server Load Peak` (`getPeakServerLoad`) | ✅ Supported | Returns: `Number`. |
| `Sine From Degrees` (`sinDeg`) | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Sine From Radians` (`sin`) | ✅ Supported | Returns: `Number`; Parameters: (Value: Number). |
| `Slot Of` (`getSlot`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Sorted Array` (`sortedArray`) | ✅ Supported | Returns: `Array`; Parameters: (Array: Array, Sort: Object|Array). |
| `Spawn Points` (`getSpawnPoints`) | ✅ Supported | Returns: `Array`; Parameters: (Team: Team). |
| `Speed Of` (`getSpeed`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Speed Of In Direction` (`getSpeedInDirection`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Direction: Vector). |
| `Square Root` (`squareRoot`) | ✅ Supported | Returns: `Number`; Parameters: (value: Number). |
| `String Contains` (`strContains`) | ✅ Supported | Returns: `Boolean`; Parameters: (String: String, String: String). |
| `String Length` (`strLen`) | ✅ Supported | Returns: `Number`; Parameters: (String: String). |
| `String Replace` (`stringReplace`) | ✅ Supported | Returns: `String`; Parameters: (String: String|Array, Search: String|Array, Replacement: String|Array). |
| `String Slice` (`stringSlice`) | ✅ Supported | Returns: `String`; Parameters: (String: String, Start Index: Number, Count: Number). |
| `String Split` (`stringSplit`) | ✅ Supported | Returns: `Array`; Parameters: (String: String|Array, Separator: String|Array). |
| `Subtract` (`subtract`) | ✅ Supported | Returns: `Number|Vector`; Parameters: (a: Number|Boolean|Vector, b: Number|Boolean|Vector). |
| `Tangent From Degrees` (`tanDeg`) | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Tangent From Radians` (`tan`) | ✅ Supported | Returns: `Number`; Parameters: (Number: Number). |
| `Team Of` (`teamOf`) | ✅ Supported | Returns: `Team`; Parameters: (Player: Player). |
| `Team Score` (`teamScore`) | ✅ Supported | Returns: `Number`; Parameters: (Team: Team). |
| `Text Count` (`getNumberOfTextIds`) | ✅ Supported | Returns: `Number`. |
| `Throttle Of` (`getThrottle`) | ✅ Supported | Returns: `Vector`; Parameters: (player: Player). |
| `Total Time Elapsed` (`getTotalTimeElapsed`) | ✅ Supported | Returns: `Number`. |
| `Ultimate Charge Percent` (`getUltCharge`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Up` (`up`) | ✅ Supported | Returns: `Array`. |
| `Update Every Frame` (`updateEveryFrame`) | ✅ Supported | Returns: `Object|Array`; Parameters: (Value: Object|Array). |
| `Value In Array` (`valueInArray`) | ✅ Supported | Returns: `Any`; Parameters: (array: Any, index: Any). |
| `Vector` (`vector`) | ✅ Supported | Returns: `Vector`; Parameters: (X: Number|Boolean, Y: Number|Boolean, Z: Number|Boolean). |
| `Vector Towards` (`vectorTowards`) | ✅ Supported | Returns: `Vector`; Parameters: (Position: Any, Position: Any). |
| `Velocity Of` (`getVelocity`) | ✅ Supported | Returns: `Vector`; Parameters: (Player: Player). |
| `Vertical Angle From Direction` (`verticalAngleFromDirection`) | ✅ Supported | Returns: `Number`; Parameters: (Direction: Vector). |
| `Vertical Angle Towards` (`verticalAngleTowards`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player, Position: Vector). |
| `Vertical Facing Angle Of` (`getVerticalFacingAngle`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Vertical Speed Of` (`getVerticalSpeed`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Victim` (`victim`) | ✅ Supported | Returns: `Player`. |
| `Weapon` (`getCurrentWeapon`) | ✅ Supported | Returns: `Number`; Parameters: (Player: Player). |
| `Workshop Setting Combo` (`workshopSettingCombo`) | ✅ Supported | Returns: `Number`; Parameters: (Category: String, Name: String, Default: Number, Options: Array, SortOrder: Number). |
| `Workshop Setting Hero` (`createWorkshopSettingHero`) | ✅ Supported | Returns: `Hero`; Parameters: (CustomStringLiteral: String, CustomStringLiteral: String, HeroLiteral: Hero, IntLiteral: Number). |
| `Workshop Setting Integer` (`workshopSettingInteger`) | ✅ Supported | Returns: `Number`; Parameters: (Category: String, Name: String, Default: Number, MinValue: Number, MaxValue: Number, SortOrder: Number). |
| `Workshop Setting Real` (`createWorkshopSettingFloat`) | ✅ Supported | Returns: `Number`; Parameters: (CustomStringLiteral: String, CustomStringLiteral: String, FloatLiteral: Number, FloatLiteral: Number, FloatLiteral: Number, IntLiteral: Number). |
| `Workshop Setting Toggle` (`workshopSettingToggle`) | ✅ Supported | Returns: `Boolean`; Parameters: (Category: String, Name: String, Default: Boolean, SortOrder: Number). |
| `World Vector Of` (`worldVector`) | ✅ Supported | Returns: `Vector`; Parameters: (localVector: Vector, relativePlayer: Player, transformation: Transform). |
| `X Component Of` (`__xComponentOf__`) | ✅ Supported | Returns: `Number`; Parameters: (Vector: Vector). |
| `Y Component Of` (`__yComponentOf__`) | ✅ Supported | Returns: `Number`; Parameters: (Vector: Vector). |
| `Z Component Of` (`__zComponentOf__`) | ✅ Supported | Returns: `Number`; Parameters: (Vector: Vector). |

## 8. Enumerated Domains

All 52 canonical Workshop enumerated domains are supported:

| Feature | Status | Notes |
| --- | --- | --- |
| `AccelReeval` (`AccelReeval`) | ✅ Supported | 2 members, including `Direction Rate and Max Speed`, `None` (2 total members). |
| `AssistReeval` (`AssistReeval`) | ✅ Supported | 2 members, including `Assisters and Targets`, `None` (2 total members). |
| `BarrierLos` (`BarrierLos`) | ✅ Supported | 3 members, including `Barriers Do Not Block LOS`, `Enemy Barriers Block LOS`, `All Barriers Block LOS` (3 total members). |
| `Beam` (`Beam`) | ✅ Supported | 20 members, including `Grapple Beam`, `Good Beam`, `Bad Beam`, `Brigitte Flail Chain Beam`, … (20 total members). |
| `Button` (`Button`) | ✅ Supported | 10 members, including `Primary Fire`, `Secondary Fire`, `Ability 1`, `Ability 2`, … (10 total members). |
| `ChaseRateReeval` (`ChaseRateReeval`) | ✅ Supported | 2 members, including `None`, `Destination and Rate` (2 total members). |
| `ChaseTimeReeval` (`ChaseTimeReeval`) | ✅ Supported | 2 members, including `None`, `Destination and Duration` (2 total members). |
| `Clipping` (`Clipping`) | ✅ Supported | 2 members, including `Do Not Clip`, `Clip Against Surfaces` (2 total members). |
| `Color` (`Color`) | ✅ Supported | 17 members, including `Yellow`, `White`, `Red`, `Orange`, … (17 total members). |
| `Comms` (`Comms`) | ✅ Supported | 36 members, including `Acknowledge`, `Attacking`, `Countdown`, `Defending`, … (36 total members). |
| `DamageReeval` (`DamageReeval`) | ✅ Supported | 3 members, including `None`, `Receivers and Damagers`, `Receivers Damagers and Damage Percent` (3 total members). |
| `DynamicEffect` (`DynamicEffect`) | ✅ Supported | 100 members, including `Bad Explosion`, `Buff Impact Sound`, `Debuff Impact Sound`, `Buff Explosion Sound`, … (100 total members). |
| `Effect` (`Effect`) | ✅ Supported | 71 members, including `Orb`, `Bad Aura`, `Ring`, `Baptiste Immortality Field Protected Effect`, … (71 total members). |
| `EffectReeval` (`EffectReeval`) | ✅ Supported | 8 members, including `None`, `Visible To Position and Radius`, `Visible To`, `Color`, … (8 total members). |
| `EventPlayer` (`EventPlayer`) | ✅ Supported | 13 members, including `All`, `Slot 0`, `Slot 1`, `Slot 2`, … (13 total members). |
| `EventTeam` (`EventTeam`) | ✅ Supported | 3 members, including `All`, `Team 1`, `Team 2` (3 total members). |
| `FacingReeval` (`FacingReeval`) | ✅ Supported | 2 members, including `Direction and Turn Rate`, `None` (2 total members). |
| `Gamemode` (`Gamemode`) | ✅ Supported | 35 members, including `Assault`, `Bounty Hunter`, `Clash`, `Control`, … (35 total members). |
| `HealingReeval` (`HealingReeval`) | ✅ Supported | 3 members, including `None`, `Receivers and Healers`, `Receivers Healers and Healing Percent` (3 total members). |
| `Health` (`Health`) | ✅ Supported | 3 members, including `Armor`, `Shields`, `Health` (3 total members). |
| `Hero` (`Hero`) | ✅ Supported | 53 members, including `D.Va`, `Orisa`, `Reinhardt`, `Roadhog`, … (53 total members). |
| `HeroStat` (`HeroStat`) | ✅ Supported | 33 members, including `Healing Dealt`, `All Damage Dealt`, `Barrier Damage Dealt`, `Critical Hit Accuracy`, … (33 total members). |
| `HudPosition` (`HudPosition`) | ✅ Supported | 3 members, including `Left`, `Right`, `Top` (3 total members). |
| `HudReeval` (`HudReeval`) | ✅ Supported | 16 members, including `Visible To Sort Order String and Color`, `Visible To and String`, `Visible To`, `Visible To String and Color`, … (16 total members). |
| `Icon` (`Icon`) | ✅ Supported | 36 members, including `No`, `Question Mark`, `Skull`, `Checkmark`, … (36 total members). |
| `IconReeval` (`IconReeval`) | ✅ Supported | 8 members, including `Visible To and Position`, `Color`, `None`, `Position`, … (8 total members). |
| `Impulse` (`Impulse`) | ✅ Supported | 3 members, including `Cancel Contrary Motion`, `Cancel Contrary Motion XYZ`, `Incorporate Contrary Motion` (3 total members). |
| `Invis` (`Invis`) | ✅ Supported | 3 members, including `All`, `Enemies`, `None` (3 total members). |
| `InworldTextReeval` (`InworldTextReeval`) | ✅ Supported | 9 members, including `Visible To`, `Visible To and Color`, `Visible To and Position`, `Visible To and String`, … (9 total members). |
| `LosCheck` (`LosCheck`) | ✅ Supported | 4 members, including `Off`, `Surfaces`, `Surfaces And All Barriers`, `Surfaces And Enemy Barriers` (4 total members). |
| `Map` (`Map`) | ✅ Supported | 91 members, including `Ayutthaya`, `Black Forest`, `Castillo`, `Château Guillard`, … (91 total members). |
| `ModifyHealth` (`ModifyHealth`) | ✅ Supported | 2 members, including `Damage`, `Heal` (2 total members). |
| `Operation` (`Operation`) | ✅ Supported | 3 members, including `Append To Array`, `Remove From Array By Value`, `Remove From Array By Index` (3 total members). |
| `OutlineVisibility` (`OutlineVisibility`) | ✅ Supported | 3 members, including `Always`, `Default`, `Occluded` (3 total members). |
| `ProgressBarWorldReeval` (`ProgressBarWorldReeval`) | ✅ Supported | 1 members, including `Visible To And Values` (1 total members). |
| `ProgressHudReeval` (`ProgressHudReeval`) | ✅ Supported | 8 members, including `Color`, `None`, `Values`, `Values and Color`, … (8 total members). |
| `ProgressWorldTextReeval` (`ProgressWorldTextReeval`) | ✅ Supported | 16 members, including `Color`, `None`, `Position`, `Position and Color`, … (16 total members). |
| `Projectile` (`Projectile`) | ✅ Supported | 20 members, including `Orb Projectile`, `Baptiste Biotic Launcher`, `Bastion A-36 Tactical Grenade`, `Echo Sticky Bomb`, … (20 total members). |
| `ProjectileEffectReeval` (`ProjectileEffectReeval`) | ✅ Supported | 8 members, including `Visible To Position Direction and Size`, `Position Direction and Size`, `Visible To`, `None`, … (8 total members). |
| `Relativity` (`Relativity`) | ✅ Supported | 2 members, including `To World`, `To Player` (2 total members). |
| `Rounding` (`Rounding`) | ✅ Supported | 3 members, including `Up`, `Down`, `Nearest` (3 total members). |
| `SpecVisibility` (`SpecVisibility`) | ✅ Supported | 3 members, including `Default Visibility`, `Visible Always`, `Visible Never` (3 total members). |
| `StartRuleBehavior` (`StartRuleBehavior`) | ✅ Supported | 2 members, including `Restart Rule`, `Do Nothing` (2 total members). |
| `Stat` (`Stat`) | ✅ Supported | 20 members, including `Healing Dealt`, `All Damage Dealt`, `Barrier Damage Dealt`, `Damage Blocked`, … (20 total members). |
| `Status` (`Status`) | ✅ Supported | 10 members, including `Asleep`, `Burning`, `Frozen`, `Hacked`, … (10 total members). |
| `Team` (`Team`) | ✅ Supported | 3 members, including `All Teams`, `Team 1`, `Team 2` (3 total members). |
| `Throttle` (`Throttle`) | ✅ Supported | 2 members, including `Replace existing throttle`, `Add to existing throttle` (2 total members). |
| `ThrottleReeval` (`ThrottleReeval`) | ✅ Supported | 2 members, including `Direction and Magnitude`, `None` (2 total members). |
| `Transform` (`Transform`) | ✅ Supported | 2 members, including `Rotation`, `Rotation And Translation` (2 total members). |
| `Vector` (`Vector`) | ✅ Supported | 6 members, including `Up`, `Down`, `Left`, `Right`, … (6 total members). |
| `Wait` (`Wait`) | ✅ Supported | 3 members, including `Ignore Condition`, `Abort When False`, `Restart When True` (3 total members). |
| `WorldTextReeval` (`WorldTextReeval`) | ✅ Supported | 12 members, including `Color`, `None`, `String`, `String and Color`, … (12 total members). |

## 9. Custom-Game Settings

| Feature | Status | Notes |
| --- | --- | --- |
| `Main Settings` (`main`) | ✅ Supported | Custom game mode name and description strings. |
| `Lobby Settings` (`lobby`) | ✅ Supported | Team size, match start rules, spectator settings, map rotation, and lobby options. |
| `Mode Settings` (`modes`) | ✅ Supported | General mode parameters and individual game modes (Assault, Control, Escort, Hybrid, Push, Flashpoint, Clash, Deathmatch, Team Deathmatch, CTF, Elimination, etc.) and map pools (`enabled maps` / `disabled maps`). |
| `Hero Settings` (`heroes`) | ✅ Supported | Global hero rules, roster toggles (`enabled heroes` / `disabled heroes`), role limits, and per-hero ability/weapon/cooldown parameters. |
| `Workshop Extensions` (`extensions`) | ✅ Supported | Extension flags (`Beam Effects`, `Buff Status Effects`, `Debuff Status Effects`, `Buff and Debuff Sounds`, `Energy Explosion Effects`, `Kinetic Explosion Effects`, `Play More Effects`, `Spawn More Dummy Bots`). |
| `Custom Workshop Settings` (`workshop`) | ✅ Supported | User-defined custom settings defined via `Workshop Setting ...` values in rules. |

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
