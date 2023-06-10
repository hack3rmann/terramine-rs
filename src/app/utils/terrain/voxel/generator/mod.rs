pub mod noise;

use {
    crate::{
        prelude::*,
        terrain::chunk::{Chunk, chunk_array::{GENERATOR_SIZES, ChunkArray}},
    },
    self::noise::Noise2d,
    spin::RwLock,
};



module_constructor! {
    use crate::graphics::ui::imgui_ext::push_window_builder_lock_free;

    // * Safety
    // * 
    // * Safe, because it's going on in module
    // * constructor, so no one access the update list.
    unsafe { push_window_builder_lock_free(spawn_control_window) };
}



macros::atomic_static! {
    static FREQUENCY: f32 = 0.05;
    static N_OCTAVES: usize = 6;
    static PERSISTENCE: f32 = 3.0;
    static LACUNARITY: f32 = 0.5;
    static SEED: u32 = 10;
}

lazy_static! {
    static ref NOISE_VALS: RwLock<Noise2d> = RwLock::new(
        Noise2d::new(
            SEED.load(Relaxed),
            (Chunk::SIZES * USize3::from(*GENERATOR_SIZES.lock())).xz(),
            FREQUENCY.load(Relaxed),
            LACUNARITY.load(Relaxed),
            N_OCTAVES.load(Relaxed),
            PERSISTENCE.load(Relaxed),
        )
    );
}



pub fn spawn_control_window(ui: &imgui::Ui) {
    use crate::app::utils::graphics::ui::imgui_ext::make_window;

    make_window(ui, "Generator settings").build(|| {
        let _ = FREQUENCY.fetch_update(AcqRel, Relaxed, |mut freq| {
            ui.input_float("Frequency", &mut freq).build().then_some(freq)
        });

        let _ = N_OCTAVES.fetch_update(AcqRel, Relaxed, |mut n_oct| {
            ui.input_scalar("Octaves", &mut n_oct).build().then_some(n_oct)
        });

        let _ = PERSISTENCE.fetch_update(AcqRel, Relaxed, |mut pers| {
            ui.input_scalar("Persistence", &mut pers).build().then_some(pers)
        });

        let _ = LACUNARITY.fetch_update(AcqRel, Relaxed, |mut lac| {
            ui.input_scalar("Lacunarity", &mut lac).build().then_some(lac)
        });

        let _ = SEED.fetch_update(AcqRel, Relaxed, |mut seed| {
            ui.input_scalar("Seed", &mut seed).build().then_some(seed)
        });

        if ui.button("Build") {
            let mut noise_vals = NOISE_VALS.write();
            let _ = mem::replace(&mut *noise_vals, Noise2d::new(
                SEED.load(Relaxed),
                (Chunk::SIZES * USize3::from(*GENERATOR_SIZES.lock())).xz(),
                FREQUENCY.load(Relaxed),
                LACUNARITY.load(Relaxed),
                N_OCTAVES.load(Relaxed),
                PERSISTENCE.load(Relaxed),
            ));
        }
    });
}

pub fn perlin(pos: Int3, chunk_array_sizes: USize3) -> i32 {
    let coord_idx = ChunkArray::voxel_pos_to_coord_idx(
        pos,
        chunk_array_sizes,
    ).expect("failed to convert voxel pos to coord idx");

    NOISE_VALS.read()
        .map
        .get_value(coord_idx.x, coord_idx.z)
        .round() as i32
}