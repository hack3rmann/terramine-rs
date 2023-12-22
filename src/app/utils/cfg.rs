//! The place where all significant constants are placed.

#![allow(dead_code)]

pub mod save {
    pub const META_FILE_NAME: &str = "meta.off";
    pub const STACK_FILE_EXTENSION: &str = "stk";
    pub const HEAP_FILE_EXTENSION:  &str = "hp";
}

pub mod camera {
    pub const FRUSTUM_EDGE_LINE_LENGTH: f32 = 10_000.0;
    pub const VERTICAL_LOOK_EPS: f64 = 0.001;
    pub const LIGHT_NEAR_PLANE: f32 = 1.0;
    pub const LIGHT_FAR_PLANE:  f32 = 200.0;
    pub const IS_CAMERA_LOOK_ACCELERATION_ENABLED: bool = true;
    pub const MOUSE_SENSETIVITY: f32 = 18000.0 / N_SIMULATIONS_STEPS as f32;
    pub const N_SIMULATIONS_STEPS: usize = 20;

    pub mod default {
        /// These constants are shared with shader file. See `postprocessing.frag`.
        pub const NEAR_PLANE:     f32 = 0.5;
        pub const FAR_PLANE:      f32 = 10_000.0;

        pub const SPEED:	      f32 = 10.0;
        pub const SPEED_FALLOFF:  f32 = 0.995;
        pub const FOV_IN_DEGREES: f32 = 60.0;
        pub const MOUSE_SENSETIVITY: f32 = 0.2;
    }
}

pub mod window {
    pub const fn aspect_ratio(width: f32, height: f32) -> f32 {
        height / width
    }

    pub mod default {
        use math_linear::prelude::*;
        
        pub const WIDTH:  usize = 1024;
        pub const HEIGHT: usize = 768;
        pub const SIZES: USize2 = vecs!(WIDTH, HEIGHT);
        pub const ASPECT_RATIO: f32 = super::aspect_ratio(WIDTH as f32, HEIGHT as f32);
    }
}

pub mod topology {
    pub const Z_FIGHTING_BIAS: f32 = 0.001;
}

pub mod terrain {
    /// Chunk side length in voxels.
    /// Must be a power of 2 due to be halfed in process of lowering details.
    pub const CHUNK_SIZE: usize = 64;
    pub const VOXEL_SIZE: f32   = 1.0;

    pub const BACK_NORMAL:   (f32, f32, f32) = ( 1.0,  0.0,  0.0 );
    pub const FRONT_NORMAL:  (f32, f32, f32) = (-1.0,  0.0,  0.0 );
    pub const TOP_NORMAL:    (f32, f32, f32) = ( 0.0,  1.0,  0.0 );
    pub const BOTTOM_NORMAL: (f32, f32, f32) = ( 0.0, -1.0,  0.0 );
    pub const RIGHT_NORMAL:  (f32, f32, f32) = ( 0.0,  0.0,  1.0 );
    pub const LEFT_NORMAL:   (f32, f32, f32) = ( 0.0,  0.0, -1.0 );

    pub const BACK_TANGENT:   (f32, f32, f32) = ( 0.0,  1.0,  0.0 );
    pub const FRONT_TANGENT:  (f32, f32, f32) = BACK_TANGENT;
    pub const TOP_TANGENT:    (f32, f32, f32) = (-1.0,  0.0,  0.0 );
    pub const BOTTOM_TANGENT: (f32, f32, f32) = TOP_TANGENT;
    pub const RIGHT_TANGENT:  (f32, f32, f32) = BACK_TANGENT;
    pub const LEFT_TANGENT:   (f32, f32, f32) = BACK_TANGENT;

    pub const BACK_IDX:   usize = 0;
    pub const FRONT_IDX:  usize = 1;
    pub const TOP_IDX:    usize = 2;
    pub const BOTTOM_IDX: usize = 3;
    pub const RIGHT_IDX:  usize = 4;
    pub const LEFT_IDX:   usize = 5;

    pub const MAX_TASKS: usize = 10_000;
    pub const MAX_CHUNKS: usize = 100_000;

    pub mod voxel_types {
        use {
            crate::app::utils::terrain::voxel::voxel_data::{VoxelData, TextureSides},
            math_linear::prelude::Color,
        };

        pub const VOXEL_DATA: &[VoxelData] = &[
            VoxelData::new("Air",    0, Color::new(0.00, 0.00, 0.00), TextureSides::all(0)),
            VoxelData::new("Log",    1, Color::new(0.62, 0.52, 0.30), TextureSides::vertical(3, 1, 1)),
            VoxelData::new("Stone",  2, Color::new(0.45, 0.45, 0.45), TextureSides::all(2)),
            VoxelData::new("Grass",  3, Color::new(0.40, 0.64, 0.24), TextureSides::vertical(4, 6, 5)),
            VoxelData::new("Dirt",   4, Color::new(0.59, 0.42, 0.29), TextureSides::all(5)),
        ];
    }

    pub mod default {
        use math_linear::prelude::Int3;
        
        pub const WORLD_SIZES_IN_CHUNKS: Int3 = veci!(7, 1, 7);
        pub const LOD_THREASHOLD: f32 = 5.8;
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn chunk_size_is_power_of_two() {
            assert!(super::CHUNK_SIZE.is_power_of_two());
        }
    }
}

pub mod texture {
    pub const DIRECTORY: &str = "src/image/";

    pub mod atlas {
        pub const ITEM_SIZE_IN_PIXELS:    usize = 8;
        pub const ITEM_PADDING_IN_PIXELS: usize = 4;
        pub const ITEMS_COUNT_IN_ROW:     usize = 32;
        pub const BIAS:                   f32   = 0.0;
    }
}

pub mod shader {
    pub const CLEAR_DEPTH:   f32 = 1.0;
    pub const CLEAR_STENCIL: i32 = 0;
    
    pub const DIRECTORY: &str = "src/shaders/";
    
    pub const VERTEX_FILE_EXTENTION:   &str = "vert";
    pub const FRAGMENT_FILE_EXTENTION: &str = "frag";

    pub const WGSL_VERTEX_ENTRY: &str = "vs_main";
    pub const WGSL_FRAGMENT_ENTRY: &str = "fs_main";
    
    /// That constant is shared with shader. See `postprocessing.frag`.
    pub const CLEAR_COLOR: wgpu::Color = wgpu::Color { r: 0.02, g: 0.02, b: 0.02, a: 1.0 };
}

pub mod key_bindings {
    use crate::app::utils::user_io::Key;

    pub const DEBUG_VISUALS_SWITCH:           Key = Key::F3;
    pub const APP_EXIT:                       Key = Key::Escape;
    pub const MOUSE_CAPTURE:                  Key = Key::T;
    pub const ENABLE_DRAG_AND_RESIZE_WINDOWS: Key = Key::I;
    pub const ENABLE_PROFILER_WINDOW:         Key = Key::E;
    pub const SWITCH_RENDER_SHADOWS:          Key = Key::U;
    pub const RELOAD_RESOURCES:               Key = Key::H;
    pub const SPAWN_CAMERA:                   Key = Key::C;
}

pub mod timer {
    pub const N_FAMES_TO_MEASURE: usize = 16;
}