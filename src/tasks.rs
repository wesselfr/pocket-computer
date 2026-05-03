use core::{cell::RefCell, fmt::Error};

use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::{Mutex, raw::CriticalSectionRawMutex},
    channel::{Channel, Receiver, Sender},
};
use heapless::{String, Vec};
use log::{error, info};

use crate::storage::Storage;

type ScratchBuf = heapless::Vec<u8, 16384>;
static SCRATCH_BUF: Mutex<CriticalSectionRawMutex, RefCell<ScratchBuf>> =
    Mutex::new(RefCell::new(Vec::new()));

type TaskChannel = Channel<CriticalSectionRawMutex, Task, 32>;
type ResultChannel = Channel<CriticalSectionRawMutex, TaskCompleted, 32>;

static TASK_CH: TaskChannel = Channel::new();
static RESULT_CH: ResultChannel = Channel::new();

pub type TaskSender = Sender<'static, CriticalSectionRawMutex, Task, 32>;
pub type TaskReceiver = Receiver<'static, CriticalSectionRawMutex, Task, 32>;
pub type ResultSender = Sender<'static, CriticalSectionRawMutex, TaskCompleted, 32>;
pub type ResultReceiver = Receiver<'static, CriticalSectionRawMutex, TaskCompleted, 32>;

pub type TaskId = u32;

#[derive(Debug, Clone)]
pub enum TaskKind {
    SyncTime,
    HttpsGet { url: String<512> },
    DumpMemFs { len: usize },
}

impl TaskKind {
    fn uses_scratch(&self) -> bool {
        matches!(self, TaskKind::DumpMemFs { .. } | TaskKind::HttpsGet { .. })
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub kind: TaskKind,
}

#[derive(Debug)]
pub enum TaskResult {
    Ok,
    WithData { data: Vec<u8, 512> },
    Failed(TaskError),
}

#[derive(Debug)]
pub enum TaskError {
    QueueFull,
    Busy,
    Network,
    Storage,
    BufferTooSmall,
    Timeout,
}

#[derive(Debug)]
pub struct TaskCompleted {
    pub id: TaskId,
    pub used_scratch: bool,
    pub result: TaskResult,
}

pub struct TaskContext {
    pub storage: Storage<'static>,
}

pub struct TaskSystem {
    next_id: TaskId,
    scratch_busy: bool,
    task_sender: TaskSender,
    task_receiver: TaskReceiver,
    result_sender: ResultSender,
    result_receiver: ResultReceiver,
    completed: Vec<TaskCompleted, 16>,
}

impl TaskSystem {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            scratch_busy: false,
            task_sender: TASK_CH.sender(),
            task_receiver: TASK_CH.receiver(),
            result_sender: RESULT_CH.sender(),
            result_receiver: RESULT_CH.receiver(),
            completed: Vec::new(),
        }
    }

    pub fn init(&self, spawner: Spawner, ctx: TaskContext) {
        spawner.spawn(task_runner_task(self.task_receiver, self.result_sender, ctx).unwrap());
    }

    pub fn update(&mut self) {
        while let Ok(done) = self.result_receiver.try_receive() {
            if done.used_scratch {
                self.scratch_busy = false;
            }
            let _ = self.completed.push(done);
        }
    }

    pub fn dump_memfs(&mut self, fs: &mut mem_fs::MemFs) -> Result<TaskId, TaskError> {
        if self.scratch_busy {
            return Err(TaskError::Busy);
        }

        let len = SCRATCH_BUF.lock(|cell| {
            let mut scratch = cell.borrow_mut();
            scratch.clear();

            let mut failed = false;

            fs.dump(|chunk| {
                if scratch.extend_from_slice(chunk).is_err() {
                    failed = true;
                }
            })
            .map_err(|_| TaskError::BufferTooSmall)?;

            if failed {
                return Err(TaskError::BufferTooSmall);
            }

            Ok::<usize, TaskError>(scratch.len())
        })?;

        self.add_task(TaskKind::DumpMemFs { len })
    }

    pub fn add_task(&mut self, kind: TaskKind) -> Result<TaskId, TaskError> {
        if kind.uses_scratch() && self.scratch_busy {
            return Err(TaskError::Busy);
        }
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);

        let uses_scratch = kind.uses_scratch();
        if uses_scratch {
            self.scratch_busy = true;
        }

        self.task_sender.try_send(Task { id, kind }).map_err(|_| {
            if uses_scratch {
                self.scratch_busy = false;
            }
            TaskError::QueueFull
        })?;

        Ok(id)
    }

    pub fn take_result(&mut self, id: TaskId) -> Option<TaskResult> {
        let index = self.completed.iter().position(|done| done.id == id)?;
        Some(self.completed.swap_remove(index).result)
    }
}

async fn test_task() -> TaskResult {
    info!("Task running...");
    for i in 0..5 {
        info!("Task running: {}", i);
        embassy_time::Timer::after_millis(500).await;
    }
    info!("Task done!");
    TaskResult::Ok
}

async fn dump_memfs(storage: &mut Storage<'_>, len: usize) -> TaskResult {
    info!("Dumping MemFs");
    let result = SCRATCH_BUF.lock(|cell| {
        let scratch = cell.borrow();
        let data = &scratch.as_slice()[..len];
        storage.dump_memfs(data)
    });
    match result {
        Ok(()) => {
            info!("Dump Completed!");
            return TaskResult::Ok;
        }
        Err(e) => {
            error!("{:?}", e);
            return TaskResult::Failed(TaskError::Storage);
        }
    }
}

#[embassy_executor::task]
async fn task_runner_task(task_rx: TaskReceiver, result_tx: ResultSender, mut ctx: TaskContext) {
    loop {
        let task = task_rx.receive().await;

        let result = match task.kind {
            // TaskKind::SyncTime => sync_time().await,
            // TaskKind::HttpsGet { url } => http_get(url).await,
            TaskKind::DumpMemFs { len } => dump_memfs(&mut ctx.storage, len).await,
            _ => test_task().await,
        };

        let _ = result_tx
            .send(TaskCompleted {
                id: task.id,
                used_scratch: task.kind.uses_scratch(),
                result,
            })
            .await;
    }
}
