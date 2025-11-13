use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use anyhow::{Context, Result};

use crate::model::{SyncDirection, SyncRule};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
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
    Upload { rel_path: PathBuf, size: u64 },
    Download { rel_path: PathBuf, size: u64 },
    DeleteRemote { rel_path: PathBuf },
    DeleteLocal { rel_path: PathBuf },
    Conflict { rel_path: PathBuf },
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
    pub stats: PlanStats,
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
        let local_entries = self.local.list(&rule.local)?;
        let remote_entries = self.remote.list(&rule.remote)?;

        let local_index = index_entries(local_entries);
        let remote_index = index_entries(remote_entries);

        let mut actions = Vec::new();
        let mut stats = PlanStats::default();

        for (path, local_entry) in &local_index {
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

        for (path, remote_entry) in &remote_index {
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

        Ok(SyncPlan {
            rule: rule.clone(),
            actions,
            stats,
        })
    }
}

fn index_entries(entries: Vec<FileEntry>) -> HashMap<PathBuf, FileEntry> {
    entries
        .into_iter()
        .map(|entry| (entry.path.clone(), entry))
        .collect()
}

fn newer(lhs: SystemTime, rhs: SystemTime) -> bool {
    const SKEW: Duration = Duration::from_millis(500);
    lhs.duration_since(rhs)
        .map(|delta| delta > SKEW)
        .unwrap_or(false)
}

#[derive(Clone)]
pub struct SyncExecutor<'a, L: LocalStore, R: RemoteStore> {
    local: &'a L,
    remote: &'a R,
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
    pub fn new(local: &'a L, remote: &'a R) -> Self {
        Self { local, remote }
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
        let executor = SyncExecutor::new(&executor_store, &remote);
        let logs = executor.execute(&plan);
        assert!(matches!(logs[0].status, ActionStatus::Applied));

        let bytes = remote
            .read_file(Path::new("/remote"), Path::new("upload.txt"))
            .unwrap();
        assert_eq!(bytes, b"payload");
    }
}
