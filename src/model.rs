use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

pub type TargetId = u64;
pub type SessionId = u64;

#[derive(Clone, Serialize, Deserialize)]
pub struct RemoteTarget {
    pub id: TargetId,
    pub name: String,
    pub host: String,
    pub username: String,
    pub base_path: PathBuf,
    pub rules: Vec<SyncRule>,
    #[serde(default)]
    pub password: String,
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
    pub message: String,
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
        let sessions = vec![
            SyncSession {
                id: 1001,
                target_id: 1,
                status: SyncStatus::Running { progress: 0.42 },
                last_run: Some(SystemTime::now() - Duration::from_secs(120)),
                pending_actions: 12,
            },
            SyncSession {
                id: 1002,
                target_id: 2,
                status: SyncStatus::AwaitingConfirmation,
                last_run: Some(SystemTime::now() - Duration::from_secs(900)),
                pending_actions: 3,
            },
        ];

        let logs = vec![
            TransferLog {
                timestamp: SystemTime::now() - Duration::from_secs(45),
                message: "Staged 5 uploads for Production".into(),
            },
            TransferLog {
                timestamp: SystemTime::now() - Duration::from_secs(120),
                message: "Detected drift on Analytics/datasets".into(),
            },
            TransferLog {
                timestamp: SystemTime::now() - Duration::from_secs(600),
                message: "Completed sync session #998".into(),
            },
        ];

        Self {
            active_target: remote_targets.first().map(|target| target.id),
            active_view: ActiveView::Dashboard,
            settings,
            remote_targets,
            sessions,
            logs,
            target_form: None,
            connection_tests: HashMap::new(),
        }
    }

    pub fn next_target_id(&self) -> TargetId {
        self.remote_targets
            .iter()
            .map(|target| target.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
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
            password: String::new(),
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
            password: String::new(),
        },
    ]
}
