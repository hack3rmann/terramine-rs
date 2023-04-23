#![allow(dead_code)]

use {
    crate::prelude::*,
    std::sync::Mutex,
    tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LoadingError {
    #[error("failed to refresh {0} loading with value {1:.1}% because that loading isn't already exist!")]
    RefreshFailed(String, f32),

    #[error("failed to add loading {0} because it's already added with value {1:.1}%!")]
    AddFailed(String, f32),

    #[error("failed to finish loading {0} because it is not in the list")]
    LoadingNotExist(String),

    #[error("failed to finish loading {0} because its value is {1}, which is not zero")]
    LoadingNotFinished(String, f32),
}

#[derive(Debug, Default)]
pub struct Loadings {
    pub list: HashMap<String, f32>,
}
assert_impl_all!(Loadings: Send, Sync);

impl Loadings {
    pub fn new() -> Self {
        Self { list: HashMap::new() }
    }

    pub fn add(&mut self, name: String) -> Result<(), LoadingError> {
        match self.list.insert(name.clone(), 0.0) {
            None => Ok(()),
            Some(dropped_value) => Err(
                LoadingError::AddFailed(name, dropped_value)
            ),
        }
    }

    pub fn refresh(&mut self, name: &str, new_val: f32) -> Result<(), LoadingError> {
        match self.list.get_mut(name) {
            None => Err(
                LoadingError::RefreshFailed(name.into(), new_val)
            ),

            Some(value) => {
                *value = new_val;
                Ok(
                    ()
                )
            },
        }
    }

    pub fn finish(&mut self, name: &str) -> Result<(), LoadingError> {
        match self.list.remove(name) {
            None =>
                Err(LoadingError::LoadingNotExist(name.into())),

            Some(value) if value != 1.0 =>
                Err(LoadingError::LoadingNotFinished(name.into(), value)),

            _ => Ok(())
        }
    }

    pub fn spawn_info_window(&self, ui: &imgui::Ui) {
        use crate::app::utils::graphics::ui::imgui_constructor::make_window;

        if self.list.is_empty() { return }

        make_window(ui, "Loadings").build(|| {
            for (name, &value) in self.list.iter() {
                imgui::ProgressBar::new(value)
                    .overlay_text(&format!("{name}: {percent:.1}%", percent = 100.0 * value))
                    .build(ui);
            }
        });
    }
}

pub fn spawn_info_window(ui: &imgui::Ui) {
    LOADINGS.lock()
        .unwrap()
        .loads
        .spawn_info_window(ui)
}

pub fn recv_all() -> Result<(), LoadingError> {
    LOADINGS.lock()
        .unwrap()
        .recv_all()
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command<'s> {
    Add(String),
    Refresh(&'s str, f32),
    Finish(&'s str),
}

#[derive(Debug)]
pub struct ChannelLoadings<'s> {
    pub rx: UnboundedReceiver<Command<'s>>,
    pub tx: UnboundedSender<Command<'s>>,
    pub loads: Loadings,
}
assert_impl_all!(ChannelLoadings: Send, Sync);

pub type CommandSender<'s> = UnboundedSender<Command<'s>>;

lazy_static! {
    pub static ref LOADINGS: Mutex<ChannelLoadings<'static>> = Mutex::new(ChannelLoadings::new());
}

pub fn make_sender() -> CommandSender<'static> {
    LOADINGS.lock()
        .unwrap()
        .tx
        .clone()
}

pub fn start_new(name: &'static str) -> LoadingGuard {
    let sender = make_sender();
    sender.send(Command::Add(name.to_owned()))
        .expect("failed to send add command to loading");

    LoadingGuard { name, sender }
}

impl Default for ChannelLoadings<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'s> ChannelLoadings<'s> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { rx, tx, loads: Loadings::new() }
    }

    pub fn recv_all(&mut self) -> Result<(), LoadingError> {
        while let Ok(command) = self.rx.try_recv() {
            match command {
                Command::Add(name) =>
                    self.loads.add(name)?,

                Command::Refresh(name, percent) =>
                    self.loads.refresh(name, percent)?,

                Command::Finish(name) =>
                    self.loads.finish(name)?,
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct LoadingGuard {
    name: &'static str,
    sender: UnboundedSender<Command<'static>>,
}
assert_impl_all!(LoadingGuard: Send, Sync);

impl Drop for LoadingGuard {
    fn drop(&mut self) {
        self.sender.send(Command::Finish(self.name))
            .expect("failed to send finish command to loading");
    }
}

impl LoadingGuard {
    pub fn refresh(&self, value: f32) {
        self.sender.send(Command::Refresh(self.name, value))
            .expect("failed to send refresh command to loading");
    }
}
