use {
    crate::{
        prelude::*,
        iterator::Sides,
        terrain::{voxel::{voxel_data::VoxelId, Voxel}, chunk::Chunk},
    },
    std::ptr::NonNull,
};



assert_impl_all!(ChunkBorrowInfo: Send, Sync);
#[derive(Debug, Default)]
pub struct ChunkBorrowInfo {
    n_borrows: AtomicUsize,
}

impl ChunkBorrowInfo {
    pub const UNIQUE_BORROW: usize = usize::MAX;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_free(&self) -> bool {
        self.n_borrows.load(Acquire) == 0
    }

    pub fn is_unique(&self) -> bool {
        self.n_borrows.load(Acquire) == Self::UNIQUE_BORROW
    }

    pub fn is_shared(&self) -> bool {
        !self.is_unique() && self.n_borrows.load(Acquire) > 0
    }

    pub fn shared_count(&self) -> usize {
        if self.is_shared() {
            self.n_borrows.load(Acquire)
        } else { 0 }
    }

    pub fn borrow_mut(&self) -> bool {
        self.n_borrows.fetch_update(AcqRel, Relaxed, |value| {
            (value == 0).then_some(Self::UNIQUE_BORROW)
        }).is_ok()
    }

    pub fn borrow(&self) -> bool {
        self.n_borrows.fetch_update(AcqRel, Relaxed, |value| {
            (!self.is_unique()).then_some(value + 1)
        }).is_ok()
    }

    pub fn free(&self) -> bool {
        self.n_borrows.fetch_update(AcqRel, Relaxed, |value| {
            self.is_shared().then_some(value - 1)
        }).is_ok()
    }

    pub fn free_mut(&self) -> bool {
        self.n_borrows.fetch_update(AcqRel, Relaxed, |_| {
            self.is_unique().then_some(0)
        }).is_ok()
    }
}

impl Clone for ChunkBorrowInfo {
    fn clone(&self) -> Self {
        Self { n_borrows: AtomicUsize::new(self.n_borrows.load(Relaxed)) }
    }
}



assert_impl_all!(ChunkArray: Send, Sync, Component);
#[derive(Debug)]
pub struct ChunkArray {
    pub(crate) ptr: NonNull<ChunkArrayBox>,
}

unsafe impl Send for ChunkArray { }
unsafe impl Sync for ChunkArray { }

impl ChunkArray {
    /// Constructs new [`ChunkArray`].
    pub fn new() -> Self {
        Self::from(ChunkArrayBox {
            sizes: U16Vec3::ZERO,
            owned: AtomicBool::new(true),
            borrow_map: vec![],
            chunks: vec![],
        })
    }

    /// Constructs new [`ChunkArray`] with empty chunks
    pub fn new_empty(sizes: U16Vec3) -> Self {
        let volume = sizes.as_u64vec3().element_product() as usize;

        Self::from(ChunkArrayBox {
            sizes,
            owned: AtomicBool::new(true),
            borrow_map: vec![ChunkBorrowInfo::new(); volume],
            chunks: ChunkArray::chunk_pos_range(sizes)
                .map(Chunk::new_empty)
                .collect()
        })
    }

    fn allocate(array: ChunkArrayBox) -> NonNull<ChunkArrayBox> {
        Box::leak(Box::new(array)).into()
    }

    /// Computes start and end poses from chunk array sizes.
    pub fn pos_bounds(sizes: U16Vec3) -> (IVec3, IVec3) {
        (
            Self::volume_index_to_chunk_pos(sizes, U16Vec3::ZERO),
            Self::volume_index_to_chunk_pos(sizes, sizes),
        )
    }

    /// Gives iterator over chunk coordinates.
    pub fn chunk_pos_range(sizes: U16Vec3) -> Range3d {
        let (start, end) = Self::pos_bounds(sizes);
        Range3d::from(start..end)
    }

    /// Convertes global voxel position to 3d-index of a chunk in the array.
    pub fn global_voxel_pos_to_volume_index(
        voxel_pos: IVec3, chunk_array_sizes: U16Vec3,
    ) -> Option<U16Vec3> {
        let chunk_pos = Chunk::global_to_local(voxel_pos);
        let local_voxel_pos = Chunk::global_to_local_pos(chunk_pos, voxel_pos);

        let chunk_coord_idx
            = Self::local_pos_to_volume_index(chunk_array_sizes, chunk_pos)?;

        let voxel_offset_by_chunk
            = Chunk::local_to_global(chunk_coord_idx.as_ivec3());

        Some(voxel_offset_by_chunk.as_u16vec3() + local_voxel_pos.as_u16vec3())
    }

    /// Convertes 3d-index of a chunk in the array to chunk pos.
    pub fn volume_index_to_chunk_pos(sizes: U16Vec3, coord_idx: U16Vec3) -> IVec3 {
        coord_idx.as_ivec3() - sizes.as_ivec3() / 2
    }

    /// Convertes chunk pos to 3d index.
    pub fn local_pos_to_volume_index(sizes: U16Vec3, pos: IVec3)
        -> Option<U16Vec3>
    {
        let sizes = IVec3::from(sizes);
        let shifted = pos + sizes / 2;

        (
            0 <= shifted.x && shifted.x < sizes.x &&
            0 <= shifted.y && shifted.y < sizes.y &&
            0 <= shifted.z && shifted.z < sizes.z
        ).then_some(shifted.as_u16vec3())
    }

    /// Convertes 3d index to an array index.
    pub fn volume_index_to_linear(sizes: U16Vec3, coord_idx: U16Vec3) -> u64 {
        iterator::get_index(
            &coord_idx.to_array().map(|x| x as usize),
            &sizes.to_array().map(|x| x as usize),
        ) as u64
    }

    /// Convertes array index to 3d index.
    pub fn linear_index_to_volume(idx: u64, sizes: U16Vec3) -> U16Vec3 {
        iterator::linear_index_to_volume(idx, sizes.as_u64vec3()).as_u16vec3()
    }

    /// Converts array index to chunk pos.
    pub fn index_to_pos(idx: u64, sizes: U16Vec3) -> IVec3 {
        let coord_idx = Self::linear_index_to_volume(idx, sizes);
        Self::volume_index_to_chunk_pos(sizes, coord_idx)
    }

    /// Convertes chunk position to its linear index
    pub fn chunk_pos_to_linear_index(sizes: U16Vec3, pos: IVec3) -> Option<u64> {
        let coord_idx = Self::local_pos_to_volume_index(sizes, pos)?;
        Some(Self::volume_index_to_linear(sizes, coord_idx))
    }
}

impl From<ChunkArrayBox> for ChunkArray {
    fn from(value: ChunkArrayBox) -> Self {
        Self { ptr: Self::allocate(value) }
    }
}

impl Default for ChunkArray {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ChunkArray {
    fn drop(&mut self) {
        unsafe {
            self.ptr.as_mut().unown();
        }
    }
}

impl Deref for ChunkArray {
    type Target = ChunkArrayBox;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl DerefMut for ChunkArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}



#[derive(Debug)]
pub struct ChunkRef {
    pub(crate) parent: NonNull<ChunkArrayBox>,
    pub(crate) index: usize,
}

unsafe impl Send for ChunkRef { }

impl ChunkRef {
    /// Creates new shared reference to chunk
    /// 
    /// # Safety
    /// 
    /// Can only be called by [`ChunkArrayBox`]
    pub unsafe fn new(parent: NonNull<ChunkArrayBox>, index: usize) -> Self {
        Self { parent, index }
    }
}

impl Drop for ChunkRef {
    fn drop(&mut self) {
        let array = unsafe { self.parent.as_mut() };

        assert!(
            unsafe { array.chunk_shared_free(self.index) },
            "failed to release shared chunk reference",
        );

        unsafe { array.try_deallocate() };
    }
}

impl Deref for ChunkRef {
    type Target = Chunk;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let array = self.parent.as_ref();
            let chunk = array.get_chunk_unchecked(self.index)
                .unwrap_or_else(|| panic!("failed to find chunk at index {}", self.index));

            chunk.as_ref()
                .unwrap_or_else(|| panic!("there is no chunk at index {}", self.index))
        }
    }
}

impl Clone for ChunkRef {
    fn clone(&self) -> Self {
        unsafe {
            let array = self.parent.as_ref();
            array.borrow_map[self.index].borrow();
            
            Self::new(self.parent, self.index)
        }
    }
}



#[derive(Debug)]
pub struct ChunkMut {
    pub(crate) parent: NonNull<ChunkArrayBox>,
    pub(crate) index: usize,
}

unsafe impl Send for ChunkMut { }

impl ChunkMut {
    /// Creates new unique chunk reference
    /// 
    /// # Safety
    /// 
    /// Can only be called by [`ChunkArrayBox`]
    pub unsafe fn new(parent: NonNull<ChunkArrayBox>, index: usize) -> Self {
        Self { parent, index }
    }
}

impl Drop for ChunkMut {
    fn drop(&mut self) {
        let array = unsafe { self.parent.as_mut() };

        assert!(
            unsafe { array.chunk_mut_free(self.index) },
            "failed to free unique chunk reference",
        );

        unsafe { array.try_deallocate(); }
    }
}

impl Deref for ChunkMut {
    type Target = Chunk;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let array = self.parent.as_ref();
            let chunk = array.get_chunk_unchecked(self.index)
                .unwrap_or_else(|| panic!("failed to find chunk at index {}", self.index));

            chunk.as_ref()
                .unwrap_or_else(|| panic!("there is no chunk at index {}", self.index))
        }
    }
}

impl DerefMut for ChunkMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let array = self.parent.as_ref();
            let chunk = array.get_chunk_unchecked(self.index)
                .unwrap_or_else(|| panic!("failed to find chunk at index {}", self.index));

            chunk.as_mut()
                .unwrap_or_else(|| panic!("there is no chunk at index {}", self.index))
        }
    }
}



#[derive(Debug)]
pub struct ChunkArrayBox {
    pub(crate) sizes: U16Vec3,
    pub(crate) owned: AtomicBool,
    pub(crate) borrow_map: Vec<ChunkBorrowInfo>,
    pub(crate) chunks: Vec<Chunk>,
}
assert_impl_all!(ChunkArrayBox: Send, Sync);

impl ChunkArrayBox {
    /// Gives chunk pointer without checks
    /// 
    /// # Safety
    /// 
    /// - no one can borrow this chunk uniquely
    /// - pointer destroyed before [`ChunkArray`] dropped
    pub unsafe fn get_chunk_unchecked(&self, index: usize) -> Option<*mut Chunk> {
        Some(self.chunks.get(index)? as *const _ as *mut _)
    }

    /// Deallocates chunk array
    /// 
    /// # Safety
    /// 
    /// - there are no chunk references exist
    /// - chunk array box is box-allocated
    unsafe fn deallocate(this: *mut Self) {
        _ = unsafe { Box::from_raw(this) };
    }

    /// Unowns this array allocation so it evetually can be dropped
    /// 
    /// # Safety
    /// 
    /// Can only be called by [`ChunkArray`]
    unsafe fn unown(&mut self) {
        self.owned.store(false, Release);
        
        self.try_deallocate();
    }

    /// Tries to deallocate chunk array box. Returns success value
    /// 
    /// # Safety
    /// 
    /// Chunk array box should be box-allocated
    unsafe fn try_deallocate(&mut self) -> bool {
        if self.owned.load(Relaxed) {
            return false;
        }

        let is_free = self.borrow_map.iter().all(ChunkBorrowInfo::is_free);

        if is_free {
            Self::deallocate(self as *mut _);
        }

        is_free
    }

    /// Frees shared borrow of the chunk at index `index`. Returns success value
    /// 
    /// # Safety
    /// 
    /// Can only be called by [`ChunkRef`]
    unsafe fn chunk_shared_free(&self, index: usize) -> bool {
        self.borrow_map[index].free()
    }

    /// Frees unique borrow of the chunk at index `index`. Returns success value
    /// 
    /// # Safety
    /// 
    /// Can only be called by [`ChunkMut`]
    unsafe fn chunk_mut_free(&self, index: usize) -> bool {
        self.borrow_map[index].free_mut()
    }

    /// Borrows a chunk from chunk array
    pub fn chunk(&self, pos: IVec3) -> Option<ChunkRef> {
        let index = ChunkArray::chunk_pos_to_linear_index(self.sizes, pos)?;

        self.borrow_map[index as usize].borrow().then(move || unsafe {
            ChunkRef::new(NonNull::from(self), index as usize)
        })
    }

    /// Uniquely borrows a chunk from chunk array
    pub fn chunk_mut(&self, pos: IVec3) -> Option<ChunkMut> {
        let index = ChunkArray::chunk_pos_to_linear_index(self.sizes, pos)?;

        self.borrow_map[index as usize].borrow_mut().then(move || unsafe {
            ChunkMut::new(NonNull::from(self), index as usize)
        })
    }

    pub fn generate(&mut self) {
        for (chunk, borrow) in self.chunks.iter_mut().zip(self.borrow_map.iter()) {
            assert!(borrow.is_free(), "chunk should be free to be generated");

            _ = mem::replace(chunk, Chunk::new(chunk.pos.load(Relaxed), self.sizes));
        }
    }

    pub fn set_voxel(&self, pos: IVec3, new_id: VoxelId) -> AnyResult<()> {
        let mut chunk = self.chunk_mut(pos)
            .with_context(|| format!("failed to get chunk by voxel position {pos} uniquely"))?;

        chunk.set_voxel(pos, new_id)
            .with_context(|| format!("failed to set voxel on {pos}"))?;

        Ok(())
    }

    pub fn voxel(&self, pos: IVec3) -> AnyResult<Voxel> {
        use crate::terrain::chunk::ChunkOption::*;

        let chunk = self.chunk(pos)
            .with_context(|| format!("failed to get chunk by voxel position {pos} uniquely"))?;

        match chunk.get_voxel_global(pos) {
            OutsideChunk => unreachable!("voxel at position {pos} is already in that chunk"),
            Voxel(voxel) => Ok(voxel),
            Failed => Err(StrError::from("caught on failed chunk"))?,
        }
    }

    pub fn chunk_adj(&self, pos: IVec3) -> ChunkAdj {
        Range3d::adj_iter(pos)
            .map(|pos| self.chunk(pos))
            .collect()
    }

    pub fn chunk_with_adj(&self, pos: IVec3) -> (Option<ChunkRef>, ChunkAdj) {
        (self.chunk(pos), self.chunk_adj(pos))
    }

    pub fn size(&self) -> U16Vec3 {
        self.sizes
    }

    pub fn generate_all(&self) {
        for pos in ChunkArray::chunk_pos_range(self.size()) {
            let mut chunk = self.chunk_mut(pos)
                .expect("failed to generate borrowed chunk");

            _ = mem::replace(chunk.deref_mut(), Chunk::new(pos, self.size()));
        }
    }
}

impl Default for ChunkArrayBox {
    fn default() -> Self {
        Self {
            sizes: U16Vec3::ZERO,
            owned: AtomicBool::new(true),
            borrow_map: vec![],
            chunks: vec![],
        }
    }
}



pub mod render {
    use {
        super::*,
        crate::graphics::*,
    };

    const PRIMITIVE_STATE: PrimitiveState = PrimitiveState {
        cull_mode: Some(Face::Back),
        ..const_default()
    };



    #[derive(Debug, Clone)]
    pub struct ChunkArrayTextures {
        albedo: GpuImage,
    }

    impl ChunkArrayTextures {
        pub async fn load(
            device: &Device, queue: &Queue, path: impl AsRef<Path> + Send,
        ) -> AnyResult<Self> {
            let image = Image::from_file(path).await?;
            let gpu_image = GpuImage::new(device, queue, GpuImageDescriptor {
                image: &image,
                label: Some("chunk_array_textures"),
            });

            Ok(Self { albedo: gpu_image })
        }

        pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8])
            -> Result<Self, LoadImageError>
        {
            let image = Image::from_bytes(bytes)?;
            let gpu_image = GpuImage::new(device, queue, GpuImageDescriptor {
                image: &image,
                label: Some("chunk_array_textures"),
            });

            Ok(Self { albedo: gpu_image })
        }
    }

    impl AsBindGroup for ChunkArrayTextures {
        type Data = Self;

        fn label() -> Option<&'static str> {
            Some("chunk_array_textures")
        }

        fn cache_key() -> &'static str {
            "chunkArrayTextures"
        }

        fn update(
            &self, _: &Device, _: &Queue,
            bind_group: &mut PreparedBindGroup<Self::Data>,
        ) -> bool {
            bind_group.unprepared.data = self.clone();

            for (index, _resource) in bind_group.unprepared.bindings.iter() {
                match index {
                    0 => {
                        // FIXME:
                        logger::error!(
                            from = "chunk-array",
                            "failed to update texture view (not yet implemented)",
                        );
                    },
                    1 => {
                        // FIXME:
                        logger::error!(
                            from = "chunk-array",
                            "failed to update texture sampler (not yet implemented)",
                        )
                    },
                    _ => return false,
                }
            }

            true
        }

        fn bind_group_layout_entries(_: &Device) -> Vec<BindGroupLayoutEntry>
        where
            Self: Sized,
        {
            vec![
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ]
        }

        fn unprepared_bind_group(
            &self, _: &Device, _: &BindGroupLayout,
        ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
            Ok(UnpreparedBindGroup {
                bindings: vec![
                    (0, OwnedBindingResource::TextureView(self.albedo.view.clone())),
                    (1, OwnedBindingResource::Sampler(self.albedo.sampler.clone())),
                ],
                data: self.clone(),
            })
        }
    }



    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub struct ChunkArrayPipeline {
        id: PipelineId,
    }
    assert_impl_all!(ChunkArrayPipeline: Send, Sync, Component);

    impl PipelineBound for ChunkArrayPipeline {
        fn id(&self) -> PipelineId {
            self.id
        }

        fn from_pipeline(pipeline: &RenderPipeline) -> Self {
            Self { id: pipeline.id() }
        }
    }


    
    pub fn try_make_pipeline(
        device: Arc<Device>, queue: Arc<Queue>,
        cache: &mut BindsCache, loader: &mut AssetLoader,
    ) -> AnyResult<RenderPipeline> {
        use { graphics::*, crate::terrain::chunk::mesh::HiResVertex };

        const TEXTURE_ATLAS_PATH: &str = "src/image/texture_atlas.png";
        const SHADER_PATH: &str = "src/shaders/chunks_full.wgsl";

        let parse_textures = {
            let device = device.clone();
            let queue = queue.clone();

            move |bytes: Vec<u8>| {
                ChunkArrayTextures::from_bytes(&device, &queue, &bytes)
                    .map_err(AnyError::from)
            }
        };

        let parse_shader = {
            let device = device.clone();

            move |bytes| Ok(Shader::new(
                &device, String::from_utf8(bytes)?, vec![HiResVertex::BUFFER_LAYOUT],
            ))
        };

        if !loader.contains(TEXTURE_ATLAS_PATH) {
            loader.start_loading(TEXTURE_ATLAS_PATH, parse_textures);
        }

        if !loader.contains(SHADER_PATH) {
            loader.start_loading(SHADER_PATH, parse_shader);
        }

        if !loader.is_loaded(SHADER_PATH) || !loader.is_loaded(TEXTURE_ATLAS_PATH) {
            return Err(StrError::from("assets are not yet loaded").into());
        }

        let shader = loader.get::<Shader>(SHADER_PATH)
            .context("failed to get shader asset")?;

        let textures = loader.get::<ChunkArrayTextures>(TEXTURE_ATLAS_PATH)
            .context("failed to get texture atlas asset")?;

        let common_layout = {
            let entries = CommonUniform::bind_group_layout_entries(&device);
            CommonUniform::bind_group_layout(&device, &entries)
        };

        let camera_layout = {
            let entries = CameraUniform::bind_group_layout_entries(&device);
            CameraUniform::bind_group_layout(&device, &entries)
        };

        let layout = {
            let entries = ChunkArrayTextures::bind_group_layout_entries(&device);
            ChunkArrayTextures::bind_group_layout(&device, &entries)
        };

        let bind_group = textures.as_bind_group(&device, &layout)?;
        cache.add(bind_group);

        let layout = PipelineLayout::new(&device, &PipelineLayoutDescriptor {
            label: Some("chunk_array_pipeline_layout"),
            bind_group_layouts: &[&common_layout, &camera_layout, &layout],
            push_constant_ranges: &[],
        });

        Ok(RenderPipeline::new(&device, RenderPipelineDescriptor {
            shader,
            color_states: &[
                // TODO: setup output format as several GBuffer textures
                Some(ColorTargetState {
                    format: graphics::SURFACE_CFG.read().format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }),
            ],
            primitive_state: PRIMITIVE_STATE,
            label: Some("chunk_array".into()),
            layout: &layout,
        }))
    }

    pub fn try_setup_pipeline(world: &mut World) -> AnyResult<()> {
        use crate::graphics::*;

        static DONE: AtomicBool = AtomicBool::new(false);

        if DONE.load(Acquire) {
            return Ok(());
        }

        let (device, queue) = {
            let graphics = world.resource::<&Graphics>()?;
            (graphics.get_device(), graphics.get_queue())
        };

        let pipeline = {
            let mut cache = world.resource::<&mut BindsCache>()?;
            let mut loader = world.resource::<&mut AssetLoader>()?;

            try_make_pipeline(device, queue, &mut cache, &mut loader)
                .context("failed to make pipeline")?
        };

        let array_pipeline = ChunkArrayPipeline::from_pipeline(&pipeline);

        world.resource::<&mut PipelineCache>()?
            .insert(pipeline);

        let entities = world.query::<&ChunkArray>()
            .iter()
            .map(|(entity, _)| entity)
            .collect_vec();

        for entity in entities {
            world.insert_one(entity, array_pipeline)?;
        }

        DONE.store(true, Release);

        Ok(())
    }



    pub fn render(
        world: &World, encoder: &mut CommandEncoder,
        targets: &[TextureView], depth: Option<&TextureView>,
    ) -> AnyResult<()> {
        let mut query = world.query::<(&GpuMesh, &ChunkArrayPipeline)>();

        let pipeline_cache = world.resource::<&PipelineCache>()?;
        
        let binds_cache = world.resource::<&BindsCache>()?;

        let mut pass = RenderPass::new(
            "chunk_array_render_pass", encoder, targets, depth,
        );

        let common = binds_cache.get::<CommonUniform>()
            .context("failed to get common uniform bind")?;

        let camera = binds_cache.get::<CameraUniform>()
            .context("failed to get camera uniform bind")?;

        let Some(textures) = binds_cache.get::<ChunkArrayTextures>()
        else { return Ok(()) };

        for (_entity, (mesh, pipeline_bound)) in query.iter() {
            let pipeline = pipeline_cache.get(pipeline_bound)
                .context("failed to get find pipeline in cache")?;

            common.bind_group.bind(&mut pass, 0);
            camera.bind_group.bind(&mut pass, 1);
            textures.bind_group.bind(&mut pass, 2);
            // albedo.bind()
            // depth.bind()
            // normal.bind()
            // position.bind()

            mesh.render(pipeline, &mut pass);
        }

        Ok(())
    }
}




pub type ChunkAdj = Sides<Option<ChunkRef>>;



#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

    #[test]
    fn array_allocations() {
        let array = ChunkArray::new_empty(U16Vec3::new(2, 2, 2));
        let chunk_ref = array.chunk(IVec3::ZERO).unwrap();
        let mut array = array;
        let chunk_ref2 = chunk_ref.clone();

        assert_eq!(
            2,
            array.borrow_map[ChunkArray::chunk_pos_to_linear_index(
                array.sizes, IVec3::ZERO
            ).unwrap() as usize].shared_count()
        );

        drop(array);

        eprintln!("chunk is empty: {}", chunk_ref.is_empty());
    }
}