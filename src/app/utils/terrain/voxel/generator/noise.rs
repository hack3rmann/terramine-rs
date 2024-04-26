use noise::{Fbm, Perlin, utils::{PlaneMapBuilder, NoiseMapBuilder, NoiseMap}};
use glam::*;



pub struct Noise2d {
    pub map: NoiseMap,
}

impl Noise2d {
    pub fn new(seed: u32, width: usize, height: usize, frequency: f32, lacunarity: f32, n_octaves: usize, persistence: f32) -> Self {
        let fbm = {
            let mut fbm = Fbm::<Perlin>::new(seed);
            fbm.frequency = frequency as f64;
            fbm.lacunarity = lacunarity as f64;
            fbm.octaves = n_octaves;
            fbm.persistence = persistence as f64;
            fbm
        };
        
        let generator = PlaneMapBuilder::<_, 3>::new(&fbm)
            .set_size(width, height)
            .set_x_bounds(0.0, width as f64)
            .set_y_bounds(0.0, height as f64)
            .build();

        Self { map: generator }
    }
}