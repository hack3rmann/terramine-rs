use {
    std::{
        collections::HashMap,
        sync::mpsc::{self, Receiver},
    },
    thiserror::Error,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Loading {
    pub state: &'static str,
    pub percent: f64
}

impl Loading {
    /// Creates new `Loading` struct.
    #[allow(dead_code)]
    pub const fn new(state: &'static str, percent: f64) -> Self {
        Self { state, percent }
    }

    /// Gives `None` Loading.
    pub const fn none() -> Self {
        Self { state: "None", percent: 0.0 }
    }

    /// Constructs `Loading` from value and a range.
    pub fn from_range(state: &'static str, value: usize, range: std::ops::Range<usize>) -> Self {
        Self { state, percent: (value + 1) as f64 / (range.end - range.start) as f64 }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LoadingError {
    #[error("failed to refresh {0} loading with value {1:.1}% because that loading isn't already exist!")]
    RefreshFailed(String, f32),

    #[error("failed to add loading {0} because it's already added with value {1:.1}%!")]
    AddFailed(String, f32),
}

#[derive(Debug)]
pub struct Loadings {
    list: HashMap<String, f32>,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command<'s> {
    Add(String),
    Refresh(&'s str, f32),
}

#[derive(Debug)]
pub struct ChannelLoadings<'s> {
    rec: Receiver<Command<'s>>,
    loads: Loadings,
}

impl<'s> ChannelLoadings<'s> {
    pub fn new() -> (Self, mpsc::Sender<Command<'s>>){
        let (tx, rx) = mpsc::channel();
        (Self { rec: rx, loads: Loadings::new() }, tx)
    }

    pub fn recv_all(&mut self) -> Result<(), LoadingError> {
        for command in self.rec.try_iter() {
            match command {
                Command::Add(name) =>
                    self.loads.add(name)?,

                Command::Refresh(name, percent) =>
                    self.loads.refresh(name, percent)?,
            }
        }

        Ok(())
    }
}
