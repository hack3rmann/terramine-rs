use {
    crate::app::utils::{
        terrain::voxel::voxel_data::Id,
        concurrency::channel::Channel,
    },
    math_linear::prelude::*,
    lazy_static::lazy_static,
    std::sync::Mutex,
};

lazy_static! {
    pub(super) static ref COMMAND_CHANNEL: Mutex<Channel<Command>> = Mutex::new(Channel::default());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    SetVoxel {
        pos: Int3,
        new_id: Id,
    },

    FillVoxels {
        pos_from: Int3,
        pos_to: Int3,
        new_id: Id,
    },

    DropAllMeshes,
}

pub fn command(command: Command) {
    COMMAND_CHANNEL.lock()
        .unwrap()
        .sender
        .send(command)
        .expect("failed to send command");
}
