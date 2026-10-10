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

/// A non-ongoing player event identity.
///
/// Workshop can add player-scoped event identities independently of this
/// crate. Consumers should use a wildcard arm when matching this type.
#[non_exhaustive]
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

impl PlayerEventKind {
    /// The inverse of [`catalog_id`](Self::catalog_id).
    pub(crate) fn from_catalog_id(id: &str) -> Option<Self> {
        Some(match id {
            "playerDealtDamage" => Self::DealtDamage,
            "playerDealtFinalBlow" => Self::DealtFinalBlow,
            "playerDealtHealing" => Self::DealtHealing,
            "playerDealtKnockback" => Self::DealtKnockback,
            "playerDied" => Self::Died,
            "playerEarnedElimination" => Self::EarnedElimination,
            "playerJoined" => Self::Joined,
            "playerLeft" => Self::Left,
            "playerReceivedHealing" => Self::ReceivedHealing,
            "playerReceivedKnockback" => Self::ReceivedKnockback,
            "playerTookDamage" => Self::TookDamage,
            _ => return None,
        })
    }

    /// The canonical catalog event id (`playerDealtDamage`) the Workshop
    /// event section resolves this kind from. Public since
    /// `rule-content-v1` names events by this id.
    pub fn catalog_id(self) -> &'static str {
        match self {
            Self::DealtDamage => "playerDealtDamage",
            Self::DealtFinalBlow => "playerDealtFinalBlow",
            Self::DealtHealing => "playerDealtHealing",
            Self::DealtKnockback => "playerDealtKnockback",
            Self::Died => "playerDied",
            Self::EarnedElimination => "playerEarnedElimination",
            Self::Joined => "playerJoined",
            Self::Left => "playerLeft",
            Self::ReceivedHealing => "playerReceivedHealing",
            Self::ReceivedKnockback => "playerReceivedKnockback",
            Self::TookDamage => "playerTookDamage",
        }
    }
}

/// The operation used by a Workshop variable modification action.
///
/// Workshop can add modification operations independently of this crate.
/// Consumers should use a wildcard arm when matching this type.
#[non_exhaustive]
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
    RemoveFromArrayByValue,
    RemoveFromArrayByIndex,
}

impl ModifyOp {
    /// The inverse of [`catalog_id`](Self::catalog_id).
    pub(crate) fn from_catalog_id(id: &str) -> Option<Self> {
        Some(match id {
            "add" => Self::Add,
            "subtract" => Self::Subtract,
            "multiply" => Self::Multiply,
            "divide" => Self::Divide,
            "modulo" => Self::Modulo,
            "min" => Self::Min,
            "max" => Self::Max,
            "raiseToPower" => Self::RaiseToPower,
            "appendToArray" => Self::AppendToArray,
            "removeFromArrayByValue" => Self::RemoveFromArrayByValue,
            "removeFromArrayByIndex" => Self::RemoveFromArrayByIndex,
            _ => return None,
        })
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Add => "Add",
            Self::Subtract => "Subtract",
            Self::Multiply => "Multiply",
            Self::Divide => "Divide",
            Self::Modulo => "Modulo",
            Self::Min => "Min",
            Self::Max => "Max",
            Self::RaiseToPower => "RaiseToPower",
            Self::AppendToArray => "AppendToArray",
            Self::RemoveFromArrayByValue => "RemoveFromArrayByValue",
            Self::RemoveFromArrayByIndex => "RemoveFromArrayByIndex",
        }
    }

    /// The canonical catalog operator id (`add`, `removeFromArrayByIndex`)
    /// the Workshop modify-operator position resolves this operation from.
    /// Public since `rule-content-v1` names modify operands by this id.
    pub fn catalog_id(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Subtract => "subtract",
            Self::Multiply => "multiply",
            Self::Divide => "divide",
            Self::Modulo => "modulo",
            Self::Min => "min",
            Self::Max => "max",
            Self::RaiseToPower => "raiseToPower",
            Self::AppendToArray => "appendToArray",
            Self::RemoveFromArrayByValue => "removeFromArrayByValue",
            Self::RemoveFromArrayByIndex => "removeFromArrayByIndex",
        }
    }
}
