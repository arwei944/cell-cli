use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum TransactionState { #[default]
Init, Preparing, Prepared, Committing, Committed, Aborting, Aborted }
impl fmt::Display for TransactionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Init => "Init", Self::Preparing => "Preparing",
            Self::Prepared => "Prepared", Self::Committing => "Committing",
            Self::Committed => "Committed", Self::Aborting => "Aborting",
            Self::Aborted => "Aborted",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum ParticipantState { #[default]
Init, Preparing, Prepared, Committing, Committed, Aborting, Aborted }
impl fmt::Display for ParticipantState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Init => "Init", Self::Preparing => "Preparing",
            Self::Prepared => "Prepared", Self::Committing => "Committing",
            Self::Committed => "Committed", Self::Aborting => "Aborting",
            Self::Aborted => "Aborted",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VoteResult { Yes, No }
impl fmt::Display for VoteResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self { Self::Yes => "Yes", Self::No => "No" })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareResult {
    pub participant_id: String, pub vote: VoteResult,
    pub message: String, pub prepared_at: String,
}
impl PrepareResult {
    pub fn yes(pid: impl Into<String>) -> Self {
        Self { participant_id: pid.into(), vote: VoteResult::Yes,
            message: "Prepared successfully".into(), prepared_at: chrono::Utc::now().to_rfc3339() }
    }
    pub fn no(pid: impl Into<String>, msg: impl Into<String>) -> Self {
        Self { participant_id: pid.into(), vote: VoteResult::No,
            message: msg.into(), prepared_at: chrono::Utc::now().to_rfc3339() }
    }
    pub fn is_yes(&self) -> bool { self.vote == VoteResult::Yes }
}

pub type PrepareFn = Arc<dyn Fn(&str) -> PrepareResult + Send + Sync>;
pub type CommitFn = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;
pub type AbortFn = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

#[derive(Clone)]
pub struct Participant {
    pub id: String, pub name: String, pub state: ParticipantState,
    pub prepare_fn: Option<PrepareFn>, pub commit_fn: Option<CommitFn>,
    pub abort_fn: Option<AbortFn>, pub last_error: Option<String>,
}
impl fmt::Debug for Participant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Participant").field("id", &self.id).field("name", &self.name)
            .field("state", &self.state).field("last_error", &self.last_error).finish()
    }
}
impl Participant {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self { id: id.into(), name: name.into(), state: ParticipantState::Init,
            prepare_fn: None, commit_fn: None, abort_fn: None, last_error: None }
    }
    pub fn with_prepare<F>(mut self, f: F) -> Self
    where F: Fn(&str) -> PrepareResult + Send + Sync + 'static {
        self.prepare_fn = Some(Arc::new(f)); self
    }
    pub fn with_commit<F>(mut self, f: F) -> Self
    where F: Fn(&str) -> Result<(), String> + Send + Sync + 'static {
        self.commit_fn = Some(Arc::new(f)); self
    }
    pub fn prepare(&mut self, tx_id: &str) -> PrepareResult {
        self.state = ParticipantState::Preparing;
        let r = if let Some(f) = &self.prepare_fn { f(tx_id) } else { PrepareResult::yes(&self.id) };
        if r.is_yes() { self.state = ParticipantState::Prepared; }
        else { self.state = ParticipantState::Aborted; self.last_error = Some(r.message.clone()); }
        r
    }
    pub fn commit(&mut self, tx_id: &str) -> Result<(), String> {
        self.state = ParticipantState::Committing;
        let r = self.commit_fn.as_ref().map_or(Ok(()), |f| f(tx_id));
        if r.is_ok() { self.state = ParticipantState::Committed; }
        else if let Err(e) = &r { self.last_error = Some(e.clone()); }
        r
    }
    pub fn abort(&mut self, tx_id: &str) -> Result<(), String> {
        self.state = ParticipantState::Aborting;
        let r = self.abort_fn.as_ref().map_or(Ok(()), |f| f(tx_id));
        if r.is_ok() { self.state = ParticipantState::Aborted; }
        else if let Err(e) = &r { self.last_error = Some(e.clone()); }
        r
    }
    pub fn is_prepared(&self) -> bool { self.state == ParticipantState::Prepared }
    pub fn is_committed(&self) -> bool { self.state == ParticipantState::Committed }
    pub fn is_aborted(&self) -> bool { self.state == ParticipantState::Aborted }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLogEntry {
    pub transaction_id: String, pub state: TransactionState,
    pub timestamp: String, pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoPhaseTransaction {
    pub id: String, pub name: String, pub state: TransactionState,
    pub participant_ids: Vec<String>, pub prepare_results: HashMap<String, PrepareResult>,
    pub logs: Vec<TransactionLogEntry>, pub created_at: String,
    pub updated_at: String, pub timeout_ms: u64, pub error_message: Option<String>,
}
impl TwoPhaseTransaction {
    pub fn new(id: impl Into<String>, name: impl Into<String>, pids: Vec<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let tx_id = id.into();
        Self { id: tx_id.clone(), name: name.into(), state: TransactionState::Init,
            participant_ids: pids, prepare_results: HashMap::new(),
            logs: vec![TransactionLogEntry {
                transaction_id: tx_id, state: TransactionState::Init,
                timestamp: now.clone(), details: "Transaction initialized".into() }],
            created_at: now.clone(), updated_at: now, timeout_ms: 30000, error_message: None }
    }
    fn add_log(&mut self, state: TransactionState, details: impl Into<String>) {
        let now = chrono::Utc::now().to_rfc3339();
        self.logs.push(TransactionLogEntry {
            transaction_id: self.id.clone(), state: state.clone(),
            timestamp: now.clone(), details: details.into() });
        self.state = state; self.updated_at = now;
    }
    pub fn mark_preparing(&mut self) { self.add_log(TransactionState::Preparing, "Starting prepare phase"); }
    pub fn mark_prepared(&mut self) { self.add_log(TransactionState::Prepared, "All participants prepared"); }
    pub fn mark_committing(&mut self) { self.add_log(TransactionState::Committing, "Starting commit phase"); }
    pub fn mark_committed(&mut self) { self.add_log(TransactionState::Committed, "Transaction committed successfully"); }
    pub fn mark_aborting(&mut self, r: impl Into<String>) { self.add_log(TransactionState::Aborting, format!("Aborting: {}", r.into())); }
    pub fn mark_aborted(&mut self, r: impl Into<String>) {
        let r = r.into(); self.error_message = Some(r.clone());
        self.add_log(TransactionState::Aborted, format!("Transaction aborted: {r}"));
    }
    pub fn add_prepare_result(&mut self, result: PrepareResult) {
        let pid = result.participant_id.clone();
        self.add_log(TransactionState::Preparing, format!("Participant '{}' voted {}", pid, result.vote));
        self.prepare_results.insert(pid, result);
    }
    pub fn all_prepared(&self) -> bool {
        !self.participant_ids.is_empty() && self.participant_ids.iter()
            .all(|id| self.prepare_results.get(id).is_some_and(PrepareResult::is_yes))
    }
    pub fn any_voted_no(&self) -> bool { self.prepare_results.values().any(|r| !r.is_yes()) }
    pub fn is_committed(&self) -> bool { self.state == TransactionState::Committed }
    pub fn is_aborted(&self) -> bool { self.state == TransactionState::Aborted }
}

#[derive(Debug, Clone, Default)]
pub struct TwoPhaseCoordinator {
    participants: HashMap<String, Participant>,
    transactions: HashMap<String, TwoPhaseTransaction>,
}
impl TwoPhaseCoordinator {
    pub fn new() -> Self { Self { participants: HashMap::new(), transactions: HashMap::new() } }
    pub fn register_participant(&mut self, p: Participant) { self.participants.insert(p.id.clone(), p); }
    pub fn get_participant(&self, id: &str) -> Option<&Participant> { self.participants.get(id) }
    pub fn participant_count(&self) -> usize { self.participants.len() }
    fn validate_pids(&self, pids: &[String]) -> Result<(), TwoPhaseCommitError> {
        for id in pids {
            if !self.participants.contains_key(id) {
                return Err(TwoPhaseCommitError::ParticipantNotFound(id.clone()));
            }
        }
        Ok(())
    }
    pub fn begin_transaction(&mut self, name: impl Into<String>, pids: Vec<String>) -> Result<&TwoPhaseTransaction, TwoPhaseCommitError> {
        self.validate_pids(&pids)?;
        self.create_tx(Uuid::new_v4().to_string(), name, pids)
    }
    pub fn begin_transaction_with_id(&mut self, id: impl Into<String>, name: impl Into<String>, pids: Vec<String>) -> Result<&TwoPhaseTransaction, TwoPhaseCommitError> {
        let tx_id = id.into();
        if self.transactions.contains_key(&tx_id) {
            return Err(TwoPhaseCommitError::TransactionAlreadyExists(tx_id));
        }
        self.validate_pids(&pids)?;
        self.create_tx(tx_id, name, pids)
    }
    fn create_tx(&mut self, tx_id: String, name: impl Into<String>, pids: Vec<String>) -> Result<&TwoPhaseTransaction, TwoPhaseCommitError> {
        self.transactions.insert(tx_id.clone(), TwoPhaseTransaction::new(tx_id.clone(), name, pids));
        Ok(self.transactions.get(&tx_id).unwrap())
    }
    fn check_state(&self, tx_id: &str, expected: TransactionState, msg: &str) -> Result<(), TwoPhaseCommitError> {
        let tx = self.transactions.get(tx_id)
            .ok_or_else(|| TwoPhaseCommitError::TransactionNotFound(tx_id.to_string()))?;
        if tx.state != expected {
            return Err(TwoPhaseCommitError::InvalidState(tx.state.clone(), msg.to_string()));
        }
        Ok(())
    }
    pub fn prepare(&mut self, tx_id: &str) -> Result<&TwoPhaseTransaction, TwoPhaseCommitError> {
        self.check_state(tx_id, TransactionState::Init, "prepare requires Init state")?;
        self.transactions.get_mut(tx_id).unwrap().mark_preparing();
        let pids = self.transactions.get(tx_id).unwrap().participant_ids.clone();
        for pid in &pids {
            let result = self.participants.get_mut(pid)
                .ok_or_else(|| TwoPhaseCommitError::ParticipantNotFound(pid.clone()))?
                .prepare(tx_id);
            self.transactions.get_mut(tx_id).unwrap().add_prepare_result(result);
        }
        let tx = self.transactions.get_mut(tx_id).unwrap();
        if tx.any_voted_no() {
            let voter = tx.prepare_results.values().find(|r| !r.is_yes())
                .map(|r| r.participant_id.clone()).unwrap_or_default();
            tx.mark_aborting(format!("Participant '{voter}' voted No"));
            self.do_abort(tx_id)?;
            return Ok(self.transactions.get(tx_id).unwrap());
        }
        if tx.all_prepared() { tx.mark_prepared(); }
        Ok(self.transactions.get(tx_id).unwrap())
    }
    pub fn commit(&mut self, tx_id: &str) -> Result<&TwoPhaseTransaction, TwoPhaseCommitError> {
        self.check_state(tx_id, TransactionState::Prepared, "commit requires Prepared state")?;
        self.transactions.get_mut(tx_id).unwrap().mark_committing();
        let pids = self.transactions.get(tx_id).unwrap().participant_ids.clone();
        let mut failed: Vec<(String, String)> = Vec::new();
        for pid in &pids {
            let p = self.participants.get_mut(pid)
                .ok_or_else(|| TwoPhaseCommitError::ParticipantNotFound(pid.clone()))?;
            if let Err(e) = p.commit(tx_id) { failed.push((pid.clone(), e)); }
        }
        let tx = self.transactions.get_mut(tx_id).unwrap();
        if failed.is_empty() { tx.mark_committed(); }
        else {
            let errs: Vec<String> = failed.iter().map(|(p, e)| format!("{p}: {e}")).collect();
            return Err(TwoPhaseCommitError::CommitFailed(errs.join("; ")));
        }
        Ok(self.transactions.get(tx_id).unwrap())
    }
    pub fn abort(&mut self, tx_id: &str, reason: impl Into<String>) -> Result<&TwoPhaseTransaction, TwoPhaseCommitError> {
        let tx = self.transactions.get_mut(tx_id)
            .ok_or_else(|| TwoPhaseCommitError::TransactionNotFound(tx_id.to_string()))?;
        if tx.state == TransactionState::Committed || tx.state == TransactionState::Aborted {
            return Err(TwoPhaseCommitError::InvalidState(
                tx.state.clone(), "cannot abort a completed transaction".into()));
        }
        let reason = reason.into();
        tx.error_message = Some(reason.clone());
        tx.mark_aborting(reason);
        self.do_abort(tx_id)?;
        Ok(self.transactions.get(tx_id).unwrap())
    }
    fn do_abort(&mut self, tx_id: &str) -> Result<(), TwoPhaseCommitError> {
        let pids = self.transactions.get(tx_id).unwrap().participant_ids.clone();
        let mut errors: Vec<(String, String)> = Vec::new();
        for pid in &pids {
            if let Some(p) = self.participants.get_mut(pid)
                && p.state != ParticipantState::Aborted && p.state != ParticipantState::Init
                    && let Err(e) = p.abort(tx_id) { errors.push((pid.clone(), e)); }
        }
        let tx = self.transactions.get_mut(tx_id).unwrap();
        if errors.is_empty() {
            let reason = tx.error_message.clone().unwrap_or_else(|| "Unknown reason".into());
            tx.mark_aborted(reason);
        } else {
            let errs: Vec<String> = errors.iter().map(|(p, e)| format!("{p}: {e}")).collect();
            tx.mark_aborted(format!("Abort with errors: {}", errs.join("; ")));
        }
        Ok(())
    }
    pub fn handle_timeout(&mut self, tx_id: &str) -> Result<&TwoPhaseTransaction, TwoPhaseCommitError> {
        let state = self.transactions.get(tx_id)
            .ok_or_else(|| TwoPhaseCommitError::TransactionNotFound(tx_id.to_string()))?
            .state.clone();
        match state {
            TransactionState::Preparing | TransactionState::Init =>
                self.abort(tx_id, "Transaction timeout during prepare phase"),
            TransactionState::Prepared => self.commit(tx_id),
            _ => Err(TwoPhaseCommitError::InvalidState(
                state, "timeout handling not applicable in this state".into())),
        }
    }
    pub fn get_status(&self, tx_id: &str) -> Option<TransactionState> {
        self.transactions.get(tx_id).map(|t| t.state.clone())
    }
    pub fn get_transaction(&self, id: &str) -> Option<&TwoPhaseTransaction> { self.transactions.get(id) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TwoPhaseCommitError {
    ParticipantNotFound(String), TransactionNotFound(String),
    TransactionAlreadyExists(String), InvalidState(TransactionState, String),
    CommitFailed(String), Timeout(String),
}
impl fmt::Display for TwoPhaseCommitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParticipantNotFound(id) => write!(f, "Participant not found: {id}"),
            Self::TransactionNotFound(id) => write!(f, "Transaction not found: {id}"),
            Self::TransactionAlreadyExists(id) => write!(f, "Transaction already exists: {id}"),
            Self::InvalidState(s, m) => write!(f, "Invalid state '{s}': {m}"),
            Self::CommitFailed(m) => write!(f, "Commit failed: {m}"),
            Self::Timeout(m) => write!(f, "Timeout: {m}"),
        }
    }
}
impl std::error::Error for TwoPhaseCommitError {}

#[cfg(test)]
mod tests {
    use super::*;
    fn mk_yes(id: &str) -> Participant { Participant::new(id, format!("p-{id}")) }
    fn mk_no(id: &str, r: &str) -> Participant {
        let rs = r.to_string(); let i = id.to_string();
        Participant::new(id, format!("p-{id}")).with_prepare(move |_| PrepareResult::no(&i, &rs))
    }
    fn mk_fail_commit(id: &str) -> Participant {
        let i = id.to_string();
        Participant::new(id, format!("p-{id}")).with_commit(move |_| Err(format!("{i}: commit failed")))
    }

    #[test] fn t01_state_result_participant() {
        assert_eq!(TransactionState::Init.to_string(), "Init");
        assert_eq!(TransactionState::Committed.to_string(), "Committed");
        assert_eq!(VoteResult::Yes.to_string(), "Yes");
        assert_eq!(VoteResult::No.to_string(), "No");
        assert_eq!(ParticipantState::Init.to_string(), "Init");
        assert_eq!(ParticipantState::Committed.to_string(), "Committed");
        assert_eq!(ParticipantState::Aborted.to_string(), "Aborted");
        let r = PrepareResult::yes("p1"); assert!(r.is_yes()); assert_eq!(r.participant_id, "p1");
        let r2 = PrepareResult::no("p1", "not ready"); assert!(!r2.is_yes()); assert_eq!(r2.message, "not ready");
        let mut p = mk_yes("p1"); assert!(p.prepare("tx-1").is_yes()); assert!(p.is_prepared());
        assert!(p.commit("tx-1").is_ok()); assert!(p.is_committed());
        let mut p2 = mk_no("p2", "rejected"); let r2 = p2.prepare("tx-1");
        assert!(!r2.is_yes()); assert!(p2.is_aborted());
        assert_eq!(p2.last_error.as_deref(), Some("rejected"));
    }
    #[test] fn t02_register_status_normal_commit() {
        let mut c = TwoPhaseCoordinator::new(); assert_eq!(c.participant_count(), 0);
        c.register_participant(mk_yes("p1")); c.register_participant(mk_yes("p2"));
        assert_eq!(c.participant_count(), 2);
        assert!(c.get_participant("p1").is_some());
        assert!(c.get_participant("p3").is_none());
        assert_eq!(c.get_status("nonexistent"), None);
        let tx = c.begin_transaction("test", vec!["p1".into(), "p2".into()]).unwrap();
        let id = tx.id.clone(); assert_eq!(tx.state, TransactionState::Init);
        assert_eq!(c.get_status(&id), Some(TransactionState::Init));
        c.prepare(&id).unwrap(); let tx = c.get_transaction(&id).unwrap();
        assert_eq!(tx.state, TransactionState::Prepared); assert!(tx.all_prepared());
        assert_eq!(c.get_status(&id), Some(TransactionState::Prepared));
        c.commit(&id).unwrap();
        assert!(c.get_transaction(&id).unwrap().is_committed());
        assert_eq!(c.get_status(&id), Some(TransactionState::Committed));
    }
    #[test] fn t03_no_vote_abort() {
        let mut c = TwoPhaseCoordinator::new();
        c.register_participant(mk_yes("p1")); c.register_participant(mk_no("p2", "rejected"));
        let tx = c.begin_transaction("test", vec!["p1".into(), "p2".into()]).unwrap();
        let id = tx.id.clone(); let result = c.prepare(&id).unwrap();
        assert!(result.is_aborted()); assert!(result.any_voted_no());
        assert!(result.error_message.is_some());
        assert!(c.get_participant("p1").unwrap().is_aborted());
        assert!(c.get_participant("p2").unwrap().is_aborted());
    }
    #[test] fn t04_abort_transaction() {
        let mut c = TwoPhaseCoordinator::new();
        c.register_participant(mk_yes("p1")); c.register_participant(mk_yes("p2"));
        let tx = c.begin_transaction("test", vec!["p1".into(), "p2".into()]).unwrap();
        let id = tx.id.clone(); c.prepare(&id).unwrap();
        let result = c.abort(&id, "user cancelled").unwrap();
        assert!(result.is_aborted());
        assert!(c.get_participant("p1").unwrap().is_aborted());
        assert!(c.get_participant("p2").unwrap().is_aborted());
    }
    #[test] fn t05_partial_failure_commit() {
        let mut c = TwoPhaseCoordinator::new();
        c.register_participant(mk_yes("p1")); c.register_participant(mk_fail_commit("p2"));
        let tx = c.begin_transaction("test", vec!["p1".into(), "p2".into()]).unwrap();
        let id = tx.id.clone(); c.prepare(&id).unwrap();
        let result = c.commit(&id); assert!(result.is_err());
        match result.unwrap_err() {
            TwoPhaseCommitError::CommitFailed(_) => {},
            _ => panic!("Expected CommitFailed"),
        }
    }
    #[test] fn t06_transaction_log() {
        let mut c = TwoPhaseCoordinator::new(); c.register_participant(mk_yes("p1"));
        let tx = c.begin_transaction("test", vec!["p1".into()]).unwrap();
        let id = tx.id.clone(); c.prepare(&id).unwrap(); c.commit(&id).unwrap();
        let tx = c.get_transaction(&id).unwrap(); assert!(!tx.logs.is_empty());
        let states: Vec<TransactionState> = tx.logs.iter().map(|l| l.state.clone()).collect();
        assert!(states.contains(&TransactionState::Init));
        assert!(states.contains(&TransactionState::Preparing));
        assert!(states.contains(&TransactionState::Prepared));
        assert!(states.contains(&TransactionState::Committing));
        assert!(states.contains(&TransactionState::Committed));
        for log in &tx.logs { assert_eq!(log.transaction_id, id); assert!(!log.timestamp.is_empty()); }
    }
    #[test] fn t07_multiple_participants() {
        let mut c = TwoPhaseCoordinator::new();
        let pids: Vec<String> = (1..=5).map(|i| format!("p{i}")).collect();
        for id in &pids { c.register_participant(mk_yes(id)); }
        assert_eq!(c.participant_count(), 5);
        let tx = c.begin_transaction("multi", pids.clone()).unwrap();
        let id = tx.id.clone(); c.prepare(&id).unwrap();
        let tx = c.get_transaction(&id).unwrap();
        assert_eq!(tx.participant_ids.len(), 5); assert!(tx.all_prepared());
        c.commit(&id).unwrap();
        assert!(c.get_transaction(&id).unwrap().is_committed());
        for pid in &pids { assert!(c.get_participant(pid).unwrap().is_committed()); }
    }
    #[test] fn t08_timeout_init_aborts() {
        let mut c = TwoPhaseCoordinator::new(); c.register_participant(mk_yes("p1"));
        let tx = c.begin_transaction("timeout", vec!["p1".into()]).unwrap();
        let id = tx.id.clone(); let result = c.handle_timeout(&id).unwrap();
        assert!(result.is_aborted());
        assert!(result.error_message.as_ref().unwrap().contains("timeout"));
    }
    #[test] fn t09_prepared_timeout_commits() {
        let mut c = TwoPhaseCoordinator::new(); c.register_participant(mk_yes("p1"));
        let tx = c.begin_transaction("test", vec!["p1".into()]).unwrap();
        let id = tx.id.clone(); c.prepare(&id).unwrap();
        assert_eq!(c.get_status(&id), Some(TransactionState::Prepared));
        let result = c.handle_timeout(&id).unwrap(); assert!(result.is_committed());
    }
    #[test] fn t10_error_cases() {
        let mut c = TwoPhaseCoordinator::new();
        let r = c.begin_transaction("test", vec!["unknown".into()]); assert!(r.is_err());
        match r.unwrap_err() {
            TwoPhaseCommitError::ParticipantNotFound(id) => assert_eq!(id, "unknown"),
            _ => panic!("Expected ParticipantNotFound"),
        }
        c.register_participant(mk_yes("p1"));
        let tx = c.begin_transaction("test", vec!["p1".into()]).unwrap();
        let id = tx.id.clone();
        let r = c.commit(&id); assert!(r.is_err());
        match r.unwrap_err() {
            TwoPhaseCommitError::InvalidState(state, _) => assert_eq!(state, TransactionState::Init),
            _ => panic!("Expected InvalidState"),
        }
        c.begin_transaction_with_id("custom", "n", vec!["p1".into()]).unwrap();
        let dup = c.begin_transaction_with_id("custom", "n2", vec!["p1".into()]);
        assert!(dup.is_err());
        match dup.unwrap_err() {
            TwoPhaseCommitError::TransactionAlreadyExists(id) => assert_eq!(id, "custom"),
            _ => panic!("Expected TransactionAlreadyExists"),
        }
        assert!(c.get_transaction("nonexistent").is_none());
        let e = TwoPhaseCommitError::ParticipantNotFound("p1".into());
        assert!(e.to_string().contains("p1"));
        let e = TwoPhaseCommitError::CommitFailed("oops".into());
        assert!(e.to_string().contains("oops"));
    }
    #[test] fn t11_default_and_display() {
        assert_eq!(TransactionState::default(), TransactionState::Init);
        assert_eq!(ParticipantState::default(), ParticipantState::Init);
        let c = TwoPhaseCoordinator::default(); assert_eq!(c.participant_count(), 0);
        let e = TwoPhaseCommitError::Timeout("t".into()); assert!(e.to_string().contains("Timeout"));
        let e = TwoPhaseCommitError::TransactionNotFound("tx1".into()); assert!(e.to_string().contains("tx1"));
    }
    #[test] fn t12_begin_with_id() {
        let mut c = TwoPhaseCoordinator::new(); c.register_participant(mk_yes("p1"));
        let tx = c.begin_transaction_with_id("tx-123", "test", vec!["p1".into()]).unwrap();
        assert_eq!(tx.id, "tx-123"); assert_eq!(tx.name, "test");
        assert_eq!(tx.participant_ids, vec!["p1".to_string()]);
    }
}
