#![allow(clippy::manual_strip, clippy::too_many_arguments)]

use crate::{
    prelude::*,
    concurrency::channel::Channel,
};



module_constructor! {
    // use crate::graphics::ui::imgui_ext::push_window_builder_lock_free;

    env_logger::init();

    // * Safety
    // * 
    // * Safe, because it's going on in module
    // * constructor, so no one access the update list.
    unsafe {
        // FIXME:
        // push_window_builder_lock_free(spawn_window);
        app::update::push_function_lock_free(update);
    }
}



lazy_static! {
    static ref CHANNEL: Mutex<Channel<Message>> = Mutex::new(Channel::default());
}

static LOG_MESSAGES: Mutex<VecDeque<Message>> = Mutex::new(VecDeque::new());



#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Display)]
#[display("[{msg_type}]-[{from}]: {content}")]
pub struct Message {
    pub content: StaticStr,
    pub from: StaticStr,
    pub msg_type: MsgType,
}
assert_impl_all!(Message: Send, Sync);



#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Display)]
#[display(style = "UPPERCASE")]
pub enum MsgType {
    #[default]
    Info,
    Error,
}



pub fn recv_all() {
    let mut channel = CHANNEL.lock();
    let mut messages = LOG_MESSAGES.lock();

    while let Ok(msg) = channel.receiver.try_recv() {
        messages.push_back(msg);
    }
}

pub fn update() {
    recv_all();
}

pub fn log(msg_type: MsgType, from: impl Into<StaticStr>, content: impl Into<StaticStr>) {
    let (from, content) = (from.into(), content.into());

    eprintln!("{msg_type} from {from}: {content}");
    CHANNEL.lock()
        .sender
        .send(Message { msg_type, from, content})
        .expect("failed to send message");
}

pub fn scope(from: impl Into<StaticStr>, work: impl Into<StaticStr>) -> LogGuard {
    LogGuard::new(from, work)
}



#[must_use]
#[derive(Debug)]
pub struct LogGuard {
    pub from: StaticStr,
    pub work: StaticStr,
}
assert_impl_all!(LogGuard: Send, Sync);

impl LogGuard {
    pub fn new(from: impl Into<StaticStr>, work: impl Into<StaticStr>) -> Self {
        let (from, work) = (from.into(), work.into());
        info!(from = from.clone(), "start {work}.");
        Self { from, work }
    }
}

impl Drop for LogGuard {
    fn drop(&mut self) {
        let from = mem::take(&mut self.from);
        info!(from = from, "end {work}.", work = self.work);
    }
}



pub macro log($msg_type:ident, from = $from:expr, $($content:tt)*) {
    {
        use $crate::app::utils::logger::{log, MsgType::$msg_type};
        log($msg_type, $from, std::fmt::format(format_args!($($content)*)));
    }
}

pub macro info {
    (from = $from:expr, $($fmt:tt)*) => {
        $crate::logger::log!(Info, from = $from, $($fmt)*);
    },

    ($($fmt:tt)*) => { $crate::logger::info!(from = "*unknown*", $($fmt)*); },
}

pub macro error {
    (from = $from:expr, $($fmt:tt)*) => {
        $crate::logger::log!(Error, from = $from, $($fmt)*);
    },

    ($($fmt:tt)*) => { $crate::logger::error!(from = "*unknown*", $($fmt)*); },
}

pub macro log_dbg($expr:expr) {
    {
        use $crate::app::utils::logger::log;
        let result = $expr;
        info!(from = "dbg", "{expr} = {result:?}", expr = stringify!($expr));
        result
    }
}

pub macro scope(from = $from:expr, $($content:tt)*) {
    let _logger_scope_guard = $crate::app::utils::logger::scope(
        $from, std::fmt::format(format_args!($($content)*))
    );
}



// FIXME: 
// pub fn spawn_window(ui: &imgui::Ui) {
//     use {
//         crate::app::utils::{
//             graphics::ui::imgui_ext::make_window,
//         },
//         cpython::{Python, PyResult, py_fn, PyDict},
//     };
// 
//     const ERROR_COLOR: [f32; 4] = [0.8, 0.1, 0.05, 1.0];
//     const INFO_COLOR:  [f32; 4] = [1.0, 1.0, 1.0,  1.0];
// 
//     const PADDING: f32 = 10.0;
//     const HEIGHT:  f32 = 300.0;
// 
//     let [width, height] = ui.io().display_size;
// 
//     make_window(ui, "Log list")
//         .collapsed(true, imgui::Condition::Appearing)
//         .save_settings(false)
//         .collapsible(true)
//         .bg_alpha(0.8)
//         .position([PADDING, height - PADDING], imgui::Condition::Always)
//         .position_pivot([0.0, 1.0])
//         .size([width - 2.0 * PADDING, HEIGHT], imgui::Condition::Always)
//         .build(|| {
//             use crate::app::utils::terrain::chunk::commands::{Command, command};
// 
//             let messages = LOG_MESSAGES.lock();
// 
//             static INPUT: Mutex<String> = Mutex::new(String::new());
//             let mut input = INPUT.lock();
// 
//             let is_enter_pressed = ui.input_text("Console", &mut input)
//                 .enter_returns_true(true)
//                 .build();
// 
//             let buf = input.replace("^;", "\n");
// 
//             let gil = Python::acquire_gil();
//             let py = gil.python();
//         
//             let voxel_set = py_fn!(py, voxel_set(x: i32, y: i32, z: i32, new_id: u16) -> PyResult<i32> {
//                 command(Command::SetVoxel { pos: veci!(x, y, z), new_id });
//                 Ok(0)
//             });
// 
//             let voxel_fill = py_fn!(py, voxel_fill(
//                 sx: i32, sy: i32, sz: i32,
//                 ex: i32, ey: i32, ez: i32, new_id: u16
//             ) -> PyResult<i32> {
//                 command(Command::FillVoxels { pos_from: veci!(sx, sy, sz), pos_to: veci!(ex, ey, ez), new_id });
//                 Ok(0)
//             });
// 
//             let drop_all_meshes = py_fn!(py, drop_all_meshes() -> PyResult<i32> {
//                 command(Command::DropAllMeshes);
//                 Ok(0)
//             });
// 
//             let locals = PyDict::new(py);
// 
//             locals.set_item(py, "voxel_set", voxel_set)
//                 .unwrap_or_else(|err|
//                     log!(Error, from = "logger", "failed to set 'voxel_set' item: {err:?}")
//                 );
// 
//             locals.set_item(py, "voxel_fill", voxel_fill)
//                 .unwrap_or_else(|err|
//                     log!(Error, from = "logger", "failed to set 'voxel_fill' item: {err:?}")
//                 );
//                 
//             locals.set_item(py, "drop_all_meshes", drop_all_meshes)
//                 .unwrap_or_else(|err|
//                     log!(Error, from = "logger", "failed to set 'drop_all_meshes' item: {err:?}")
//                 );
// 
//             if is_enter_pressed {
//                 py.run(&buf, None, Some(&locals))
//                     .unwrap_or_else(|err| log!(Error, from = "logger", "{err:?}"));
//             }
// 
//             for msg in messages.iter().rev() {
//                 let color = match msg.msg_type {
//                     MsgType::Error => ERROR_COLOR,
//                     MsgType::Info  => INFO_COLOR,
//                 };
// 
//                 ui.text_colored(color, &format!("[LOG]: {msg}"));
//             }
//         });
// }



pub trait LogError<T> {
    fn log_error(self, from: impl Into<StaticStr>, msg: impl Into<StaticStr>) -> T where T: Default;
    fn log_error_or(self, from: impl Into<StaticStr>, msg: impl Into<StaticStr>, default: T) -> T;
    fn log_error_or_else(self, from: impl Into<StaticStr>, msg: impl Into<StaticStr>, f: impl FnOnce() -> T) -> T;
}

impl<T, E: std::fmt::Display> LogError<T> for Result<T, E> {
    fn log_error(self, from: impl Into<StaticStr>, msg: impl Into<StaticStr>) -> T where T: Default {
        match self {
            Ok(item) => item,
            Err(err) => {
                let msg: StaticStr = msg.into();
                log!(Error, from = from, "{msg}: {err}");
                default()
            }
        }
    }

    fn log_error_or(self, from: impl Into<StaticStr>, msg: impl Into<StaticStr>, default: T) -> T {
        match self {
            Ok(item) => item,
            Err(err) => {
                let msg = msg.into();
                log!(Error, from = from, "{msg}: {err}");
                default
            }
        }
    }

    fn log_error_or_else(self, from: impl Into<StaticStr>, msg: impl Into<StaticStr>, f: impl FnOnce() -> T) -> T {
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