use std::{thread, thread::available_parallelism};

use anyhow::Result;
use async_channel::{bounded, Receiver as AsyncReceiver, Sender as AsyncSender};
use crossbeam_channel::{unbounded, Receiver as SyncReceiver, Sender as SyncSender};
use once_cell::sync::Lazy;

use crate::{
    model::{AppSettings, RemoteTarget},
    sync::{
        execute_jobs_with_progress, plan_jobs_with_progress, ExecutionSummary, PlanJobsResult,
        SyncJob,
    },
};

pub enum TaskEvent<T> {
    Progress { completed: usize, total: usize },
    Finished(Result<T>),
}

type PlanResponder = AsyncSender<TaskEvent<PlanJobsResult>>;
type ExecuteResponder = AsyncSender<TaskEvent<ExecutionSummary>>;

enum TaskMessage {
    Plan {
        target: RemoteTarget,
        respond_to: PlanResponder,
    },
    Execute {
        target: RemoteTarget,
        jobs: Vec<SyncJob>,
        settings: AppSettings,
        respond_to: ExecuteResponder,
    },
}

struct TaskQueue {
    sender: SyncSender<TaskMessage>,
}

impl TaskQueue {
    fn new(worker_count: usize) -> Self {
        let (tx, rx) = unbounded();
        for index in 0..worker_count {
            spawn_worker(rx.clone(), index);
        }
        Self { sender: tx }
    }

    fn submit(&self, task: TaskMessage) {
        let _ = self.sender.send(task);
    }
}

fn spawn_worker(receiver: SyncReceiver<TaskMessage>, index: usize) {
    thread::Builder::new()
        .name(format!("task-worker-{index}"))
        .spawn(move || {
            while let Ok(task) = receiver.recv() {
                match task {
                    TaskMessage::Plan { target, respond_to } => {
                        let rules_total = target.rules.len().max(1);
                        let _ = respond_to.send_blocking(TaskEvent::Progress {
                            completed: 0,
                            total: rules_total,
                        });
                        let result = plan_jobs_with_progress(&target, |completed, total| {
                            let total = total.max(1);
                            let _ = respond_to.send_blocking(TaskEvent::Progress {
                                completed: completed.min(total),
                                total,
                            });
                        });
                        let _ = respond_to.send_blocking(TaskEvent::Finished(result));
                    }
                    TaskMessage::Execute {
                        target,
                        jobs,
                        settings,
                        respond_to,
                    } => {
                        let total_actions: usize =
                            jobs.iter().map(|job| job.plan.actions.len()).sum::<usize>().max(1);
                        let _ = respond_to.send_blocking(TaskEvent::Progress {
                            completed: 0,
                            total: total_actions,
                        });
                        let limit = if settings.limit_bandwidth {
                            Some(settings.bandwidth_mbps)
                        } else {
                            None
                        };
                        let result =
                            execute_jobs_with_progress(&target, &jobs, limit, |completed, total| {
                                let total = total.max(1);
                                let _ = respond_to.send_blocking(TaskEvent::Progress {
                                    completed: completed.min(total),
                                    total,
                                });
                            });
                        let _ = respond_to.send_blocking(TaskEvent::Finished(result));
                    }
                }
            }
        })
        .expect("failed to spawn task worker");
}

static TASK_QUEUE: Lazy<TaskQueue> = Lazy::new(|| {
    let workers = available_parallelism()
        .map(|n| n.get().clamp(2, 4))
        .unwrap_or(2);
    TaskQueue::new(workers)
});

pub fn submit_plan(target: RemoteTarget) -> AsyncReceiver<TaskEvent<PlanJobsResult>> {
    let (tx, rx) = bounded(16);
    TASK_QUEUE.submit(TaskMessage::Plan {
        target,
        respond_to: tx,
    });
    rx
}

pub fn submit_execute(
    target: RemoteTarget,
    jobs: Vec<SyncJob>,
    settings: AppSettings,
) -> AsyncReceiver<TaskEvent<ExecutionSummary>> {
    let (tx, rx) = bounded(16);
    TASK_QUEUE.submit(TaskMessage::Execute {
        target,
        jobs,
        settings,
        respond_to: tx,
    });
    rx
}
