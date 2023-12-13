#![allow(clippy::manual_strip, clippy::too_many_arguments)]

use crate::{
    prelude::*,
    concurrency::channel::Channel,
};



module_constructor! {
    use crate::graphics::ui::egui_util::push_window_builder_lock_free;

    env_logger::init();

    // * Safety
    // * 
    // * Safe, because it's going on in module
    // * constructor, so no one access the update list.
    unsafe {
        push_window_builder_lock_free(build_window);
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
        $crate::logger::log!(Error, from = $from, $($fmt)*)
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



pub mod python {
    use {
        super::*,
        cpython::{PyDict, Python, py_fn, PyResult, PyObject},
        crate::app::utils::terrain::chunk::commands::{Command, command},
    };

    lazy_static! {
        pub static ref LOCALS: PyDict = {
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

            let log = py_fn!(py, log(value: PyObject) -> PyResult<i32> {
                logger::info!("{}", value);
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

            locals.set_item(py, "log", log)
                .unwrap_or_else(|err|
                    log!(Error, from = "logger", "failed to log a message: {err:?}")
                );

            locals
        };
    }

    pub fn run(code: &str) {
        let gil = Python::acquire_gil();
        let py = gil.python();

        py.run(code, None, Some(&LOCALS))
            .unwrap_or_else(|err| logger::error!(from = "logger", "{err:?}"));
    }
}



pub fn build_window(ctx: &mut egui::Context) {
    const OFFSET: f32 = 10.0;

    egui::Window::new("Console")
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -OFFSET])
        .min_width(ctx.available_rect().x_range().span() - 2.0 * OFFSET)
        .show(ctx, |ui| {
            static INPUT_HISTORY: Mutex<Vec<String>> = const_default();
            let mut input_history = INPUT_HISTORY.lock();

            static INPUT: Mutex<String> = const_default();
            let mut input = INPUT.lock();

            macros::atomic_static! {
                static LAST_SEARCH_INDEX: usize = usize::MAX;
            }
            
            let text_edit_response = ui.add(
                egui::TextEdit::singleline(input.deref_mut())
                    .cursor_at_end(true)
                    .hint_text("Input a command here...")
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace)
            );

            if text_edit_response.lost_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let code = input.replace("^;", "\n");

                    python::run(&code);

                    LAST_SEARCH_INDEX.store(0, Release);
                }

                input_history.push(mem::take(input.deref_mut()));
            } else if !input_history.is_empty() && text_edit_response.has_focus() {
                let mut index = LAST_SEARCH_INDEX.load(Acquire);

                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    match index {
                        0 | usize::MAX => index = input_history.len() - 1,
                        _ => index -= 1,
                    }

                    match input_history.get(index) {
                        None => input.clear(),
                        Some(src) => input.clone_from(src),
                    }
                } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    if index < input_history.len() - 1 {
                        index += 1;
                        input.clone_from(&input_history[index]);
                    } else {
                        index = usize::MAX;
                        input.clear();
                    }
                }

                LAST_SEARCH_INDEX.store(index, Release);
            }

            for msg in LOG_MESSAGES.lock().iter().rev() {
                let color = match msg.msg_type {
                    MsgType::Error => egui::Color32::RED,
                    MsgType::Info  => egui::Color32::GRAY,
                };

                ui.label(egui::RichText::new(msg.to_string())
                    .color(color)
                    .monospace()
                );
            }
        });
}



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