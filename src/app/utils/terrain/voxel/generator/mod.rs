pub mod noise;

use {
    crate::app::utils::terrain::chunk::{Chunk, chunk_array::{GENERATOR_SIZES, ChunkArray}},
    self::noise::Noise2d,
    math_linear::prelude::*,
    lazy_static::lazy_static,
    std::sync::{RwLock, atomic::{Ordering, AtomicUsize, AtomicU32}},
    portable_atomic::AtomicF32,
};

static FREQUENCY: AtomicF32 = AtomicF32::new(1.0);
static N_OCTAVES: AtomicUsize = AtomicUsize::new(6);
static PERSISTENCE: AtomicF32 = AtomicF32::new(1.0);
static LACUNARITY: AtomicF32 = AtomicF32::new(0.5);
static SEED: AtomicU32 = AtomicU32::new(0);

lazy_static! {
    static ref NOISE_VALS: RwLock<Noise2d> = RwLock::new(
        Noise2d::new(
            SEED.load(Ordering::Relaxed),
            (Chunk::SIZES * USize3::from(*GENERATOR_SIZES.lock().unwrap())).xz(),
            FREQUENCY.load(Ordering::Relaxed),
            LACUNARITY.load(Ordering::Relaxed),
            N_OCTAVES.load(Ordering::Relaxed),
            PERSISTENCE.load(Ordering::Relaxed),
        )
    );
}

pub fn spawn_control_window(ui: &imgui::Ui) {
    use crate::app::utils::graphics::ui::imgui_constructor::make_window;

    make_window(ui, "Generator settings").build(|| {
        let _ = FREQUENCY.fetch_update(Ordering::AcqRel, Ordering::Relaxed, |mut freq| {
            ui.input_float("Frequency", &mut freq).build().then(|| freq)
        });

        let _ = N_OCTAVES.fetch_update(Ordering::AcqRel, Ordering::Relaxed, |mut n_oct| {
            ui.input_scalar("Octaves", &mut n_oct).build().then(|| n_oct)
        });

        let _ = PERSISTENCE.fetch_update(Ordering::AcqRel, Ordering::Relaxed, |mut pers| {
            ui.input_scalar("Persistence", &mut pers).build().then(|| pers)
        });

        let _ = LACUNARITY.fetch_update(Ordering::AcqRel, Ordering::Relaxed, |mut lac| {
            ui.input_scalar("Lacunarity", &mut lac).build().then(|| lac)
        });

        let _ = SEED.fetch_update(Ordering::AcqRel, Ordering::Relaxed, |mut seed| {
            ui.input_scalar("Seed", &mut seed).build().then(|| seed)
        });

        if ui.button("Build") {
            let mut noise_vals = NOISE_VALS.write().unwrap();
            let _ = std::mem::replace(&mut *noise_vals, Noise2d::new(
                SEED.load(Ordering::Relaxed),
                (Chunk::SIZES * USize3::from(*GENERATOR_SIZES.lock().unwrap())).xz(),
                FREQUENCY.load(Ordering::Relaxed),
                LACUNARITY.load(Ordering::Relaxed),
                N_OCTAVES.load(Ordering::Relaxed),
                PERSISTENCE.load(Ordering::Relaxed),
            ));
        }
    });
}

pub fn perlin(pos: Int3, chunk_array_sizes: USize3) -> i32 {
    let coord_idx = ChunkArray::voxel_pos_to_coord_idx(
        pos,
        chunk_array_sizes,
    ).expect("failed to convert voxel pos to coord idx");

    NOISE_VALS.read().unwrap()
        .map
        .get_value(coord_idx.x, coord_idx.z)
        .round() as i32
}