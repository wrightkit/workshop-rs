# Program Structure & Variables

[← Back to Language Support Matrix](../language-support.md)

## Program Structure & Rules

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

## Variables & Subroutines

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
