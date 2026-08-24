# Actions Inventory

[← Back to Language Support Matrix](../language-support.md)

This document inventories all **219 canonical Workshop actions** supported by `workshop-rs`.

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
| `Force Player Hero` | ✅ Supported | Parameters: (Player: Player|Array, Hero: Hero). |
| `Force Throttle` | ✅ Supported | Parameters: (Player: Player|Array, MoveSpeed: Number|Boolean, InAirSpeed: Number|Boolean, SpectatorSpeed: Number|Boolean, GrappleBoost: Number|Boolean, JumpPower: Number|Boolean, MoveSpeed: Number|Boolean). |
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
| `Stop Chasing Variable` | ✅ Supported | Parameters: (Variable: Variable). |
| `Stop Damage Modification` | ✅ Supported | Parameters: (DamageModificationId: DamageModificationId). |
| `Stop Damage Over Time` | ✅ Supported | Parameters: (DotId: DotId). |
| `Stop Facing` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Forcing Dummy Bot Name` | ✅ Supported | Parameters: (Object: Player|Array). |
| `Stop Forcing Hero` | ✅ Supported | Parameters: (Player: Player|Array). |
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
