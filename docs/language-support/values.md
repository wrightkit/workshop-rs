# Values Inventory

[← Back to Language Support Matrix](../language-support.md)

This document inventories all **255 canonical Workshop values and expressions** supported by `workshop-rs`.

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
| `Random Value In Array` | ✅ Supported | Returns: `Any`; Parameters: (Array: Array). |
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
