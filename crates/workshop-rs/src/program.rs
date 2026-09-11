//! Canonical Workshop program concepts.

use crate::settings::Settings;

/// A complete Workshop program built from Workshop concepts.
#[derive(Debug, Clone, Default)]
pub struct Program {
    pub settings: Option<Settings>,
    pub global_variables: Vec<Variable>,
    pub player_variables: Vec<Variable>,
    pub subroutines: Vec<Subroutine>,
    pub rules: Vec<Rule>,
}

impl Program {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn global_variable(&mut self, variable: Variable) -> &mut Self {
        self.global_variables.push(variable);
        self
    }

    pub fn player_variable(&mut self, variable: Variable) -> &mut Self {
        self.player_variables.push(variable);
        self
    }

    pub fn subroutine(&mut self, subroutine: Subroutine) -> &mut Self {
        self.subroutines.push(subroutine);
        self
    }

    pub fn rule(&mut self, rule: Rule) -> &mut Self {
        self.rules.push(rule);
        self
    }
}

/// A Workshop global or player variable declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    /// The raw Workshop declaration index, when the declaration has one.
    pub index: Option<u32>,
}

impl Variable {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            index: None,
        }
    }

    pub fn with_index(name: impl Into<String>, index: u32) -> Self {
        Self {
            name: name.into(),
            index: Some(index),
        }
    }
}

/// A Workshop subroutine declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subroutine {
    pub name: String,
    /// The raw Workshop declaration index, when the declaration has one.
    pub index: Option<u32>,
}

impl Subroutine {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            index: None,
        }
    }

    pub fn with_index(name: impl Into<String>, index: u32) -> Self {
        Self {
            name: name.into(),
            index: Some(index),
        }
    }
}

/// A Workshop rule with explicit conditions and a linear action stream.
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub disabled: bool,
    pub event: Event,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}

impl Rule {
    pub fn new(name: impl Into<String>, event: Event) -> Self {
        Self {
            name: name.into(),
            disabled: false,
            event,
            conditions: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn condition(mut self, condition: impl Into<Condition>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }
}

/// A rule condition. Conditions remain distinct from general value expressions.
#[derive(Debug, Clone)]
pub struct Condition {
    pub value: Value,
    pub disabled: bool,
}

impl Condition {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            disabled: false,
        }
    }

    pub fn disabled(value: Value) -> Self {
        Self {
            value,
            disabled: true,
        }
    }
}

impl From<Value> for Condition {
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

/// A Workshop event identity and its native filters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Global,
    EachPlayer,
    EachPlayerWithFilters {
        team: EventTeam,
        target: EventTarget,
    },
    Player {
        kind: PlayerEventKind,
        team: EventTeam,
        target: EventTarget,
    },
    Subroutine(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTeam {
    All,
    Team1,
    Team2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventTarget {
    All,
    Slot(u8),
    Hero(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerEventKind {
    DealtDamage,
    DealtFinalBlow,
    DealtHealing,
    DealtKnockback,
    Died,
    EarnedElimination,
    Joined,
    Left,
    ReceivedHealing,
    ReceivedKnockback,
    TookDamage,
}

/// A Workshop action line. Control flow is represented in the same order as
/// the Workshop source, including its explicit `End` lines.
#[derive(Debug, Clone)]
pub enum Action {
    SetGlobalVariable {
        variable: String,
        value: Value,
    },
    ModifyGlobalVariable {
        variable: String,
        op: ModifyOp,
        value: Value,
    },
    SetPlayerVariable {
        player: Value,
        variable: String,
        value: Value,
    },
    ModifyPlayerVariable {
        player: Value,
        variable: String,
        op: ModifyOp,
        value: Value,
    },
    AssignMember {
        target: Value,
        op: Option<ModifyOp>,
        value: Value,
    },
    CallSubroutine {
        subroutine: String,
    },
    If {
        condition: Value,
    },
    ElseIf {
        condition: Value,
    },
    Else,
    While {
        condition: Value,
    },
    ForGlobalVariable {
        variable: String,
        start: Value,
        stop: Value,
        step: Value,
    },
    ForPlayerVariable {
        player: Value,
        variable: String,
        start: Value,
        stop: Value,
        step: Value,
    },
    End,
    Disabled {
        action: Box<Action>,
    },
    Call {
        name: String,
        args: Vec<Value>,
    },
}

impl Action {
    /// Mark an action as disabled.
    pub fn disabled(action: Action) -> Self {
        Self::Disabled {
            action: Box::new(action),
        }
    }

    /// Construct a dynamic action call by canonical Workshop id.
    pub fn call(name: impl Into<String>, args: impl IntoIterator<Item = Value>) -> Self {
        Self::Call {
            name: name.into(),
            args: args.into_iter().collect(),
        }
    }
}

/// The operation used by a Workshop variable modification action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifyOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Min,
    Max,
    RaiseToPower,
    AppendToArray,
    RemoveFromArray,
    RemoveFromArrayByIndex,
}

/// A composable Workshop value expression.
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    LocalizedString(String),
    Bool(bool),
    Null,
    Array(Vec<Value>),
    Vector {
        x: Box<Value>,
        y: Box<Value>,
        z: Box<Value>,
    },
    Enum {
        value_type: String,
        value: String,
    },
    GlobalVariable(String),
    PlayerVariable {
        player: Box<Value>,
        variable: String,
    },
    Subroutine(String),
    EventPlayer,
    Call {
        name: String,
        args: Vec<Value>,
    },
}

impl Value {
    /// Construct a numeric Workshop literal.
    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }

    /// Construct a custom Workshop string literal.
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn global_variable(name: impl Into<String>) -> Self {
        Self::GlobalVariable(name.into())
    }

    pub fn player_variable(player: Value, name: impl Into<String>) -> Self {
        Self::PlayerVariable {
            player: Box::new(player),
            variable: name.into(),
        }
    }

    /// Construct a dynamic value call by canonical Workshop id.
    pub fn call(name: impl Into<String>, args: impl IntoIterator<Item = Value>) -> Self {
        Self::Call {
            name: name.into(),
            args: args.into_iter().collect(),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Number(f64::from(value))
    }
}

macro_rules! impl_integer_value {
    ($($type:ty),+ $(,)?) => {
        $(
            impl From<$type> for Value {
                fn from(value: $type) -> Self {
                    Self::Number(value as f64)
                }
            }
        )+
    };
}

impl_integer_value!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(values: Vec<T>) -> Self {
        Self::Array(values.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Value>, const N: usize> From<[T; N]> for Value {
    fn from(values: [T; N]) -> Self {
        Self::Array(values.into_iter().map(Into::into).collect())
    }
}

include!(concat!(env!("OUT_DIR"), "/typed_api.rs"));
