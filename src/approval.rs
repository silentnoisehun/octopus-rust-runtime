use crate::outcome::ExecutionOutcome;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static APPROVAL_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
    Expired,
    Replayed,
}

impl ApprovalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Denied => "denied",
            Self::Expired => "expired",
            Self::Replayed => "replayed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActionPlan {
    pub id: String,
    pub action_type: String,
    pub target: String,
    pub description: String,
    pub dry_run_output: String,
    pub idempotency_key: String,
    pub created_at: u128,
    pub expires_at: u128,
}

#[derive(Debug, Clone)]
pub struct ApprovalToken {
    pub action_id: String,
    pub token: String,
    pub status: ApprovalStatus,
    pub granted_at: Option<u128>,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: u128,
    pub action_id: String,
    pub status: ApprovalStatus,
    pub details: String,
}

struct ApprovalState {
    pending: HashMap<String, ActionPlan>,
    tokens: HashMap<String, ApprovalToken>,
    audit_log: Vec<AuditEntry>,
}

impl ApprovalState {
    fn new() -> Self {
        Self {
            pending: HashMap::new(),
            tokens: HashMap::new(),
            audit_log: Vec::new(),
        }
    }
}

static STATE: Mutex<Option<ApprovalState>> = Mutex::new(None);

fn state() -> std::sync::MutexGuard<'static, Option<ApprovalState>> {
    STATE.lock().unwrap()
}

pub fn create_action(
    action_type: &str,
    target: &str,
    description: &str,
    dry_run_output: &str,
    ttl_secs: u64,
) -> ActionPlan {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let id = format!(
        "action-{}-{}-{}",
        APPROVAL_SEQ.fetch_add(1, Ordering::Relaxed),
        now,
        short_hash(&format!("{action_type}:{target}:{description}"))
    );

    let idempotency_key = format!("idem-{}", short_hash(&format!("{id}:{now}")));

    let plan = ActionPlan {
        id: id.clone(),
        action_type: action_type.to_string(),
        target: target.to_string(),
        description: description.to_string(),
        dry_run_output: dry_run_output.to_string(),
        idempotency_key,
        created_at: now,
        expires_at: now + (ttl_secs as u128 * 1000),
    };

    let mut s = state();
    let s = s.get_or_insert_with(ApprovalState::new);
    s.pending.insert(id, plan.clone());

    s.audit_log.push(AuditEntry {
        timestamp: now,
        action_id: plan.id.clone(),
        status: ApprovalStatus::Pending,
        details: format!("Action created: {} on {}", action_type, target),
    });

    plan
}

pub fn approve_action(action_id: &str) -> Result<ApprovalToken, ExecutionOutcome> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let mut s = state();
    let s = s.get_or_insert_with(ApprovalState::new);

    let plan = s.pending.get(action_id).cloned().ok_or_else(|| {
        ExecutionOutcome::failed(
            "action_not_found",
            format!("[approval] action not found: {action_id}"),
        )
    })?;

    if now > plan.expires_at {
        s.pending.remove(action_id);
        s.audit_log.push(AuditEntry {
            timestamp: now,
            action_id: action_id.to_string(),
            status: ApprovalStatus::Expired,
            details: "Action expired before approval".to_string(),
        });
        return Err(ExecutionOutcome::failed(
            "approval_expired",
            format!("[approval] action has expired: {action_id}"),
        ));
    }

    let token = format!(
        "tok-{}-{}",
        short_hash(&format!("{action_id}:{now}")),
        APPROVAL_SEQ.fetch_add(1, Ordering::Relaxed)
    );

    let approval_token = ApprovalToken {
        action_id: action_id.to_string(),
        token: token.clone(),
        status: ApprovalStatus::Approved,
        granted_at: Some(now),
    };

    s.tokens.insert(token.clone(), approval_token.clone());
    s.pending.remove(action_id);

    s.audit_log.push(AuditEntry {
        timestamp: now,
        action_id: action_id.to_string(),
        status: ApprovalStatus::Approved,
        details: format!("Token granted: {token}"),
    });

    Ok(approval_token)
}

pub fn deny_action(action_id: &str, reason: &str) -> Result<(), ExecutionOutcome> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let mut s = state();
    let s = s.get_or_insert_with(ApprovalState::new);

    if !s.pending.contains_key(action_id) {
        return Err(ExecutionOutcome::failed(
            "action_not_found",
            format!("[approval] action not found: {action_id}"),
        ));
    }

    s.pending.remove(action_id);
    s.audit_log.push(AuditEntry {
        timestamp: now,
        action_id: action_id.to_string(),
        status: ApprovalStatus::Denied,
        details: format!("Denied: {reason}"),
    });

    Ok(())
}

pub fn consume_token(token: &str) -> Result<ActionPlan, ExecutionOutcome> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let mut s = state();
    let s = s.get_or_insert_with(ApprovalState::new);

    let approval = s.tokens.get(token).cloned().ok_or_else(|| {
        ExecutionOutcome::failed(
            "token_not_found",
            format!("[approval] token not found: {token}"),
        )
    })?;

    if approval.status == ApprovalStatus::Replayed {
        return Err(ExecutionOutcome::failed(
            "token_replayed",
            format!("[approval] token already consumed: {token}"),
        ));
    }

    if approval.status != ApprovalStatus::Approved {
        return Err(ExecutionOutcome::failed(
            "token_invalid",
            format!(
                "[approval] token is not approved: {}",
                approval.status.as_str()
            ),
        ));
    }

    // Mark as consumed (replayed prevents reuse)
    if let Some(entry) = s.tokens.get_mut(token) {
        entry.status = ApprovalStatus::Replayed;
    }

    s.audit_log.push(AuditEntry {
        timestamp: now,
        action_id: approval.action_id.clone(),
        status: ApprovalStatus::Replayed,
        details: format!("Token consumed: {token}"),
    });

    // Find the original plan from audit log or reconstruct
    let plan = ActionPlan {
        id: approval.action_id.clone(),
        action_type: "executed".to_string(),
        target: "unknown".to_string(),
        description: "action executed via token".to_string(),
        dry_run_output: String::new(),
        idempotency_key: String::new(),
        created_at: 0,
        expires_at: 0,
    };

    Ok(plan)
}

pub fn list_pending() -> Vec<ActionPlan> {
    let s = state();
    let s = s.as_ref().unwrap();
    s.pending.values().cloned().collect()
}

pub fn audit_log() -> Vec<AuditEntry> {
    let s = state();
    let s = s.as_ref().unwrap();
    s.audit_log.clone()
}

fn short_hash(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
        .chars()
        .take(12)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn create_action_returns_plan() {
        let plan = create_action(
            "file-write",
            "test.rs",
            "Write test file",
            "dry run output",
            300,
        );
        assert!(plan.id.starts_with("action-"));
        assert_eq!(plan.action_type, "file-write");
        assert_eq!(plan.target, "test.rs");
    }

    #[test]
    fn approve_action_returns_token() {
        let plan = create_action("file-write", "test.rs", "Write test file", "dry run", 300);
        let token = approve_action(&plan.id).unwrap();
        assert!(token.token.starts_with("tok-"));
        assert_eq!(token.status, ApprovalStatus::Approved);
    }

    #[test]
    fn deny_action_removes_pending() {
        let plan = create_action("file-write", "test.rs", "Write test file", "dry run", 300);
        deny_action(&plan.id, "not needed").unwrap();
        let pending = list_pending();
        assert!(!pending.iter().any(|p| p.id == plan.id));
    }

    #[test]
    fn consume_token_prevents_replay() {
        let plan = create_action("file-write", "test.rs", "Write test file", "dry run", 300);
        let token = approve_action(&plan.id).unwrap();
        consume_token(&token.token).unwrap();
        let result = consume_token(&token.token);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code.as_deref(), Some("token_replayed"));
    }

    #[test]
    fn expired_action_cannot_be_approved() {
        let plan = create_action("file-write", "test.rs", "Write test file", "dry run", 0);
        // Wait a bit to ensure expiration
        std::thread::sleep(Duration::from_millis(10));
        let result = approve_action(&plan.id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code.as_deref(),
            Some("approval_expired")
        );
    }

    #[test]
    fn audit_log_records_entries() {
        let plan = create_action("file-write", "test.rs", "Write test file", "dry run", 300);
        let _ = approve_action(&plan.id);
        let log = audit_log();
        assert!(log.iter().any(|e| e.action_id == plan.id));
    }

    #[test]
    fn nonexistent_action_returns_error() {
        let result = approve_action("nonexistent");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code.as_deref(),
            Some("action_not_found")
        );
    }
}
