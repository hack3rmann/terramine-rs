#![allow(dead_code)]

use {
    crate::prelude::*,
    tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};



module_constructor! {
    use crate::graphics::ui::imgui_ext::push_window_builder;

    push_window_builder(spawn_info_window);
}



#[derive(Error, Debug, Clone, PartialEq)]
pub enum LoadingError {
    #[error("failed to refresh {0} loading with value {1:.1}% because that loading isn't already exist!")]
    RefreshFailed(StaticStr, f32),

    #[error("failed to add loading {0} because it's already added with value {1:.1}%!")]
    AddFailed(StaticStr, f32),

    #[error("failed to finish loading {0} because it is not in the list")]
    LoadingNotExist(StaticStr),

    #[error("failed to finish loading {0} because its value is {1}, which is not zero")]
    LoadingNotFinished(StaticStr, f32),
}



#[derive(Debug, Default)]
pub struct Loadings {
    pub list: HashMap<StaticStr, f32>,
}
assert_impl_all!(Loadings: Send, Sync);

impl Loadings {
    pub fn new() -> Self {
        Self { list: HashMap::new() }
    }

    pub fn add(&mut self, name: impl Into<StaticStr>) -> Result<(), LoadingError> {
        let name = name.into();

        match self.list.insert(name.clone(), 0.0) {
            None => Ok(()),
            Some(dropped_value) => Err(
                LoadingError::AddFailed(name, dropped_value)
            ),
        }
    }

    pub fn refresh(&mut self, name: &str, new_val: f32) -> Result<(), LoadingError> {
        match self.list.get_mut(name) {
            None => Err(LoadingError::RefreshFailed(name.to_owned().into(), new_val)),

            Some(value) => {
                *value = new_val;
                Ok(())
            },
        }
    }

    pub fn finish(&mut self, name: &str) -> Result<(), LoadingError> {
        match self.list.remove(name) {
            None =>
                Err(LoadingError::LoadingNotExist(StrView::from(name).to_static())),

            Some(value) if value != 1.0 =>
                Err(LoadingError::LoadingNotFinished(StrView::from(name).to_static(), value)),

            _ => Ok(())
        }
    }

    pub fn spawn_info_window(&self, ui: &imgui::Ui) {
        use crate::app::utils::graphics::ui::imgui_ext::make_window;

        ensure_or!(!self.list.is_empty(), return);

        make_window(ui, "Loadings").build(|| {
            for (name, &value) in self.list.iter() {
                imgui::ProgressBar::new(value)
                    .overlay_text(&format!("{name}: {percent:.1}%", percent = 100.0 * value))
                    .build(ui);
            }
        });
    }
}



#[derive(Debug, Clone, PartialEq)]
pub enum LoadingCommand {
    Add(StaticStr),
    Refresh(StaticStr, f32),
    Finish(StaticStr),
}
assert_impl_all!(LoadingCommand: Send, Sync);



#[derive(Debug)]
pub struct ChannelLoadings {
    pub rx: UnboundedReceiver<LoadingCommand>,
    pub tx: UnboundedSender<LoadingCommand>,
    pub loads: Loadings,
}
assert_impl_all!(ChannelLoadings: Send, Sync);

impl ChannelLoadings {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { rx, tx, loads: Loadings::new() }
    }

    pub fn recv_all(&mut self) -> Result<(), LoadingError> {
        while let Ok(command) = self.rx.try_recv() {
            match command {
                LoadingCommand::Add(name) =>
                    self.loads.add(name)?,

                LoadingCommand::Refresh(name, percent) =>
                    self.loads.refresh(&name, percent)?,

                LoadingCommand::Finish(name) =>
                    self.loads.finish(&name)?,
            }
        }

        Ok(())
    }
}

impl Default for ChannelLoadings {
    fn default() -> Self {
        Self::new()
    }
}



#[derive(Debug)]
pub struct LoadingGuard {
    name: StaticStr,
    sender: UnboundedSender<LoadingCommand>,
}
assert_impl_all!(LoadingGuard: Send, Sync);

impl Drop for LoadingGuard {
    fn drop(&mut self) {
        let name = mem::take(&mut self.name);
        self.sender.send(LoadingCommand::Finish(name))
            .expect("failed to send finish command to loading");
    }
}

impl LoadingGuard {
    pub fn refresh(&self, value: impl Into<LoadingValue>) {
        let value = value.into();
        self.sender.send(LoadingCommand::Refresh(self.name.clone(), value.0))
            .expect("failed to send refresh command to loading");
    }
}



#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Deref, From, Into)]
pub struct LoadingValue(f32);
assert_impl_all!(LoadingValue: Send, Sync);

impl From<Discrete> for LoadingValue {
    fn from(Discrete(current, size): Discrete) -> Self {
        match size {
            0 | 1 => Self(0.0),
            _ => Self(current as f32 / (size - 1) as f32)
        }
    }
}



#[derive(Clone, Copy, Default, PartialEq)]
pub struct Discrete(pub usize, pub usize);
assert_impl_all!(Discrete: Send, Sync);

impl Discrete {
    pub const fn new(current: usize, size: usize) -> Self {
        Self(current, size)
    }
}



lazy_static! {
    pub static ref LOADINGS: Mutex<ChannelLoadings> = Mutex::new(ChannelLoadings::new());
}



pub type CommandSender = UnboundedSender<LoadingCommand>;



pub fn spawn_info_window(ui: &imgui::Ui) {
    LOADINGS.lock()
        .loads
        .spawn_info_window(ui)
}

pub fn recv_all() -> Result<(), LoadingError> {
    LOADINGS.lock().recv_all()
}

pub fn update() {
    recv_all().log_error("loading", "failed to receive all loadings");
}

pub fn make_sender() -> CommandSender {
    LOADINGS.lock().tx.clone()
}

pub fn start_new(name: impl Into<StaticStr>) -> LoadingGuard {
    let name = name.into();

    let sender = make_sender();
    sender.send(LoadingCommand::Add(name.clone()))
        .expect("failed to send add command to loading");

    LoadingGuard { name, sender }
}
