mod from_wir;
mod to_wir;

use crate::core::error::WorkshopError;
use crate::wir;

pub(super) fn malformed_id(kind: &str, index: usize) -> WorkshopError {
    WorkshopError::Malformed {
        message: format!("dangling {kind} {index}"),
        span: None,
    }
}

fn wir_value_children(value: &wir::Value) -> Vec<wir::ValueId> {
    match value {
        wir::Value::Array(values) => values.clone(),
        wir::Value::Vector { x, y, z } => vec![*x, *y, *z],
        wir::Value::PlayerVariable { player, .. } => vec![*player],
        wir::Value::Call { args, .. } => args.clone(),
        _ => Vec::new(),
    }
}
