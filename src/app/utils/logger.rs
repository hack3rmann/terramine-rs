#![macro_use]
#![allow(clippy::manual_strip, clippy::too_many_arguments)]

use {
    crate::{
        prelude::*,
        concurrency::channel::Channel,
    },
    std::sync::Mutex,
};

lazy_static! {
    static ref CHANNEL: Mutex<Channel<Message>> = Mutex::new(Channel::default());
}

static LOG_MESSAGES: Mutex<VecDeque<Message>> = Mutex::new(VecDeque::new());

pub type CowStr = Cow<'static, str>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Display)]
#[display("[{msg_type}]-[{from}]: {content}")]
pub struct Message {
    pub content: CowStr,
    pub from: CowStr,
    pub msg_type: MsgType,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Display)]
#[display(style = "UPPERCASE")]
pub enum MsgType {
    #[default]
    Info,
    Error,
}

pub fn recv_all() {
    let mut channel = CHANNEL.lock()
        .expect("channel mutex should be not poisoned");

    let mut messages = LOG_MESSAGES.lock()
        .expect("messages mutex should be not poisoned");

    while let Ok(msg) = channel.receiver.try_recv() {
        messages.push_back(msg);
    }
}

pub fn log(msg_type: MsgType, from: impl Into<CowStr>, content: impl Into<CowStr>) {
    let (from, content) = (from.into(), content.into());

    eprintln!("{msg_type} from {from}: {content}");
    CHANNEL.lock()
        .expect("channel mutex should be not poisoned")
        .sender
        .send(Message { msg_type, from, content})
        .expect("failed to send message");
}

pub fn work(from: impl Into<CowStr>, work: impl Into<CowStr>) -> WorkLogGuard {
    WorkLogGuard::new(from, work)
}

#[must_use]
#[derive(Debug)]
pub struct WorkLogGuard {
    pub from: CowStr,
    pub work: CowStr,
}

impl WorkLogGuard {
    pub fn new(from: impl Into<CowStr>, work: impl Into<CowStr>) -> Self {
        let (from, work) = (from.into(), work.into());
        log!(Info, from = from.clone(), "Start {work}");
        Self { from, work }
    }
}

impl Drop for WorkLogGuard {
    fn drop(&mut self) {
        let from = mem::take(&mut self.from);
        log!(Info, from = from, "End {work}", work = self.work);
    }
}

#[macro_export]
macro_rules! log {
    ($msg_type:ident, from = $from:expr, $($content:tt)*) => {{
        use $crate::app::utils::logger::{log, MsgType::*};
        log($msg_type, $from, std::fmt::format(format_args!($($content)*)));
    }};
}

#[macro_export]
macro_rules! log_dbg {
    ($expr:expr) => {{
        use $crate::app::utils::logger::log;
        let result = $expr;
        log!(Info, from = "dbg", "{expr} = {result:?}", expr = stringify!($expr));
        result
    }};
}

#[macro_export]
macro_rules! work {
    (from = $from:expr, $($content:tt)*) => {{
        use $crate::app::utils::logger::work;
        work($from, std::fmt::format(format_args!($($content)*)))
    }};
}

pub use crate::{log, log_dbg, work};

pub fn spawn_window(ui: &imgui::Ui) {
    use {
        crate::app::utils::{
            graphics::ui::imgui_constructor::make_window,
        },
        cpython::{Python, PyResult, py_fn, PyDict},
    };

    const ERROR_COLOR: [f32; 4] = [0.8, 0.1, 0.05, 1.0];
    const INFO_COLOR:  [f32; 4] = [1.0, 1.0, 1.0,  1.0];

    const PADDING: f32 = 10.0;
    const HEIGHT:  f32 = 300.0;

    let [width, height] = ui.io().display_size;

    make_window(ui, "Log list")
        .collapsed(true, imgui::Condition::Appearing)
        .save_settings(false)
        .collapsible(true)
        .bg_alpha(0.8)
        .position([PADDING, height - PADDING], imgui::Condition::Always)
        .position_pivot([0.0, 1.0])
        .size([width - 2.0 * PADDING, HEIGHT], imgui::Condition::Always)
        .build(|| {
            use crate::app::utils::terrain::chunk::commands::{Command, command};

            let messages = LOG_MESSAGES.lock()
                .expect("messages lock should be not poisoned");

            static INPUT: Mutex<String> = Mutex::new(String::new());
            let mut input = INPUT.lock()
                .unwrap();

            let is_enter_pressed = ui.input_text("Console", &mut input)
                .enter_returns_true(true)
                .build();

            let buf = input.replace("^;", "\n");

            let gil = Python::acquire_gil();
            let py = gil.python();
        
            let voxel_set = py_fn!(py, voxel_set(x: i32, y: i32, z: i32, new_id: u16) -> PyResult<i32> {
                command(Command::SetVoxel { pos: veci!(x, y, z), new_id });
                Ok(0)
            });

            let voxel_fill = py_fn!(py, voxel_fill(
                sx: i32, sy: i32, sz: i32,
                ex: i32, ey: i32, ez: i32, new_id: u16
            ) -> PyResult<i32> {
                command(Command::FillVoxels { pos_from: veci!(sx, sy, sz), pos_to: veci!(ex, ey, ez), new_id });
                Ok(0)
            });

            let drop_all_meshes = py_fn!(py, drop_all_meshes() -> PyResult<i32> {
                command(Command::DropAllMeshes);
                Ok(0)
            });

            let locals = PyDict::new(py);

            locals.set_item(py, "voxel_set", voxel_set)
                .unwrap_or_else(|err|
                    log!(Error, from = "logger", "failed to set 'voxel_set' item: {err:?}")
                );

            locals.set_item(py, "voxel_fill", voxel_fill)
                .unwrap_or_else(|err|
                    log!(Error, from = "logger", "failed to set 'voxel_fill' item: {err:?}")
                );
                
            locals.set_item(py, "drop_all_meshes", drop_all_meshes)
                .unwrap_or_else(|err|
                    log!(Error, from = "logger", "failed to set 'drop_all_meshes' item: {err:?}")
                );

            if is_enter_pressed {
                py.run(&buf, None, Some(&locals))
                    .unwrap_or_else(|err| log!(Error, from = "logger", "{err:?}"));
            }

            for msg in messages.iter().rev() {
                let color = match msg.msg_type {
                    MsgType::Error => ERROR_COLOR,
                    MsgType::Info  => INFO_COLOR,
                };

                ui.text_colored(color, &format!("[LOG]: {msg}"));
            }
        });
}

pub trait LogError<T> {
    fn log_error(self, from: impl Into<CowStr>, msg: impl Into<CowStr>) -> T where T: Default;
    fn log_error_or(self, from: impl Into<CowStr>, msg: impl Into<CowStr>, default: T) -> T;
    fn log_error_or_else(self, from: impl Into<CowStr>, msg: impl Into<CowStr>, f: impl FnOnce() -> T) -> T;
}

impl<T, E: std::error::Error> LogError<T> for Result<T, E> {
    fn log_error(self, from: impl Into<CowStr>, msg: impl Into<CowStr>) -> T where T: Default {
        match self {
            Ok(item) => item,
            Err(err) => {
                let msg = msg.into();
                log!(Error, from = from, "{msg}: {err}");
                Default::default()
            }
        }
    }

    fn log_error_or(self, from: impl Into<CowStr>, msg: impl Into<CowStr>, default: T) -> T {
        match self {
            Ok(item) => item,
            Err(err) => {
                let msg = msg.into();
                log!(Error, from = from, "{msg}: {err}");
                default
            }
        }
    }

    fn log_error_or_else(self, from: impl Into<CowStr>, msg: impl Into<CowStr>, f: impl FnOnce() -> T) -> T {
        match self {
            Ok(item) => item,
            Err(err) => {
                let msg = msg.into();
                log!(Error, from = from, "{msg}: {err}");
                f()
            }
        }
    }
}