# Conditions & Control Flow

[← Back to Language Support Matrix](../language-support.md)

## Conditions & Branching

| Feature | Status | Notes |
| --- | --- | --- |
| Comparison condition | ✅ Supported | Binary condition expression (`Value OP Value`) with implicit logical AND across rule conditions. |
| `If` | ✅ Supported | Begins a conditional execution block: `If(Condition);`. |
| `Else If` | ✅ Supported | Secondary conditional execution block: `Else If(Condition);`. |
| `Else` | ✅ Supported | Fallback conditional execution block: `Else;`. |
| `End` | ✅ Supported | Terminates an `If`, `While`, or `For` block (including oracle-compatible implicit trailing `End`). |

## Loops & Iteration

| Feature | Status | Notes |
| --- | --- | --- |
| `While` | ✅ Supported | Loop block executed while condition evaluates to true: `While(Condition); ... End;`. |
| `For Global Variable` | ✅ Supported | Loop iterating a global variable from start to stop with step: `For Global Variable(var, start, stop, step); ... End;`. |
| `For Player Variable` | ✅ Supported | Loop iterating a player variable from start to stop with step: `For Player Variable(player, var, start, stop, step); ... End;`. |
| `Loop` | ✅ Supported | Restarts rule execution from the first action. |
| `Loop If` | ✅ Supported | Restarts rule execution if condition evaluates to true. |
| `Loop If Condition Is True` | ✅ Supported | Restarts rule execution if all rule conditions are currently true. |
| `Loop If Condition Is False` | ✅ Supported | Restarts rule execution if any rule condition is currently false. |
| `Break` | ✅ Supported | Breaks out of the innermost `While` or `For` loop. |
| `Continue` | ✅ Supported | Advances to the next iteration of the innermost `While` or `For` loop. |

## Jumps, Delays & Termination

| Feature | Status | Notes |
| --- | --- | --- |
| `Skip` | ✅ Supported | Unconditionally skips the specified number of subsequent actions. |
| `Skip If` | ✅ Supported | Skips subsequent actions if condition evaluates to true. |
| `Wait` | ✅ Supported | Pauses execution for a duration with condition restart/abort behavior. |
| `Wait Until` | ✅ Supported | Pauses execution until condition evaluates to true or timeout expires. |
| `Abort` | ✅ Supported | Immediately terminates rule action execution. |
| `Abort If` | ✅ Supported | Terminates rule action execution if condition evaluates to true. |
| `Abort If Condition Is True` | ✅ Supported | Terminates rule execution if all rule conditions are currently true. |
| `Abort If Condition Is False` | ✅ Supported | Terminates rule execution if any rule condition is currently false. |
| `Return` | ✅ Supported | Returns from a subroutine execution rule. |
