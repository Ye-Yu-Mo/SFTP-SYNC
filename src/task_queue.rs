use std::{thread, thread::available_parallelism};

use anyhow::Result;
use async_channel::{bounded, Receiver as AsyncReceiver, Sender as AsyncSender};
use crossbeam_channel::{unbounded, Receiver as SyncReceiver, Sender as SyncSender};
use once_cell::sync::Lazy;

use crate::{
    model::RemoteTarget,
    sync::{
        execute_jobs_for_target, plan_jobs_for_target, ExecutionSummary, PlanJobsResult, SyncJob,
    },
};

type PlanResponder = AsyncSender<Result<PlanJobsResult>>;
type ExecuteResponder = AsyncSender<Result<ExecutionSummary>>;

enum TaskMessage {
    Plan {
        target: RemoteTarget,
        respond_to: PlanResponder,
    },
    Execute {
        target: RemoteTarget,
        jobs: Vec<SyncJob>,
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
                        let result = plan_jobs_for_target(&target);
                        let _ = respond_to.send_blocking(result);
                    }
                    TaskMessage::Execute {
                        target,
                        jobs,
                        respond_to,
                    } => {
                        let result = execute_jobs_for_target(&target, &jobs);
                        let _ = respond_to.send_blocking(result);
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

pub fn submit_plan(target: RemoteTarget) -> AsyncReceiver<Result<PlanJobsResult>> {
    let (tx, rx) = bounded(1);
    TASK_QUEUE.submit(TaskMessage::Plan {
        target,
        respond_to: tx,
    });
    rx
}

pub fn submit_execute(
    target: RemoteTarget,
    jobs: Vec<SyncJob>,
) -> AsyncReceiver<Result<ExecutionSummary>> {
    let (tx, rx) = bounded(1);
    TASK_QUEUE.submit(TaskMessage::Execute {
        target,
        jobs,
        respond_to: tx,
    });
    rx
}
