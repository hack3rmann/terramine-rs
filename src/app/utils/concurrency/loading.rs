#![allow(dead_code)]

use {
    crate::app::utils::cfg,
    std::{
        collections::HashMap,
        sync::Mutex,
    },
    tokio::sync::mpsc::{self, Receiver, Sender},
    thiserror::Error,
    lazy_static::lazy_static,
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

#[derive(Debug)]
pub struct Loadings {
    pub list: HashMap<String, f32>,
}

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

            Some(value) => Ok(
                *value = new_val
            ),
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
        if self.list.is_empty() { return }

        ui.window("Loadings")
            .movable(true)
            .resizable(true)
            .build(|| {
                for (name, &value) in self.list.iter() {
                    imgui::ProgressBar::new(value)
                        .overlay_text(name)
                        .build(ui);
                }
            });
    }
}

pub fn spawn_info_window(ui: &imgui::Ui) {
    LOADINGS.lock()
        .expect("mutex should be not poisoned")
        .loads
        .spawn_info_window(ui)
}

pub fn recv_all() -> Result<(), LoadingError> {
    LOADINGS.lock()
        .expect("mutex should be not poisoned")
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
    pub rx: Receiver<Command<'s>>,
    pub tx: Sender<Command<'s>>,
    pub loads: Loadings,
}

pub type CommandSender<'s> = mpsc::Sender<Command<'s>>;

lazy_static! {
    pub static ref LOADINGS: Mutex<ChannelLoadings<'static>> = Mutex::new(ChannelLoadings::new());
}

pub const BUFFER_SIZE: usize = cfg::concurrency::loadings::BUFFER_SIZE;

pub fn make_sender() -> CommandSender<'static> {
    LOADINGS.lock()
        .expect("mutex should be not poisoned")
        .tx
        .clone()
}

impl<'s> ChannelLoadings<'s> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(BUFFER_SIZE);
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
