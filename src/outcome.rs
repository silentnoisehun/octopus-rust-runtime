#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Completed,
    Failed,
}

impl ExecutionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub status: ExecutionStatus,
    pub output: String,
    pub code: Option<String>,
}

impl ExecutionOutcome {
    pub fn completed(output: impl Into<String>) -> Self {
        Self {
            status: ExecutionStatus::Completed,
            output: output.into(),
            code: None,
        }
    }

    pub fn failed(code: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            status: ExecutionStatus::Failed,
            output: output.into(),
            code: Some(code.into()),
        }
    }

    pub fn is_failed(&self) -> bool {
        self.status == ExecutionStatus::Failed
    }

    pub fn exit_code(&self) -> i32 {
        if self.is_failed() {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_outcome_has_zero_exit_code() {
        let outcome = ExecutionOutcome::completed("ok");
        assert_eq!(outcome.status, ExecutionStatus::Completed);
        assert_eq!(outcome.exit_code(), 0);
        assert_eq!(outcome.code, None);
    }

    #[test]
    fn failed_outcome_preserves_machine_code() {
        let outcome = ExecutionOutcome::failed("path_denied", "denied");
        assert!(outcome.is_failed());
        assert_eq!(outcome.exit_code(), 1);
        assert_eq!(outcome.code.as_deref(), Some("path_denied"));
    }
}
