use std::{
    path::{Path, PathBuf},
    sync::{mpsc, Arc},
    thread,
};

use anyhow::{Context, Result};
use async_channel::Sender;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use parking_lot::Mutex;

use crate::model::TargetId;

#[derive(Clone)]
pub struct WatchTarget {
    pub target_id: TargetId,
    pub roots: Vec<PathBuf>,
}

#[derive(Clone)]
pub struct WatchEvent {
    pub target_id: TargetId,
}

enum Command {
    Configure {
        enabled: bool,
        targets: Vec<WatchTarget>,
    },
    #[allow(dead_code)]
    Shutdown,
}

struct ActiveWatcher {
    _watcher: RecommendedWatcher,
    #[allow(dead_code)]
    roots: Arc<Vec<(PathBuf, TargetId)>>,
}

static COMMAND_TX: Lazy<Mutex<Option<mpsc::Sender<Command>>>> = Lazy::new(|| Mutex::new(None));

pub fn ensure_service(event_tx: Sender<WatchEvent>) {
    let mut guard = COMMAND_TX.lock();
    if guard.is_none() {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let event_tx_clone = event_tx.clone();
        thread::spawn(move || run_watch_loop(cmd_rx, event_tx_clone));
        *guard = Some(cmd_tx);
    }
}

pub fn configure(enabled: bool, targets: Vec<WatchTarget>) {
    if let Some(tx) = COMMAND_TX.lock().as_ref() {
        let _ = tx.send(Command::Configure { enabled, targets });
    }
}

#[allow(dead_code)]
pub fn shutdown() {
    if let Some(tx) = COMMAND_TX.lock().take() {
        let _ = tx.send(Command::Shutdown);
    }
}

fn run_watch_loop(cmd_rx: mpsc::Receiver<Command>, event_tx: Sender<WatchEvent>) {
    let mut _active: Option<ActiveWatcher> = None;
    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            Command::Shutdown => {
                _active = None;
                break;
            }
            Command::Configure { enabled, targets } => {
                if !enabled || targets.is_empty() {
                    _active = None;
                    continue;
                }
                match build_watcher(targets, event_tx.clone()) {
                    Ok(watcher) => _active = Some(watcher),
                    Err(err) => {
                        eprintln!("watcher configuration failed: {err:?}");
                        _active = None;
                    }
                };
            }
        }
    }
}

fn build_watcher(targets: Vec<WatchTarget>, event_tx: Sender<WatchEvent>) -> Result<ActiveWatcher> {
    let roots: Vec<(PathBuf, TargetId)> = targets
        .into_iter()
        .flat_map(|target| {
            target
                .roots
                .into_iter()
                .map(move |root| (root, target.target_id))
        })
        .collect();

    let roots_arc = Arc::new(roots);
    let callback_roots = roots_arc.clone();
    let watcher_event_tx = event_tx.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if let Some(first_path) = event.paths.first() {
                    if let Some(target_id) = match_target(callback_roots.clone(), first_path) {
                        let _ = watcher_event_tx.try_send(WatchEvent { target_id });
                    }
                }
            }
        },
        Config::default().with_poll_interval(std::time::Duration::from_secs(2)),
    )
    .context("failed to start filesystem watcher")?;

    for (root, _) in roots_arc.iter() {
        if !root.exists() {
            continue;
        }
        watcher
            .watch(root, RecursiveMode::Recursive)
            .with_context(|| format!("failed to watch {}", root.display()))?;
    }

    Ok(ActiveWatcher {
        _watcher: watcher,
        roots: roots_arc,
    })
}

fn match_target(roots: Arc<Vec<(PathBuf, TargetId)>>, path: &Path) -> Option<TargetId> {
    roots.iter().find_map(|(root, target)| {
        if path.starts_with(root) {
            Some(*target)
        } else {
            None
        }
    })
}
