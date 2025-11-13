use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime},
};

use anyhow::{anyhow, Context, Result};
use ssh2::{OpenFlags, OpenType, Sftp};

use crate::{
    connection,
    model::{RemoteTarget, SessionId, SyncDirection, SyncRule, SyncSession, SyncStatus, TargetId},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryKind {
    File,
    #[allow(dead_code)]
    Directory,
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub kind: EntryKind,
    pub size: u64,
    pub modified: SystemTime,
}

pub trait LocalStore {
    fn list(&self, root: &Path) -> Result<Vec<FileEntry>>;
    fn read_file(&self, root: &Path, rel_path: &Path) -> Result<Vec<u8>>;
    fn write_file(&self, root: &Path, rel_path: &Path, bytes: &[u8]) -> Result<()>;
    fn remove_file(&self, root: &Path, rel_path: &Path) -> Result<()>;
    fn ensure_dir(&self, root: &Path, rel_path: &Path) -> Result<()>;
}

pub trait RemoteStore {
    fn list(&self, root: &Path) -> Result<Vec<FileEntry>>;
    fn read_file(&self, root: &Path, rel_path: &Path) -> Result<Vec<u8>>;
    fn write_file(&self, root: &Path, rel_path: &Path, bytes: &[u8]) -> Result<()>;
    fn remove_file(&self, root: &Path, rel_path: &Path) -> Result<()>;
    fn ensure_dir(&self, root: &Path, rel_path: &Path) -> Result<()>;
}

#[derive(Clone, Debug)]
pub enum SyncAction {
    Upload {
        rel_path: PathBuf,
        #[allow(dead_code)]
        size: u64,
    },
    Download {
        rel_path: PathBuf,
        #[allow(dead_code)]
        size: u64,
    },
    DeleteRemote { rel_path: PathBuf },
    DeleteLocal { rel_path: PathBuf },
    Conflict {
        #[allow(dead_code)]
        rel_path: PathBuf,
    },
}

#[derive(Clone, Debug, Default)]
pub struct PlanStats {
    pub uploads: usize,
    pub downloads: usize,
    pub deletes_remote: usize,
    pub deletes_local: usize,
    pub conflicts: usize,
}

#[derive(Clone, Debug)]
pub struct SyncPlan {
    pub rule: SyncRule,
    pub actions: Vec<SyncAction>,
    #[allow(dead_code)]
    pub stats: PlanStats,
}

pub type FileIndex = HashMap<PathBuf, FileEntry>;

#[derive(Clone)]
pub struct SyncJob {
    pub id: SessionId,
    pub target_id: TargetId,
    #[allow(dead_code)]
    pub rule: SyncRule,
    #[allow(dead_code)]
    pub local_index: FileIndex,
    #[allow(dead_code)]
    pub remote_index: FileIndex,
    pub plan: SyncPlan,
    pub created_at: SystemTime,
}

impl SyncJob {
    #[allow(dead_code)]
    pub fn plan<L: LocalStore, R: RemoteStore>(
        id: SessionId,
        target_id: TargetId,
        rule: &SyncRule,
        local: &L,
        remote: &R,
    ) -> Result<Self> {
        let local_index = index_entries(local.list(&rule.local)?);
        let remote_index = index_entries(remote.list(&rule.remote)?);
        let (actions, stats) = diff_actions(rule, &local_index, &remote_index);

        Ok(Self {
            id,
            target_id,
            rule: rule.clone(),
            created_at: SystemTime::now(),
            plan: SyncPlan {
                rule: rule.clone(),
                actions,
                stats,
            },
            local_index,
            remote_index,
        })
    }

    pub fn pending_actions(&self) -> usize {
        self.plan.actions.len()
    }

    pub fn to_session(&self) -> SyncSession {
        let status = if self.plan.actions.is_empty() {
            SyncStatus::Idle
        } else {
            SyncStatus::AwaitingConfirmation
        };

        SyncSession {
            id: self.id,
            target_id: self.target_id,
            status,
            last_run: Some(self.created_at),
            pending_actions: self.pending_actions(),
        }
    }
}

#[derive(Clone)]
pub struct PlannedJob {
    pub target_id: TargetId,
    pub rule: SyncRule,
    pub local_index: FileIndex,
    pub remote_index: FileIndex,
    pub actions: Vec<SyncAction>,
    pub stats: PlanStats,
    pub created_at: SystemTime,
}

impl PlannedJob {
    pub fn into_sync_job(self, id: SessionId) -> SyncJob {
        let PlannedJob {
            target_id,
            rule,
            local_index,
            remote_index,
            actions,
            stats,
            created_at,
        } = self;
        let plan_rule = rule.clone();

        SyncJob {
            id,
            target_id,
            rule,
            local_index,
            remote_index,
            plan: SyncPlan {
                rule: plan_rule,
                actions,
                stats,
            },
            created_at,
        }
    }
}

pub struct PlanJobsResult {
    pub jobs: Vec<PlannedJob>,
    pub warnings: Vec<String>,
}

#[derive(Default)]
pub struct ExecutionSummary {
    pub applied: usize,
    pub skipped: usize,
    pub failures: Vec<(SyncAction, String)>,
}

pub struct SyncPlanner<'a, L: LocalStore, R: RemoteStore> {
    local: &'a L,
    remote: &'a R,
}

impl<'a, L: LocalStore, R: RemoteStore> SyncPlanner<'a, L, R> {
    pub fn new(local: &'a L, remote: &'a R) -> Self {
        Self { local, remote }
    }

    pub fn plan(&self, rule: &SyncRule) -> Result<SyncPlan> {
        let local_index = index_entries(self.local.list(&rule.local)?);
        let remote_index = index_entries(self.remote.list(&rule.remote)?);
        let (actions, stats) = diff_actions(rule, &local_index, &remote_index);

        Ok(SyncPlan {
            rule: rule.clone(),
            actions,
            stats,
        })
    }
}

fn diff_actions(
    rule: &SyncRule,
    local_index: &FileIndex,
    remote_index: &FileIndex,
) -> (Vec<SyncAction>, PlanStats) {
    let mut actions = Vec::new();
    let mut stats = PlanStats::default();

    for (path, local_entry) in local_index {
        match remote_index.get(path) {
            None => match rule.direction {
                SyncDirection::Push => {
                    actions.push(SyncAction::Upload {
                        rel_path: path.clone(),
                        size: local_entry.size,
                    });
                    stats.uploads += 1;
                }
                SyncDirection::Pull => {
                    actions.push(SyncAction::DeleteLocal {
                        rel_path: path.clone(),
                    });
                    stats.deletes_local += 1;
                }
                SyncDirection::Bidirectional => {
                    actions.push(SyncAction::Upload {
                        rel_path: path.clone(),
                        size: local_entry.size,
                    });
                    stats.uploads += 1;
                }
            },
            Some(remote_entry) => match rule.direction {
                SyncDirection::Push => {
                    if newer(local_entry.modified, remote_entry.modified) {
                        actions.push(SyncAction::Upload {
                            rel_path: path.clone(),
                            size: local_entry.size,
                        });
                        stats.uploads += 1;
                    }
                }
                SyncDirection::Pull => {
                    if newer(remote_entry.modified, local_entry.modified) {
                        actions.push(SyncAction::Download {
                            rel_path: path.clone(),
                            size: remote_entry.size,
                        });
                        stats.downloads += 1;
                    }
                }
                SyncDirection::Bidirectional => {
                    let local_newer = newer(local_entry.modified, remote_entry.modified);
                    let remote_newer = newer(remote_entry.modified, local_entry.modified);
                    match (local_newer, remote_newer) {
                        (true, false) => {
                            actions.push(SyncAction::Upload {
                                rel_path: path.clone(),
                                size: local_entry.size,
                            });
                            stats.uploads += 1;
                        }
                        (false, true) => {
                            actions.push(SyncAction::Download {
                                rel_path: path.clone(),
                                size: remote_entry.size,
                            });
                            stats.downloads += 1;
                        }
                        (true, true) => {
                            actions.push(SyncAction::Conflict {
                                rel_path: path.clone(),
                            });
                            stats.conflicts += 1;
                        }
                        _ => {}
                    }
                }
            },
        }
    }

    for (path, remote_entry) in remote_index {
        if local_index.contains_key(path) {
            continue;
        }

        match rule.direction {
            SyncDirection::Push => {
                actions.push(SyncAction::DeleteRemote {
                    rel_path: path.clone(),
                });
                stats.deletes_remote += 1;
            }
            SyncDirection::Pull => {
                actions.push(SyncAction::Download {
                    rel_path: path.clone(),
                    size: remote_entry.size,
                });
                stats.downloads += 1;
            }
            SyncDirection::Bidirectional => {
                actions.push(SyncAction::Download {
                    rel_path: path.clone(),
                    size: remote_entry.size,
                });
                stats.downloads += 1;
            }
        }
    }

    (actions, stats)
}

fn index_entries(entries: Vec<FileEntry>) -> FileIndex {
    entries
        .into_iter()
        .map(|entry| (entry.path.clone(), entry))
        .collect()
}

#[allow(dead_code)]
pub fn plan_jobs_for_target(target: &RemoteTarget) -> Result<PlanJobsResult> {
    plan_jobs_with_progress(target, |_completed, _total| {})
}

pub fn plan_jobs_with_progress(
    target: &RemoteTarget,
    mut progress: impl FnMut(usize, usize),
) -> Result<PlanJobsResult> {
    let remote_store = SftpRemoteStore::connect(target)?;
    let local_store = FsLocalStore::default();

    let total_rules = target.rules.len().max(1);
    progress(0, total_rules);

    let mut jobs = Vec::new();
    let mut warnings = Vec::new();

    for (index, rule) in target.rules.iter().enumerate() {
        match plan_single_job(target, rule, &local_store, &remote_store) {
            Ok(job) => jobs.push(job),
            Err(err) => warnings.push(format!(
                "Failed to plan rule {} for {}: {err}",
                rule.local.display(),
                target.name
            )),
        }
        progress(index + 1, total_rules);
    }

    if jobs.is_empty() {
        return Err(anyhow!(
            "no sync plan could be generated for {}",
            target.name
        ));
    }

    Ok(PlanJobsResult { jobs, warnings })
}

fn plan_single_job<L: LocalStore, R: RemoteStore>(
    target: &RemoteTarget,
    rule: &SyncRule,
    local: &L,
    remote: &R,
) -> Result<PlannedJob> {
    let mut resolved_rule = rule.clone();
    resolved_rule.remote = resolve_remote_root(&target.base_path, &rule.remote);

    let local_index = index_entries(local.list(&resolved_rule.local)?);
    let remote_index = index_entries(remote.list(&resolved_rule.remote)?);
    let (actions, stats) = diff_actions(&resolved_rule, &local_index, &remote_index);

    Ok(PlannedJob {
        target_id: target.id,
        rule: resolved_rule,
        local_index,
        remote_index,
        actions,
        stats,
        created_at: SystemTime::now(),
    })
}

fn resolve_remote_root(base_path: &Path, rule_remote: &Path) -> PathBuf {
    if rule_remote.is_absolute() {
        return rule_remote.to_path_buf();
    }

    if base_path.as_os_str().is_empty() {
        return rule_remote.to_path_buf();
    }

    if rule_remote.as_os_str().is_empty() {
        return base_path.to_path_buf();
    }

    base_path.join(rule_remote)
}

#[allow(dead_code)]
pub fn execute_jobs_for_target(target: &RemoteTarget, jobs: &[SyncJob]) -> Result<ExecutionSummary> {
    execute_jobs_with_progress(target, jobs, None, |_completed, _total| {})
}

pub fn execute_jobs_with_progress(
    target: &RemoteTarget,
    jobs: &[SyncJob],
    bandwidth_limit_mbps: Option<u32>,
    mut progress: impl FnMut(usize, usize),
) -> Result<ExecutionSummary> {
    if jobs.is_empty() {
        progress(1, 1);
        return Ok(ExecutionSummary::default());
    }

    let remote_store = SftpRemoteStore::connect(target)
        .with_context(|| format!("failed to connect to {}", target.host))?;
    let local_store = FsLocalStore::default();
    let limiter = bandwidth_limit_mbps.map(|mbps| {
        let bytes_per_sec = (mbps as u64).saturating_mul(125_000);
        Mutex::new(BandwidthLimiter::new(bytes_per_sec))
    });
    let executor = SyncExecutor::new(&local_store, &remote_store, limiter);

    let total_actions: usize = jobs.iter().map(|job| job.plan.actions.len()).sum();
    let mut summary = ExecutionSummary::default();
    let mut completed = 0;
    progress(completed, total_actions.max(1));

    for job in jobs {
        for log in executor.execute(&job.plan) {
            match log.status {
                ActionStatus::Applied => summary.applied += 1,
                ActionStatus::SkippedConflict => summary.skipped += 1,
                ActionStatus::Failed(reason) => {
                    summary.failures.push((log.action.clone(), reason));
                }
            }
            completed += 1;
            progress(completed, total_actions.max(1));
        }
    }

    Ok(summary)
}

pub struct SftpRemoteStore {
    _session: ssh2::Session,
    sftp: Sftp,
}

impl SftpRemoteStore {
    pub fn connect(target: &RemoteTarget) -> Result<Self> {
        let session = connection::establish_session(target)
            .with_context(|| format!("failed to connect to {}", target.host))?;
        let sftp = session.sftp().context("failed to start SFTP subsystem")?;
        Ok(Self {
            _session: session,
            sftp,
        })
    }

    fn collect_entries(
        &self,
        root: &Path,
        rel_path: &Path,
        out: &mut Vec<FileEntry>,
    ) -> Result<()> {
        let dir_path = if rel_path.as_os_str().is_empty() {
            root.to_path_buf()
        } else {
            root.join(rel_path)
        };

        for (entry_path, stat) in self
            .sftp
            .readdir(&dir_path)
            .with_context(|| format!("failed to read {}", dir_path.display()))?
        {
            let Some(name) = entry_path.file_name() else {
                continue;
            };

            if name == OsStr::new(".") || name == OsStr::new("..") {
                continue;
            }

            let child_rel = if rel_path.as_os_str().is_empty() {
                PathBuf::from(name)
            } else {
                rel_path.join(name)
            };

            if stat.is_dir() {
                self.collect_entries(root, &child_rel, out)?;
            } else if stat.is_file() {
                out.push(FileEntry {
                    path: child_rel,
                    kind: EntryKind::File,
                    size: stat.size.unwrap_or(0),
                    modified: stat
                        .mtime
                        .map(|secs| SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
                        .unwrap_or(SystemTime::UNIX_EPOCH),
                });
            }
        }

        Ok(())
    }

    fn absolute_path(&self, root: &Path, rel_path: &Path) -> PathBuf {
        if rel_path.as_os_str().is_empty() {
            root.to_path_buf()
        } else if rel_path.is_absolute() {
            rel_path.to_path_buf()
        } else {
            root.join(rel_path)
        }
    }
}

impl RemoteStore for SftpRemoteStore {
    fn list(&self, root: &Path) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        self.collect_entries(root, Path::new(""), &mut entries)?;
        Ok(entries)
    }

    fn read_file(&self, root: &Path, rel_path: &Path) -> Result<Vec<u8>> {
        let path = self.absolute_path(root, rel_path);
        let mut file = self
            .sftp
            .open(&path)
            .with_context(|| format!("failed to open {}", path.display()))?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .with_context(|| format!("failed to read {}", path.display()))?;
        Ok(buf)
    }

    fn write_file(&self, root: &Path, rel_path: &Path, bytes: &[u8]) -> Result<()> {
        let path = self.absolute_path(root, rel_path);
        if let Some(parent) = rel_path.parent() {
            self.ensure_dir(root, parent)?;
        }
        let mut file = self
            .sftp
            .open_mode(
                &path,
                OpenFlags::WRITE | OpenFlags::TRUNCATE | OpenFlags::CREATE,
                0o644,
                OpenType::File,
            )
            .with_context(|| format!("failed to open {} for write", path.display()))?;
        file.write_all(bytes)
            .with_context(|| format!("failed to write {}", path.display()))
    }

    fn remove_file(&self, root: &Path, rel_path: &Path) -> Result<()> {
        let path = self.absolute_path(root, rel_path);
        self.sftp
            .unlink(&path)
            .with_context(|| format!("failed to remove {}", path.display()))
    }

    fn ensure_dir(&self, root: &Path, rel_path: &Path) -> Result<()> {
        let target = self.absolute_path(root, rel_path);
        let mut current = PathBuf::new();

        for component in target.components() {
            match component {
                Component::RootDir => {
                    current.push(Path::new("/"));
                    continue;
                }
                Component::Prefix(_) => {
                    current.push(component.as_os_str());
                    continue;
                }
                Component::CurDir | Component::ParentDir => continue,
                Component::Normal(part) => current.push(part),
            }

            if current.as_os_str().is_empty() {
                continue;
            }

            if self.sftp.stat(&current).is_ok() {
                continue;
            }

            self.sftp
                .mkdir(&current, 0o755)
                .with_context(|| format!("mkdir {}", current.display()))?;
        }

        Ok(())
    }
}

fn newer(lhs: SystemTime, rhs: SystemTime) -> bool {
    const SKEW: Duration = Duration::from_millis(500);
    lhs.duration_since(rhs)
        .map(|delta| delta > SKEW)
        .unwrap_or(false)
}

pub struct SyncExecutor<'a, L: LocalStore, R: RemoteStore> {
    local: &'a L,
    remote: &'a R,
    limiter: Option<Mutex<BandwidthLimiter>>,
}

#[derive(Clone, Debug)]
pub enum ActionStatus {
    Applied,
    SkippedConflict,
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct ExecutionLog {
    pub action: SyncAction,
    pub status: ActionStatus,
}

impl<'a, L: LocalStore, R: RemoteStore> SyncExecutor<'a, L, R> {
    fn new(
        local: &'a L,
        remote: &'a R,
        limiter: Option<Mutex<BandwidthLimiter>>,
    ) -> Self {
        Self {
            local,
            remote,
            limiter,
        }
    }

    pub fn execute(&self, plan: &SyncPlan) -> Vec<ExecutionLog> {
        plan.actions
            .iter()
            .map(|action| {
                let status = match action {
                    SyncAction::Upload { rel_path, .. } => self
                        .local
                        .read_file(&plan.rule.local, rel_path)
                        .and_then(|bytes| {
                            let parent = rel_path.parent().unwrap_or(Path::new(""));
                            self.remote.ensure_dir(&plan.rule.remote, parent)?;
                            self.throttle(bytes.len());
                            self.remote.write_file(&plan.rule.remote, rel_path, &bytes)
                        })
                        .map(|_| ActionStatus::Applied)
                        .unwrap_or_else(|err| ActionStatus::Failed(err.to_string())),
                    SyncAction::Download { rel_path, .. } => self
                        .remote
                        .read_file(&plan.rule.remote, rel_path)
                        .and_then(|bytes| {
                            let parent = rel_path.parent().unwrap_or(Path::new(""));
                            self.local.ensure_dir(&plan.rule.local, parent)?;
                            self.throttle(bytes.len());
                            self.local.write_file(&plan.rule.local, rel_path, &bytes)
                        })
                        .map(|_| ActionStatus::Applied)
                        .unwrap_or_else(|err| ActionStatus::Failed(err.to_string())),
                    SyncAction::DeleteRemote { rel_path } => self
                        .remote
                        .remove_file(&plan.rule.remote, rel_path)
                        .map(|_| ActionStatus::Applied)
                        .unwrap_or_else(|err| ActionStatus::Failed(err.to_string())),
                    SyncAction::DeleteLocal { rel_path } => self
                        .local
                        .remove_file(&plan.rule.local, rel_path)
                        .map(|_| ActionStatus::Applied)
                        .unwrap_or_else(|err| ActionStatus::Failed(err.to_string())),
                    SyncAction::Conflict { .. } => ActionStatus::SkippedConflict,
                };

                ExecutionLog {
                    action: action.clone(),
                    status,
                }
            })
            .collect()
    }

    fn throttle(&self, bytes: usize) {
        if let Some(limiter) = &self.limiter {
            if let Ok(mut guard) = limiter.lock() {
                guard.consume(bytes as u64);
            }
        }
    }
}

struct BandwidthLimiter {
    limit_per_sec: f64,
    allowance: f64,
    last_check: Instant,
}

impl BandwidthLimiter {
    fn new(limit_bytes_per_sec: u64) -> Self {
        Self {
            limit_per_sec: limit_bytes_per_sec as f64,
            allowance: limit_bytes_per_sec as f64,
            last_check: Instant::now(),
        }
    }

    fn consume(&mut self, bytes: u64) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_check).as_secs_f64();
        self.last_check = now;
        self.allowance =
            (self.allowance + elapsed * self.limit_per_sec).min(self.limit_per_sec);

        let bytes_needed = bytes as f64;
        if self.allowance >= bytes_needed {
            self.allowance -= bytes_needed;
            return;
        }

        let deficit = bytes_needed - self.allowance;
        let sleep_seconds = deficit / self.limit_per_sec;
        if sleep_seconds.is_finite() && sleep_seconds > 0.0 {
            std::thread::sleep(Duration::from_secs_f64(sleep_seconds));
        }
        self.allowance = self.limit_per_sec - deficit.clamp(0.0, self.limit_per_sec);
    }
}

#[derive(Default, Clone)]
pub struct InMemoryRemote {
    entries: Arc<Mutex<HashMap<PathBuf, (Vec<u8>, SystemTime)>>>,
}

impl InMemoryRemote {
    fn now() -> SystemTime {
        SystemTime::now()
    }
}

impl RemoteStore for InMemoryRemote {
    fn list(&self, _root: &Path) -> Result<Vec<FileEntry>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries
            .iter()
            .map(|(path, (bytes, modified))| FileEntry {
                path: path.clone(),
                kind: EntryKind::File,
                size: bytes.len() as u64,
                modified: *modified,
            })
            .collect())
    }

    fn read_file(&self, _root: &Path, rel_path: &Path) -> Result<Vec<u8>> {
        let entries = self.entries.lock().unwrap();
        entries
            .get(rel_path)
            .map(|(bytes, _)| bytes.clone())
            .with_context(|| format!("remote missing {}", rel_path.display()))
    }

    fn write_file(&self, _root: &Path, rel_path: &Path, bytes: &[u8]) -> Result<()> {
        let mut entries = self.entries.lock().unwrap();
        entries.insert(rel_path.to_path_buf(), (bytes.to_vec(), Self::now()));
        Ok(())
    }

    fn remove_file(&self, _root: &Path, rel_path: &Path) -> Result<()> {
        let mut entries = self.entries.lock().unwrap();
        entries.remove(rel_path);
        Ok(())
    }

    fn ensure_dir(&self, _root: &Path, _rel_path: &Path) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct FsLocalStore;

impl FsLocalStore {
    fn full_path(root: &Path, rel_path: &Path) -> PathBuf {
        if rel_path.as_os_str().is_empty() {
            root.to_path_buf()
        } else {
            root.join(rel_path)
        }
    }

    fn collect(root: &Path, rel_path: &Path, output: &mut Vec<FileEntry>) -> Result<()> {
        let dir = Self::full_path(root, rel_path);
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let child_rel = rel_path.join(file_name);
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                Self::collect(root, &child_rel, output)?;
            } else if metadata.is_file() {
                output.push(FileEntry {
                    path: child_rel,
                    kind: EntryKind::File,
                    size: metadata.len(),
                    modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                });
            }
        }

        Ok(())
    }
}

impl LocalStore for FsLocalStore {
    fn list(&self, root: &Path) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        Self::collect(root, Path::new(""), &mut entries)?;
        Ok(entries)
    }

    fn read_file(&self, root: &Path, rel_path: &Path) -> Result<Vec<u8>> {
        let path = Self::full_path(root, rel_path);
        fs::read(&path).with_context(|| format!("failed to read {}", path.display()))
    }

    fn write_file(&self, root: &Path, rel_path: &Path, bytes: &[u8]) -> Result<()> {
        let path = Self::full_path(root, rel_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&path, bytes).with_context(|| format!("failed to write {}", path.display()))
    }

    fn remove_file(&self, root: &Path, rel_path: &Path) -> Result<()> {
        let path = Self::full_path(root, rel_path);
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
        }
        Ok(())
    }

    fn ensure_dir(&self, root: &Path, rel_path: &Path) -> Result<()> {
        let path = Self::full_path(root, rel_path);
        fs::create_dir_all(&path).with_context(|| format!("failed to create {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};
    use tempfile::tempdir;

    #[test]
    fn planner_detects_uploads_and_downloads() {
        let temp = tempdir().unwrap();
        let local_root = temp.path().join("local");
        fs::create_dir_all(&local_root).unwrap();
        fs::write(local_root.join("only_local.txt"), b"local").unwrap();
        fs::write(local_root.join("stale.txt"), b"outdated").unwrap();

        thread::sleep(Duration::from_millis(600));

        let remote = InMemoryRemote::default();
        remote
            .write_file(
                Path::new("/remote"),
                Path::new("only_remote.txt"),
                b"remote",
            )
            .unwrap();
        remote
            .write_file(Path::new("/remote"), Path::new("stale.txt"), b"fresh")
            .unwrap();

        let rule = SyncRule {
            local: local_root.clone(),
            remote: PathBuf::from("/remote"),
            direction: SyncDirection::Bidirectional,
        };

        let local_store = FsLocalStore::default();
        let planner = SyncPlanner::new(&local_store, &remote);
        let plan = planner.plan(&rule).unwrap();

        assert_eq!(plan.stats.uploads, 1);
        assert_eq!(plan.stats.downloads, 2);
        assert_eq!(plan.actions.len(), 3);
    }

    #[test]
    fn executor_applies_plan_against_mock_remote() {
        let temp = tempdir().unwrap();
        let local_root = temp.path().join("local");
        fs::create_dir_all(&local_root).unwrap();
        fs::write(local_root.join("upload.txt"), b"payload").unwrap();

        let remote = InMemoryRemote::default();
        let rule = SyncRule {
            local: local_root.clone(),
            remote: PathBuf::from("/remote"),
            direction: SyncDirection::Push,
        };

        let local_store = FsLocalStore::default();
        let planner = SyncPlanner::new(&local_store, &remote);
        let plan = planner.plan(&rule).unwrap();
        assert_eq!(plan.stats.uploads, 1);

        let executor_store = FsLocalStore::default();
        let executor = SyncExecutor::new(&executor_store, &remote, None);
        let logs = executor.execute(&plan);
        assert!(matches!(logs[0].status, ActionStatus::Applied));

        let bytes = remote
            .read_file(Path::new("/remote"), Path::new("upload.txt"))
            .unwrap();
        assert_eq!(bytes, b"payload");
    }

    #[test]
    fn resolve_remote_root_joins_base_path() {
        let resolved =
            super::resolve_remote_root(Path::new("/srv/www"), Path::new("apps/web"));
        assert_eq!(resolved, PathBuf::from("/srv/www/apps/web"));
    }

    #[test]
    fn resolve_remote_root_preserves_absolute_paths() {
        let resolved = super::resolve_remote_root(Path::new("/srv/www"), Path::new("/data"));
        assert_eq!(resolved, PathBuf::from("/data"));
    }

    #[test]
    fn resolve_remote_root_handles_empty_relative_path() {
        let resolved = super::resolve_remote_root(Path::new("/srv/www"), Path::new(""));
        assert_eq!(resolved, PathBuf::from("/srv/www"));
    }
}
