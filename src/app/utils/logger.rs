#![macro_use]

use {
    crate::app::utils::{
        user_io::Keyboard,
        concurrency::channel::Channel,
    },
    std::{sync::Mutex, collections::VecDeque, borrow::Cow},
    lazy_static::lazy_static,
};

lazy_static! {
    static ref CHANNEL: Mutex<Channel<Message>> = Mutex::new(Channel::default());
}

static LOG_MESSAGES: Mutex<VecDeque<Message>> = Mutex::new(VecDeque::new());

pub type MsgStr = Cow<'static, str>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Message {
    pub content: MsgStr,
    pub from: MsgStr,
    pub msg_type: MsgType,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "[{msg_type}]-[{from}]: {content}",
            msg_type = self.msg_type,
            from = self.from,
            content = self.content,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum MsgType {
    #[default]
    Info,
    Error,
}

impl std::fmt::Display for MsgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Error => write!(f, "ERROR"),
        }
    }
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

pub fn log(msg_type: MsgType, from: MsgStr, content: MsgStr) {
    CHANNEL.lock()
        .expect("channel mutex should be not poisoned")
        .sender
        .send(Message { msg_type, from, content })
        .expect("failed to send message");
}

#[macro_export]
macro_rules! log {
    ($msg_type:ident, $from:expr, $content:expr) => {{
        use $crate::app::utils::logger::{self, MsgType, MsgStr};
        logger::log(MsgType::$msg_type, MsgStr::from($from), MsgStr::from($content));
    }};
}

pub use crate::log;

pub fn spawn_window(ui: &imgui::Ui, keyboard: &Keyboard) {
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

    make_window(ui, "Log list", keyboard)
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

            let mut buf = String::with_capacity(256);
            let is_enter_pressed = ui.input_text("Console", &mut buf)
                .enter_returns_true(true)
                .build();

            let gil = Python::acquire_gil();
            let py = gil.python();
        
            let voxel_set = py_fn!(py, voxel_set(x: i32, y: i32, z: i32, new_id: u16) -> PyResult<i32> {
                command(Command::SetVoxel { pos: veci!(x, y, z), new_id });
                Ok(0)
            });

            let drop_all_meshes = py_fn!(py, drop_all_meshes() -> PyResult<i32> {
                command(Command::DropAllMeshes);
                Ok(0)
            });

            let locals = PyDict::new(py);

            locals.set_item(py, "voxel_set", voxel_set)
                .unwrap_or_else(|err|
                    log!(Error, "logger", format!("failed to set 'voxel_set' item: {err:?}"))
                );
                
            locals.set_item(py, "drop_all_meshes", drop_all_meshes)
                .unwrap_or_else(|err|
                    log!(Error, "logger", format!("failed to set 'drop_all_meshes' item: {err:?}"))
                );

            if is_enter_pressed {
                py.run(&buf, None, Some(&locals))
                    .unwrap_or_else(|err| log!(Error, "logger", format!("{err:?}")));
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