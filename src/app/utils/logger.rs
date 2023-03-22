#![macro_use]

use {
    crate::app::utils::{user_io::Keyboard, terrain::chunk::chunk_array::ChunkArray},
    std::{sync::Mutex, collections::VecDeque, borrow::Cow},
    tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, self},
    lazy_static::lazy_static,
};

lazy_static! {
    static ref CHANNEL: Mutex<Channel> = Mutex::new(Channel::default());
}

static LOG_MESSAGES: Mutex<VecDeque<Message>> = Mutex::new(VecDeque::new());

#[derive(Debug)]
struct Channel {
    sender: UnboundedSender<Message>,
    receiver: UnboundedReceiver<Message>,
}

impl Default for Channel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }
}

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

pub fn spawn_window(ui: &imgui::Ui, keyboard: &Keyboard, chunk_arr: &mut ChunkArray) {
    use {
        crate::app::utils::{
            graphics::ui::imgui_constructor::make_window,
            terrain::voxel::voxel_data::Id,
        },
        cpython::{Python, PyResult, PyErr, py_fn, PyDict, PyString},
        std::sync::atomic::{AtomicUsize, Ordering},
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
            let messages = LOG_MESSAGES.lock()
                .expect("messages lock should be not poisoned");

            let mut buf = String::with_capacity(256);
            let is_enter_pressed = ui.input_text("Console", &mut buf)
                .enter_returns_true(true)
                .build();

            let gil = Python::acquire_gil();
            let py = gil.python();
        
            // FIXME:

            static CHUNK_ARR_ADDR: AtomicUsize = AtomicUsize::new(0);
            CHUNK_ARR_ADDR.store((chunk_arr as *const _) as usize, Ordering::SeqCst);

            let voxel_set = py_fn!(py, voxel_set(x: i32, y: i32, z: i32, new_id: u16) -> PyResult<Id> {
                let chunk_arr_addr = CHUNK_ARR_ADDR.load(Ordering::SeqCst);
                let chunk_arr = unsafe {
                    (chunk_arr_addr as *mut ChunkArray)
                        .as_mut()
                        .ok_or_else(|| PyErr::new::<PyString, _>(py, PyString::new(py, "chunk_arr ptr is null")))?
                };

                let old_id = chunk_arr.set_voxel(veci!(x, y, z), new_id)
                    .map_err(|err| PyErr::new::<PyString, _>(py, PyString::new(py, &err.to_string())))?;

                Ok(old_id)
            });

            let locals = PyDict::new(py);
            locals.set_item(py, "voxel_set", voxel_set)
                .unwrap_or_else(|err|
                    log!(Error, "logger", format!("failed to set 'voxel_set' item: {err:?}"))
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