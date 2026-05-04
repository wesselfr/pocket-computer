use core::{
    cell::RefCell,
    str::{FromStr, from_utf8},
};
use embassy_executor::Spawner;
use embassy_net::{
    Runner, Stack, StackResources,
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
};
use embassy_sync::{
    blocking_mutex::{Mutex, raw::CriticalSectionRawMutex},
    channel::{Channel, Receiver, Sender},
};
use esp_hal::rng::Rng;
use esp_radio::wifi::{Config, Interface, Interfaces, WifiController, sta::StationConfig};
use heapless::{String, Vec, format};
use log::{error, info};
use reqwless::{
    client::HttpClient,
    request::{Method, RequestBuilder},
};

use crate::storage::Storage;

const NUM_CHANNELS: usize = 16;

type ScratchBuf = heapless::Vec<u8, 16384>;
static SCRATCH_BUF: Mutex<CriticalSectionRawMutex, RefCell<ScratchBuf>> =
    Mutex::new(RefCell::new(Vec::new()));

type TaskChannel = Channel<CriticalSectionRawMutex, Task, NUM_CHANNELS>;
type ResultChannel = Channel<CriticalSectionRawMutex, TaskCompleted, NUM_CHANNELS>;

static TASK_CH: TaskChannel = Channel::new();
static RESULT_CH: ResultChannel = Channel::new();

pub type TaskSender = Sender<'static, CriticalSectionRawMutex, Task, NUM_CHANNELS>;
pub type TaskReceiver = Receiver<'static, CriticalSectionRawMutex, Task, NUM_CHANNELS>;
pub type ResultSender = Sender<'static, CriticalSectionRawMutex, TaskCompleted, NUM_CHANNELS>;
pub type ResultReceiver = Receiver<'static, CriticalSectionRawMutex, TaskCompleted, NUM_CHANNELS>;

pub type TaskId = u32;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[derive(Debug, Clone)]
pub enum TaskKind {
    SyncTime,
    HttpsGet {
        host: String<512>,
        port: u16,
        path: String<512>,
    },
    UpdateWifiCredentials {
        ssid: String<512>,
        password: String<512>,
    },
    Connect,
    DumpMemFs {
        len: usize,
    },
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
    // TODO: Use a dedicated buffer for this.
    WithData { data: Vec<u8, 2048> },
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

pub struct NetworkContext {
    pub controller: WifiController<'static>,
    pub interface: Interfaces<'static>,
    pub stack: Stack<'static>,
    pub tcp: TcpClient<'static, 1, 1500, 1500>,
    pub dns: DnsSocket<'static>,
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

    pub fn init(
        &self,
        spawner: Spawner,
        ctx: TaskContext,
        wifi_controller: WifiController<'static>,
        wifi_interface: Interfaces<'static>,
    ) {
        // TODO: Move wifi setup.
        let config = embassy_net::Config::dhcpv4(Default::default());

        let rng = Rng::new();
        let seed = (rng.random() as u64) << 32 | rng.random() as u64;

        let (stack, runner) = embassy_net::new(
            wifi_interface.station,
            config,
            mk_static!(StackResources<3>, StackResources::<3>::new()),
            seed,
        );

        let tcp = TcpClient::new(
            stack,
            mk_static!(
                TcpClientState<1, 1500, 1500>,
                TcpClientState::<1, 1500, 1500>::new()
            ),
        );
        let dns = DnsSocket::new(stack);

        // Net task runner
        spawner.spawn(net_task(runner).unwrap());

        // General task runner
        spawner.spawn(
            task_runner_task(
                self.task_receiver,
                self.result_sender,
                ctx,
                NetworkContext {
                    controller: wifi_controller,
                    interface: wifi_interface,
                    stack,
                    tcp,
                    dns,
                },
            )
            .unwrap(),
        );
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

    pub fn add_http_task(
        &mut self,
        host: &str,
        port: u16,
        path: &str,
    ) -> Result<TaskId, TaskError> {
        self.add_task(TaskKind::HttpsGet {
            host: String::from_str(host).unwrap_or_default(),
            port,
            path: String::from_str(path).unwrap_or_default(),
        })
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

async fn connect(controller: &mut WifiController<'static>) -> TaskResult {
    info!("start connection task");
    match controller.connect_async().await {
        Ok(info) => {
            info!("Wifi connected to {:?}", info);
            return TaskResult::Ok;
        }
        Err(e) => {
            error!("Failed to connect to wifi: {e:?}");
            return TaskResult::Failed(TaskError::Network);
        }
    }
}

async fn http_get(
    host: &str,
    port: u16,
    path: &str,
    method: Method,
    net_ctx: &mut NetworkContext,
) -> TaskResult {
    info!("Reaching out to: {}:{}/{}", host, port, path);

    if !net_ctx.controller.is_connected() {
        let res = connect(&mut net_ctx.controller).await;
        if let TaskResult::Failed(e) = res {
            error!("Failed to connect.");
            return TaskResult::Failed(e);
        }

        info!("Waiting for link...");
        net_ctx.stack.wait_link_up().await;

        info!("Waiting for DHCP config...");
        net_ctx.stack.wait_config_up().await;
    }

    if let Some(config) = net_ctx.stack.config_v4() {
        info!("IP config: {:?}", config);
    }

    let mut client = HttpClient::new(&net_ctx.tcp, &net_ctx.dns);

    let host = format!(1024; "{}:{}", host, port).unwrap_or_default();
    let url = format!(2048; "http://{}/{}", host, path).unwrap_or_default();
    info!("URL: {}", url);

    if let Ok(req) = client.request(method, &url).await {
        let mut req = req.content_type(reqwless::headers::ContentType::TextHtml);

        let mut rx_buf = [0u8; 4096];
        let result = req.send(&mut rx_buf).await;

        match result {
            Ok(response) => {
                let data = response.body().read_to_end().await.unwrap_or_default();
                info!(
                    "Response with data: {}",
                    from_utf8(&data).unwrap_or_default()
                );

                let len = data.len();
                if let Ok(data) = Vec::from_slice(data) {
                    return TaskResult::WithData { data };
                } else {
                    error!("Buffer too small. Required size: {}", len);
                    return TaskResult::Failed(TaskError::BufferTooSmall);
                }
            }
            Err(e) => {
                error!("{:?}", e);
                return TaskResult::Failed(TaskError::Network);
            }
        }
    } else {
        error!("Failed..");
    }

    TaskResult::Ok
}

async fn update_credentials(
    controller: &mut WifiController<'static>,
    ssid: &str,
    password: &str,
) -> TaskResult {
    let station_config = Config::Station(
        StationConfig::default()
            .with_ssid(ssid)
            .with_password(password.into()),
    );

    match controller.set_config(&station_config) {
        Ok(()) => return TaskResult::Ok,
        Err(error) => {
            error!("Failed to update network credentials: {:?}", error);
            return TaskResult::Failed(TaskError::Network);
        }
    }
}

#[embassy_executor::task]
async fn task_runner_task(
    task_rx: TaskReceiver,
    result_tx: ResultSender,
    mut ctx: TaskContext,
    mut net_ctx: NetworkContext,
) {
    loop {
        let task = task_rx.receive().await;

        let result = match task.kind {
            // TaskKind::SyncTime => sync_time().await,
            TaskKind::HttpsGet {
                ref host,
                port,
                ref path,
            } => http_get(&host, port, &path, Method::GET, &mut net_ctx).await,
            TaskKind::Connect => connect(&mut net_ctx.controller).await,
            TaskKind::UpdateWifiCredentials {
                ref ssid,
                ref password,
            } => update_credentials(&mut net_ctx.controller, &ssid, &password).await,
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

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface<'static>>) {
    runner.run().await
}
