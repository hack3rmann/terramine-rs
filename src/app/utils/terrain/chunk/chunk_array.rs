use {
    crate::app::utils::{
        werror::prelude::*,
        math::prelude::*,
        graphics::{
            camera::Camera,
            Graphics,
            shader::Shader,
            debug_visuals::*,
        },
        saves::*,
        reinterpreter::*,
        concurrency::{
            loading::Loading,
            promise::Promise,
        },
    },
    super::{
        MeshedChunk,
        MeshlessChunk,
        ChunkEnvironment as ChunkEnv,
        ChunkFill,
        Addition,
        ChunkDetails,
        Detailed,
        DetailedVertexVec,
        iterator::SpaceIter,
    },
    glium::{
        uniforms::Uniforms,
        DrawError,
        Frame,
        DrawParameters,
        Depth,
        DepthTest,
        BackfaceCullingMode,
    },
    std::sync::mpsc::Sender,
};

#[derive(Clone, Copy)]
enum SaveType {
    Width,
    Height,
    Depth,
    ChunkArray,
}

impl Into<Offset> for SaveType {
    fn into(self) -> Offset { self as Offset }
}

pub struct GeneratedChunkArray<'e>(MeshlessChunkArray, Vec<ChunkEnv<'e>>);

impl GeneratedChunkArray<'static> {
    pub fn generate_mesh(self, percentage_tx: Sender<Loading>) -> (MeshlessChunkArray, Vec<DetailedVertexVec>) {
        let GeneratedChunkArray(chunk_array, chunk_env) = self;
        let volume = chunk_array.width * chunk_array.height * chunk_array.depth;

        /* Create mesh for each chunk */
        let meshes: Vec<_> = chunk_array.chunks.iter()
            .zip(chunk_env.iter())
            .zip(1_usize..)
            .map(|((chunk, env), i)| {
                /* Get mesh */
                let result = chunk.to_triangles(env);

                /* Calculate percentage */
                percentage_tx.send(Loading::from_range("Mesh generation", i, 0..volume)).wunwrap();

                return result
            })
            .collect();

        (chunk_array, meshes)
    }
}

/// Represents self-controlling chunk array.
/// * Width is bigger if you go to x+ direction
/// * Height is bigger if you go to y+ direction
/// * Depth is bigger if you go to z+ direction
#[allow(dead_code)]
pub struct MeshlessChunkArray {
    /* Size */
    width:	usize,
    height:	usize,
    depth:	usize,

    /* Chunk array itself */
    chunks: Vec<MeshlessChunk>,
}

impl MeshlessChunkArray {
    fn save_chunks(
        file_name: &str, save_path: &str, width: usize, height: usize, depth: usize,
        chunks: &Vec<MeshlessChunk>, percentage_tx: &Sender<Loading>
    ) {
        let volume = width * height * depth;

        use SaveType::*;
        Save::new(file_name)
            .create(save_path)
            .write(&width,  Width)
            .write(&height, Height)
            .write(&depth,  Depth)
            .pointer_array(volume, ChunkArray, |i| {
                /* Write chunk */
                let result = if chunks[i].is_empty() {
                    /* Save only chunk position if it is empty */
                    let mut state = ChunkFill::Empty.reinterpret_as_bytes();
                    state.append(&mut chunks[i].pos.reinterpret_as_bytes());

                    state
                } else if chunks[i].is_filled() {
                    /* Save only chunk position and one id */
                    let id = chunks[i].fill_id().wunwrap();
                    let mut state = ChunkFill::All(id).reinterpret_as_bytes();
                    state.append(&mut chunks[i].pos.reinterpret_as_bytes());

                    state
                } else {
                    /* Save chunk fully */
                    let mut state = ChunkFill::Standart.reinterpret_as_bytes();
                    state.append(&mut chunks[i].reinterpret_as_bytes());

                    state
                };

                /* Calculate percentage */
                percentage_tx.send(Loading::from_range("Saving to file", i, 0..volume)).wunwrap();

                /* Return chunk */
                return result
            })
            .save().wunwrap();
    }

    fn generate_file(
        file_name: &str, save_path: &str, percentage_tx: &Sender<Loading>,
        chunks: &mut Vec<MeshlessChunk>, width: usize, height: usize, depth: usize
    ) {
        let volume = width * height * depth;

        /* Generate chunks */
        *chunks = Vec::with_capacity(volume);
        let size = Int3::new(width as i32, height as i32, depth as i32);
        for pos in SpaceIter::new(-size/2 .. size - size/2) {
            chunks.push(MeshlessChunk::new(pos));

            /* Calculating percentage */
            // TODO: Write Usize3::from(Int3) to handle this:
            let coords = {
                let res = pos + size / 2;
                let (x, y, z) = res.as_tuple();
                [x as usize, y as usize, z as usize]
            };
            let idx = sdex::get_index(&coords, &[width, height, depth]);
            percentage_tx.send(Loading::from_range("Chunk generation", idx, 0..volume)).wunwrap();
        }

        /* Save */
        Self::save_chunks(file_name, save_path, width, height, depth, &chunks, &percentage_tx);
    }

    fn load_chunks(file_name: &str, save_path: &str, width: usize, height: usize, depth: usize, percentage_tx: &Sender<Loading>) -> Vec<MeshlessChunk> {
        let volume = width * height * depth;
        let mut chunks = vec![];

        use SaveType::*;
        let save = Save::new(file_name).open(save_path);

        if !std::path::Path::new(save_path).exists() || (width, height, depth) != (save.read(Width), save.read(Height), save.read(Depth)) {
            Self::generate_file(file_name, save_path, percentage_tx, &mut chunks, width, height, depth);
            return chunks;
        }

        chunks = save.read_pointer_array(ChunkArray, |i, bytes| {
            let offset = ChunkFill::static_size();
            let chunk_fill = ChunkFill::reinterpret_from_bytes(&bytes[0..offset]);

            /* Read chunk from bytes */
            let result = match chunk_fill {
                ChunkFill::Empty => {
                    let pos = Int3::reinterpret_from_bytes(&bytes[offset..]);

                    MeshlessChunk::new_empty(pos)
                },

                ChunkFill::All(id) => {
                    let pos = Int3::reinterpret_from_bytes(&bytes[offset..]);
                    
                    MeshlessChunk::new_filled(pos, id)
                },

                ChunkFill::Standart => {
                    let mut chunk = MeshlessChunk::reinterpret_from_bytes(&bytes[offset..]);
                    chunk.additional_data = Addition::Know {
                        fill: Some(ChunkFill::Standart),
                        details: ChunkDetails::Full
                    };

                    chunk
                },
            };

            /* Calculate percent */
            percentage_tx.send(Loading::from_range("Reading from file", i, 0..volume)).wunwrap();

            return result;
        });

        return chunks
    }

    pub fn generate(width: usize, height: usize, depth: usize) -> (Promise<(MeshlessChunkArray, Vec<DetailedVertexVec>)>, Promise<Loading>) {
        /* Create channels */
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let (percentage_tx, percentage_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            /* Name of world file */
            let (save_path, file_name) = ("src/world", "world");

            /* Load chunks */
            let chunks = Self::load_chunks(file_name, save_path, width, height, depth, &percentage_tx);

            /* Make environments with references to chunk array */
            let env = Self::make_environment(&chunks, width, height, depth, Some(percentage_tx.clone()));

            /* Create generated data */
            let array = MeshlessChunkArray { width, height, depth, chunks };
            let result = GeneratedChunkArray(array, env).generate_mesh(percentage_tx);

            /* Send */
            result_tx.send(result).wunwrap();
        });

        /* Return reciever */
        return (Promise(result_rx), Promise(percentage_rx))
    }

    /// Creates environment for ChunkArray.
    fn make_environment<'v, 'c>(chunks: &'v Vec<MeshlessChunk>, width: usize, height: usize, depth: usize, percentage_tx: Option<Sender<Loading>>) -> Vec<ChunkEnv<'c>> {
        let volume = width * height * depth;
        let mut env = vec![ChunkEnv::none(); volume];

        // TODO: Write `Usize3` vector to handle this:
        for pos in SpaceIter::new(Int3::zero() .. Int3::new(width as i32, height as i32, depth as i32)) {
            /* Local index function */
            let index = |bias| {
                let (x, y, z) = (pos + bias).as_tuple();
                sdex::get_index(&[x as usize, y as usize, z as usize], &[width, height, depth])
            };

            /* Reference to current environment variable */
            let env = &mut env[index(Int3::zero())];

            /* For `back` side */
            if pos.x() + 1 < width as i32 {
                env.back	= Some(&chunks[index(Int3::new( 1,  0,  0))]);
            }

            /* For `front` side */
            if pos.x() - 1 >= 0 {
                env.front	= Some(&chunks[index(Int3::new(-1,  0,  0))]);
            }
        
            /* For `top` side */
            if pos.y() + 1 < height as i32 {
                env.top		= Some(&chunks[index(Int3::new( 0,  1,  0))]);
            }

            /* For `bottom` side */
            if pos.y() - 1 >= 0 {
                env.bottom	= Some(&chunks[index(Int3::new( 0, -1,  0))]);
            }

            /* For `right` side */
            if pos.z() + 1 < depth as i32 {
                env.right	= Some(&chunks[index(Int3::new( 0,  0,  1))]);
            }

            /* For `left` side */
            if pos.z() - 1 >= 0 {
                env.left	= Some(&chunks[index(Int3::new( 0,  0, -1))]);
            }

            /* Calculate percentage */
            if let Some(tx) = &percentage_tx {
                let i = index(Int3::zero());
                tx.send(Loading::from_range("Calculating environment", i, 0..volume)).wunwrap();
            }
        }

        return env;
    }

    /// Gives an iterator over chunks.
    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &MeshlessChunk> {
        self.chunks.iter()
    }

    /// Gives an iterator over chunks.
    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut MeshlessChunk> {
        self.chunks.iter_mut()
    }

    /// Upgrades meshless chunk array to meshed.
    pub fn upgrade<'g, 'dp>(self, graphics: &'g Graphics, triangles: Vec<DetailedVertexVec>) -> MeshedChunkArray<'dp> {
        let (width, height, depth) = (self.width, self.height, self.depth);
        let chunks: Vec<_> = self.into_iter()
            .zip(triangles.into_iter())
            .map(|(chunk, triangles)| {
                let triangles = match &triangles {
                    Detailed::Full(vec) => Detailed::Full(&vec[..]),
                    Detailed::Low(vec) => Detailed::Low(&vec[..]),
                };
                let chunk = chunk.triangles_upgrade(graphics, triangles);
                DebugVisualized::new_meshed_chunk(chunk, &graphics.display)
            })
            .collect();

        /* Chunk draw parameters */
        let draw_params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            backface_culling: BackfaceCullingMode::CullClockwise,
            .. Default::default()
        };
        
        /* Create shaders */
        let full_shader = Shader::new("full_detail", "full_detail", &graphics.display);
        let low_shader = Shader::new("low_detail", "low_detail", &graphics.display);

        MeshedChunkArray { width, height, depth, chunks, full_shader, low_shader, draw_params }
    }
}

impl IntoIterator for MeshlessChunkArray {
    type Item = MeshlessChunk;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.chunks.into_iter()
    }
}

pub struct MeshedChunkArray<'a> {
    pub width:  usize,
    pub height: usize,
    pub depth:  usize,

    pub chunks:      Vec<DebugVisualized<MeshedChunk>>,
    pub full_shader: Shader,
    pub low_shader:  Shader,
    pub draw_params: DrawParameters<'a>
}

impl<'a> MeshedChunkArray<'a> {
    /// Renders chunks.
    pub fn render<U: Uniforms>(&mut self, target: &mut Frame, uniforms: &U, camera: &Camera) -> Result<(), DrawError> {
        /* Iterating through array */
        for chunk in self.chunks.iter_mut() {
            chunk.render_meshed_chunks(target, &self.full_shader, &self.low_shader, uniforms, &self.draw_params, camera)?;
        }
        Ok(())
    }

    pub fn get_environment<'env>(&self, chunk_pos: Int3) -> ChunkEnv<'env> {
        let chunk_array_size = Int3::new(self.width as i32, self.height as i32, self.depth as i32);
        let shifted_pos = chunk_pos + chunk_array_size / 2;

        let side_to_idx = |side: Int3| -> usize {
            let (x, y, z) = (shifted_pos + side).as_tuple();
            sdex::get_index(&[x as usize, y as usize, z as usize], &[self.width, self.height, self.depth])
        };

        let mut env = ChunkEnv::none();

        /* For `back` side */
        if shifted_pos.x() + 1 < self.width as i32 {
            env.back	= Some(&self.chunks[side_to_idx(Int3::new( 1,  0,  0))].inner.inner);
        }

        /* For `front` side */
        if shifted_pos.x() - 1 >= 0 {
            env.front	= Some(&self.chunks[side_to_idx(Int3::new(-1,  0,  0))].inner.inner);
        }
    
        /* For `top` side */
        if shifted_pos.y() + 1 < self.height as i32 {
            env.top		= Some(&self.chunks[side_to_idx(Int3::new( 0,  1,  0))].inner.inner);
        }

        /* For `bottom` side */
        if shifted_pos.y() - 1 >= 0 {
            env.bottom	= Some(&self.chunks[side_to_idx(Int3::new( 0, -1,  0))].inner.inner);
        }

        /* For `right` side */
        if shifted_pos.z() + 1 < self.depth as i32 {
            env.right	= Some(&self.chunks[side_to_idx(Int3::new( 0,  0,  1))].inner.inner);
        }

        /* For `left` side */
        if shifted_pos.z() - 1 >= 0 {
            env.left	= Some(&self.chunks[side_to_idx(Int3::new( 0,  0, -1))].inner.inner);
        }

        return env
    }

    pub fn update_chunks_details(&mut self, display: &glium::Display, camera: &Camera) {
        let updates: Vec<_> = self.chunks.iter_mut()
            .map(|chunk| chunk.update_details_data(camera) )
            .collect();

        let envs: Vec<_> = self.chunks.iter()
            .map(|chunk| self.get_environment(chunk.inner.inner.pos))
            .collect();
            
        self.chunks.iter_mut()
            .zip(envs.into_iter())
            .zip(updates.into_iter())
            .for_each(|((chunk, env), is_same)|
                if is_same { chunk.refresh_mesh(display, &env) }
            );
    }
}