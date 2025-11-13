use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::sync::{PlanJobsResult, SyncJob};

pub type TargetId = u64;
pub type SessionId = u64;

#[derive(Clone)]
pub struct RemoteTarget {
    pub id: TargetId,
    pub name: String,
    pub host: String,
    pub username: String,
    pub base_path: PathBuf,
    pub rules: Vec<SyncRule>,
    pub auth: AuthMethod,
}

impl RemoteTarget {
    pub fn summary(&self) -> String {
        format!(
            "{}@{}{}",
            self.username,
            self.host,
            self.base_path.display()
        )
    }
}

#[derive(Clone)]
pub enum AuthMethod {
    Password {
        secret: String,
        #[allow(dead_code)]
        stored: bool,
    },
    SshKey {
        private_key: PathBuf,
        passphrase: Option<String>,
        #[allow(dead_code)]
        passphrase_stored: bool,
    },
}

impl AuthMethod {
    pub fn password(secret: impl Into<String>) -> Self {
        Self::Password {
            secret: secret.into(),
            stored: false,
        }
    }

    #[allow(dead_code)]
    pub fn ssh_key(path: PathBuf) -> Self {
        Self::SshKey {
            private_key: path,
            passphrase: None,
            passphrase_stored: false,
        }
    }

    #[allow(dead_code)]
    pub fn is_password(&self) -> bool {
        matches!(self, AuthMethod::Password { .. })
    }

    #[allow(dead_code)]
    pub fn secret(&self) -> Option<&str> {
        match self {
            AuthMethod::Password { secret, .. } => Some(secret.as_str()),
            AuthMethod::SshKey { passphrase, .. } => passphrase.as_deref(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncRule {
    pub local: PathBuf,
    pub remote: PathBuf,
    pub direction: SyncDirection,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SyncDirection {
    Push,
    Pull,
    Bidirectional,
}

#[derive(Clone)]
pub struct SyncSession {
    pub id: SessionId,
    pub target_id: TargetId,
    pub status: SyncStatus,
    pub last_run: Option<SystemTime>,
    pub pending_actions: usize,
}

#[derive(Clone)]
pub enum SyncStatus {
    Idle,
    Planning,
    AwaitingConfirmation,
    Running { progress: f32 },
    Failed { reason: String },
    Completed,
}

#[derive(Clone)]
pub struct TransferLog {
    pub timestamp: SystemTime,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

#[derive(Clone)]
pub struct TaskProgress {
    pub kind: TaskKind,
    pub completed: usize,
    pub total: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskKind {
    Planning,
    Executing,
}

impl TaskProgress {
    pub fn new(kind: TaskKind, completed: usize, total: usize) -> Self {
        Self {
            kind,
            completed,
            total: total.max(1),
        }
    }

    pub fn percent(&self) -> f32 {
        let total = self.total.max(1) as f32;
        (self.completed as f32 / total).clamp(0.0, 1.0) * 100.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    SimplifiedChinese,
    TraditionalChinese,
}

#[derive(Clone)]
pub struct AppSettings {
    pub auto_connect: bool,
    pub watch_local_changes: bool,
    pub confirm_destructive: bool,
    pub limit_bandwidth: bool,
    pub bandwidth_mbps: u32,
    pub language: Language,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_connect: true,
            watch_local_changes: true,
            confirm_destructive: true,
            limit_bandwidth: false,
            bandwidth_mbps: 200,
            language: Language::English,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Dashboard,
    Settings,
    TargetSettings,
}

pub struct AppState {
    pub remote_targets: Vec<RemoteTarget>,
    pub sessions: Vec<SyncSession>,
    pub logs: Vec<TransferLog>,
    pub settings: AppSettings,
    pub active_target: Option<TargetId>,
    pub active_view: ActiveView,
    pub target_form: Option<TargetFormMode>,
    pub connection_tests: HashMap<TargetId, ConnectionTestState>,
    pub jobs: Vec<SyncJob>,
    next_session_id: SessionId,
    pub task_progress: HashMap<TargetId, TaskProgress>,
    pub bootstrap_pending: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TargetFormMode {
    Create,
    Edit(TargetId),
}

#[derive(Clone)]
pub enum ConnectionTestState {
    InProgress,
    Success(String),
    Failure(String),
}

impl AppState {
    pub fn new(settings: AppSettings, remote_targets: Vec<RemoteTarget>) -> Self {
        let remote_targets = if remote_targets.is_empty() {
            sample_remote_targets()
        } else {
            remote_targets
        };
        let logs = vec![
            TransferLog {
                timestamp: SystemTime::now() - Duration::from_secs(45),
                level: LogLevel::Info,
                message: "Staged 5 uploads for Production".into(),
            },
            TransferLog {
                timestamp: SystemTime::now() - Duration::from_secs(120),
                level: LogLevel::Info,
                message: "Detected drift on Analytics/datasets".into(),
            },
            TransferLog {
                timestamp: SystemTime::now() - Duration::from_secs(600),
                level: LogLevel::Info,
                message: "Completed sync session #998".into(),
            },
        ];

        let state = Self {
            active_target: remote_targets.first().map(|target| target.id),
            active_view: ActiveView::Dashboard,
            settings,
            remote_targets,
            sessions: Vec::new(),
            logs,
            target_form: None,
            connection_tests: HashMap::new(),
            jobs: Vec::new(),
            next_session_id: 1,
            task_progress: HashMap::new(),
            bootstrap_pending: true,
        };

        state
    }

    pub fn next_target_id(&self) -> TargetId {
        self.remote_targets
            .iter()
            .map(|target| target.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
    }

    pub fn next_session_id(&mut self) -> SessionId {
        let id = self.next_session_id;
        self.next_session_id = self.next_session_id.saturating_add(1);
        id
    }

    pub fn log_event(&mut self, level: LogLevel, message: impl Into<String>) {
        let timestamp = SystemTime::now();
        let message = message.into();
        let epoch_secs = timestamp
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        println!("[{epoch_secs}][{}] {message}", level.as_str());

        self.logs.push(TransferLog {
            timestamp,
            level,
            message,
        });
    }

    pub fn apply_planned_jobs(&mut self, target_id: TargetId, result: PlanJobsResult) {
        self.jobs.retain(|job| job.target_id != target_id);
        for warning in result.warnings {
            self.log_event(LogLevel::Warn, warning);
        }
        for planned in result.jobs {
            let id = self.next_session_id();
            self.jobs.push(planned.into_sync_job(id));
        }
        self.refresh_sessions();
    }

    pub fn set_task_progress(&mut self, target_id: TargetId, progress: TaskProgress) {
        self.task_progress.insert(target_id, progress);
    }

    pub fn clear_task_progress(&mut self, target_id: TargetId) {
        self.task_progress.remove(&target_id);
    }

    pub fn drop_jobs_for_target(&mut self, target_id: TargetId) {
        self.jobs.retain(|job| job.target_id != target_id);
        self.task_progress.remove(&target_id);
        self.refresh_sessions();
    }

    fn refresh_sessions(&mut self) {
        self.sessions = self.jobs.iter().map(SyncJob::to_session).collect();
    }

}

impl Default for AppState {
    fn default() -> Self {
        Self::new(AppSettings::default(), sample_remote_targets())
    }
}

pub fn sample_remote_targets() -> Vec<RemoteTarget> {
    vec![
        RemoteTarget {
            id: 1,
            name: "Production".into(),
            host: "prod.example.com:22".into(),
            username: "deploy".into(),
            base_path: PathBuf::from("/srv/www"),
            rules: vec![
                SyncRule {
                    local: PathBuf::from("./apps/web"),
                    remote: PathBuf::from("/web"),
                    direction: SyncDirection::Push,
                },
                SyncRule {
                    local: PathBuf::from("./secrets"),
                    remote: PathBuf::from("/config"),
                    direction: SyncDirection::Bidirectional,
                },
            ],
            auth: AuthMethod::password(String::new()),
        },
        RemoteTarget {
            id: 2,
            name: "Analytics".into(),
            host: "analytics.internal:2200".into(),
            username: "etl".into(),
            base_path: PathBuf::from("/data"),
            rules: vec![SyncRule {
                local: PathBuf::from("./datasets"),
                remote: PathBuf::from("/incoming"),
                direction: SyncDirection::Pull,
            }],
            auth: AuthMethod::password(String::new()),
        },
    ]
}
