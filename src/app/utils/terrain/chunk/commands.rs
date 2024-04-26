use crate::{
    prelude::*,
    terrain::voxel::voxel_data::VoxelId,
    concurrency::channel::Channel,
};

lazy_static! {
    pub(super) static ref COMMAND_CHANNEL: Mutex<Channel<Command>> = Mutex::new(Channel::default());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    SetVoxel {
        pos: IVec3,
        new_id: VoxelId,
    },

    FillVoxels {
        pos_from: IVec3,
        pos_to: IVec3,
        new_id: VoxelId,
    },

    DropAllMeshes,

    GenerateNew {
        sizes: U16Vec3,
    },
}

pub fn command(command: Command) {
    COMMAND_CHANNEL.lock()
        .sender
        .send(command)
        .expect("failed to send command");
}
