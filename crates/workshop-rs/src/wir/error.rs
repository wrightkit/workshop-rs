use crate::core::source::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IrError {
    DanglingReference {
        what: &'static str,
        id: u32,
    },
    Invalid {
        code: &'static str,
        message: String,
        span: Option<Span>,
    },
}

impl IrError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            IrError::DanglingReference { .. } => "dangling-reference",
            IrError::Invalid { code, .. } => code,
        }
    }

    pub(crate) fn message(&self) -> String {
        match self {
            IrError::DanglingReference { what, id } => {
                format!("dangling {what} reference: id {id}")
            }
            IrError::Invalid { message, .. } => message.clone(),
        }
    }

    pub(crate) fn span(&self) -> Option<Span> {
        match self {
            IrError::Invalid { span, .. } => *span,
            IrError::DanglingReference { .. } => None,
        }
    }
}

impl std::fmt::Display for IrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

impl std::error::Error for IrError {}
