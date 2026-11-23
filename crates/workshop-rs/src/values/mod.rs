//! The Workshop value and expression domain.
//!
//! Values and their canonical expression forms are modeled in [`crate::Program`].
//! The shared parser and output boundaries preserve their locale-independent
//! identities.

mod arity;
pub(crate) mod emitter;
mod extent;
pub(crate) mod parser;
pub(crate) mod validate;

use crate::core::error::Result;
use crate::source::Span;
use crate::{WorkshopError, wir};

pub use crate::program::Value;

fn member_access_parts<'a>(
    program: &'a wir::Program,
    args: &[wir::ValueId],
    span: Option<Span>,
) -> Result<(wir::ValueId, &'a str, Option<wir::ValueId>)> {
    if !(2..=3).contains(&args.len()) {
        return Err(WorkshopError::malformed(
            "memberAccess expects two or three arguments",
            span,
        ));
    }
    let Some(wir::ValueNode {
        value: wir::Value::String(member),
        ..
    }) = program.values.get(args[1])
    else {
        return Err(WorkshopError::malformed(
            "memberAccess member must be a string",
            span,
        ));
    };
    Ok((args[0], member, args.get(2).copied()))
}
