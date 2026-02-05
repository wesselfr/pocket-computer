use heapless::{
    String, Vec,
    spsc::{Consumer, Producer, Queue},
};
use log::info;

pub type TaskId = u32;

#[derive(Debug, Clone)]
pub enum TaskKind {
    SyncTime,
    HttpsGet { url: String<512> },
    DumpMemFs,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub kind: TaskKind,
}

#[derive(Debug, Clone)]
pub enum TaskResult {
    Ok,
    WithData { data: Vec<u8, 512> },
    Failed,
}

#[derive(Debug, Clone)]
pub struct TaskCompleted {
    pub id: TaskId,
    pub result: TaskResult,
}

pub struct TaskSystem {
    task_q: Queue<Task, 32>,
    res_q: Queue<TaskCompleted, 32>,
}

impl TaskSystem {
    pub const fn new() -> Self {
        Self {
            task_q: Queue::new(),
            res_q: Queue::new(),
        }
    }

    pub fn split(&mut self) -> (TaskQueueHandle<'_>, TaskRunner<'_>) {
        let (task_prod, task_cons) = self.task_q.split();
        let (res_prod, res_cons) = self.res_q.split();

        (
            TaskQueueHandle {
                next_id: 1,
                task_prod,
                res_cons,
            },
            TaskRunner {
                task_cons,
                res_prod,
            },
        )
    }
}

impl Default for TaskSystem {
    fn default() -> Self {
        TaskSystem::new()
    }
}

pub struct TaskQueueHandle<'a> {
    next_id: TaskId,
    task_prod: Producer<'a, Task>,
    res_cons: Consumer<'a, TaskCompleted>,
}

impl<'a> TaskQueueHandle<'a> {
    pub fn add_task(&mut self, kind: TaskKind) -> Result<TaskId, ()> {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);

        self.task_prod.enqueue(Task { id, kind }).map_err(|_| ())?;
        Ok(id)
    }

    pub fn poll_any(&mut self) -> Option<TaskCompleted> {
        self.res_cons.dequeue()
    }
}

pub struct TaskRunner<'a> {
    task_cons: Consumer<'a, Task>,
    res_prod: Producer<'a, TaskCompleted>,
}

impl<'a> TaskRunner<'a> {
    pub fn update(&mut self) {
        if let Some(task) = self.task_cons.dequeue() {
            info!("Found tasks: {:?}", task);
            let result = match task.kind {
                TaskKind::SyncTime => TaskResult::Ok,
                TaskKind::HttpsGet { url: _ } => TaskResult::WithData { data: Vec::new() },
                TaskKind::DumpMemFs => TaskResult::Ok,
            };

            let _ = self.res_prod.enqueue(TaskCompleted {
                id: task.id,
                result,
            });
        }
    }
}
