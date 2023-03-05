//! The place where all significant constants are placed.

pub mod save {
    pub const META_FILE_NAME: &str = "meta.off";
    pub const STACK_FILE_EXTENTION: &str = "stk";
    pub const HEAP_FILE_EXTENTION:  &str = "hp";
}

pub mod camera {
    pub const FRUSTUM_EDGE_LINE_LENGTH: f32 = 10_000.0;
    pub const VERTICAL_LOOK_EPS: f64 = 0.001;

    pub mod default {
        pub const NEAR_PLANE:     f32 = 0.5;
        pub const FAR_PLANE:      f32 = 10_000.0;
        pub const SPEED:	      f64 = 10.0;
        pub const SPEED_FALLOFF:  f32 = 0.88;
        pub const FOV_IN_DEGREES: f32 = 60.0;
    }
}

pub mod window {
    pub mod default {
        use math_linear::prelude::*;
        
        pub const WIDTH:  usize = 1024;
        pub const HEIGHT: usize = 768;
        pub const SIZES: USize2 = vecs!(WIDTH, HEIGHT);
    }
}

pub mod topology {
    pub const Z_FIGHTING_BIAS: f32 = 0.001;
}

pub mod terrain {
    /// Chunk side length in voxels.
    /// Must be a power of 2 due to be halfed in process of lowering details.
    pub const CHUNK_SIZE: usize = 64;
    pub const VOXEL_SIZE: f32   = 20.0;

    pub mod voxel_types {
        use {
            crate::app::utils::terrain::voxel::voxel_data::{VoxelData, TextureSides},
            math_linear::prelude::Color,
        };

        pub const VOXEL_DATA: [VoxelData; 3] = [
            VoxelData { name: "Air",    id: 0, avarage_color: Color::new(0.00, 0.00, 0.00), textures: TextureSides::all(0) },
            VoxelData { name: "Log",    id: 1, avarage_color: Color::new(0.15, 0.10, 0.05), textures: TextureSides::vertical(3, 1) },
            VoxelData { name: "Stone",  id: 2, avarage_color: Color::new(0.20, 0.20, 0.20), textures: TextureSides::all(2) },
        ];
    }

    pub mod default {
        use math_linear::prelude::Int3;
        pub const WORLD_SIZES_IN_CHUNKS: Int3 = veci!(7, 1, 7);
    }
}

pub mod texture {
    pub mod atlas {
        pub const ITEM_SIZE_IN_PIXELS:    usize = 8;
        pub const ITEM_PADDING_IN_PIXELS: usize = 4;
        pub const ITEMS_COUNT_IN_ROW:     usize = 32;
        pub const BIAS:                   f32   = 0.0;
    }
}

pub mod shader {
    pub const DIRECTORY: &str = "src/shaders/";
    pub const VERTEX_FILE_EXTENTION:   &str = "vert";
    pub const FRAGMENT_FILE_EXTENTION: &str = "frag";
    pub const CLEAR_COLOR: (f32, f32, f32, f32) = (0.08, 0.08, 0.08, 1.0);
    pub const CLEAR_DEPTH:   f32 = 1.0;
    pub const CLEAR_STENCIL: i32 = 0;

    pub mod voxel {
        pub mod light {
            pub const FRONT:  f32 = 0.9;
            pub const BACK:   f32 = 0.5;
            pub const TOP:    f32 = 1.0;
            pub const BOTTOM: f32 = 0.3;
            pub const LEFT:   f32 = 0.6;
            pub const RIGHT:  f32 = 0.7;
        }
    }
}

pub mod key_bindings {
    use glium::glutin::event::VirtualKeyCode as Key;

    pub const DEBUG_VISUALS_SWITCH:           Key = Key::F3;
    pub const APP_EXIT:                       Key = Key::Escape;
    pub const MOUSE_CAPTURE:                  Key = Key::T;
    pub const LOD_REFRESHER_SWITCH:           Key = Key::R;
    pub const ENABLE_DRAG_AND_RESIZE_WINDOWS: Key = Key::I;
    pub const ENABLE_PROFILER_WINDOW:         Key = Key::E;
}