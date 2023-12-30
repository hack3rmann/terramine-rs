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
            sizes: USize3::ZERO,
            owned: AtomicBool::new(true),
            borrow_map: vec![],
            chunks: vec![],
        })
    }

    /// Constructs new [`ChunkArray`] with empty chunks
    pub fn new_empty(sizes: USize3) -> Self {
        Self::from(ChunkArrayBox {
            sizes,
            owned: AtomicBool::new(true),
            borrow_map: vec![ChunkBorrowInfo::new(); sizes.volume()],
            chunks: ChunkArray::chunk_pos_range(sizes)
                .map(Chunk::new_empty)
                .collect()
        })
    }

    fn allocate(array: ChunkArrayBox) -> NonNull<ChunkArrayBox> {
        Box::leak(Box::new(array)).into()
    }

    /// Computes start and end poses from chunk array sizes.
    pub fn pos_bounds(sizes: USize3) -> (Int3, Int3) {
        (
            Self::volume_index_to_chunk_pos(sizes, USize3::ZERO),
            Self::volume_index_to_chunk_pos(sizes, sizes),
        )
    }

    /// Gives iterator over chunk coordinates.
    pub fn chunk_pos_range(sizes: USize3) -> Range3d {
        let (start, end) = Self::pos_bounds(sizes);
        Range3d::from(start..end)
    }

    /// Convertes global voxel position to 3d-index of a chunk in the array.
    pub fn global_voxel_pos_to_volume_index(
        voxel_pos: Int3, chunk_array_sizes: USize3
    ) -> Option<USize3> {
        let chunk_pos = Chunk::global_to_local(voxel_pos);
        let local_voxel_pos = Chunk::global_to_local_pos(chunk_pos, voxel_pos);

        let chunk_coord_idx
            = Self::local_pos_to_volume_index(chunk_array_sizes, chunk_pos)?;

        let voxel_offset_by_chunk: USize3
            = Chunk::local_to_global(chunk_coord_idx.into()).into();

        Some(voxel_offset_by_chunk + USize3::from(local_voxel_pos))
    }

    /// Convertes 3d-index of a chunk in the array to chunk pos.
    pub fn volume_index_to_chunk_pos(sizes: USize3, coord_idx: USize3) -> Int3 {
        Int3::from(coord_idx) - Int3::from(sizes) / 2
    }

    /// Convertes chunk pos to 3d index.
    pub fn local_pos_to_volume_index(sizes: USize3, pos: Int3)
        -> Option<USize3>
    {
        let sizes = Int3::from(sizes);
        let shifted = pos + sizes / 2;

        (
            0 <= shifted.x && shifted.x < sizes.x &&
            0 <= shifted.y && shifted.y < sizes.y &&
            0 <= shifted.z && shifted.z < sizes.z
        ).then_some(shifted.into())
    }

    /// Convertes 3d index to an array index.
    pub fn volume_index_to_linear(sizes: USize3, coord_idx: USize3) -> usize {
        sdex::get_index(&coord_idx.as_array(), &sizes.as_array())
    }

    /// Convertes array index to 3d index.
    pub fn linear_index_to_volume(idx: usize, sizes: USize3) -> USize3 {
        iterator::linear_index_to_volume(idx, sizes)
    }

    /// Converts array index to chunk pos.
    pub fn index_to_pos(idx: usize, sizes: USize3) -> Int3 {
        let coord_idx = Self::linear_index_to_volume(idx, sizes);
        Self::volume_index_to_chunk_pos(sizes, coord_idx)
    }

    /// Convertes chunk position to its linear index
    pub fn chunk_pos_to_linear_index(sizes: USize3, pos: Int3) -> Option<usize> {
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
    pub(crate) sizes: USize3,
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
    pub fn chunk(&self, pos: Int3) -> Option<ChunkRef> {
        let index = ChunkArray::chunk_pos_to_linear_index(self.sizes, pos)?;

        self.borrow_map[index].borrow().then(move || unsafe {
            ChunkRef::new(NonNull::from(self), index)
        })
    }

    /// Uniquely borrows a chunk from chunk array
    pub fn chunk_mut(&self, pos: Int3) -> Option<ChunkMut> {
        let index = ChunkArray::chunk_pos_to_linear_index(self.sizes, pos)?;

        self.borrow_map[index].borrow_mut().then(move || unsafe {
            ChunkMut::new(NonNull::from(self), index)
        })
    }

    pub fn generate(&mut self) {
        for (chunk, borrow) in self.chunks.iter_mut().zip(self.borrow_map.iter()) {
            assert!(borrow.is_free(), "chunk should be free to be generated");

            _ = mem::replace(chunk, Chunk::new(chunk.pos.load(Relaxed), self.sizes));
        }
    }

    pub fn set_voxel(&self, pos: Int3, new_id: VoxelId) -> AnyResult<()> {
        let mut chunk = self.chunk_mut(pos)
            .with_context(|| format!("failed to get chunk by voxel position {pos} uniquely"))?;

        chunk.set_voxel(pos, new_id)
            .with_context(|| format!("failed to set voxel on {pos}"))?;

        Ok(())
    }

    pub fn voxel(&self, pos: Int3) -> AnyResult<Voxel> {
        use crate::terrain::chunk::ChunkOption::*;

        let chunk = self.chunk(pos)
            .with_context(|| format!("failed to get chunk by voxel position {pos} uniquely"))?;

        match chunk.get_voxel_global(pos) {
            OutsideChunk => unreachable!("voxel at position {pos} is already in that chunk"),
            Voxel(voxel) => Ok(voxel),
            Failed => Err(StrError::from("caught on failed chunk"))?,
        }
    }

    pub fn chunk_adj(&self, pos: Int3) -> ChunkAdj {
        Range3d::adj_iter(pos)
            .map(|pos| self.chunk(pos))
            .collect()
    }

    pub fn chunk_with_adj(&self, pos: Int3) -> (Option<ChunkRef>, ChunkAdj) {
        (self.chunk(pos), self.chunk_adj(pos))
    }

    pub fn size(&self) -> USize3 {
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
            sizes: USize3::ZERO,
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
            let gpu_image = GpuImage::new(GpuImageDescriptor {
                device,
                queue,
                image: &image,
                label: Some("chunk_array_textures".into()),
            });

            Ok(Self { albedo: gpu_image })
        }
    }

    impl AsBindGroup for ChunkArrayTextures {
        type Data = Self;

        fn label() -> Option<&'static str> {
            Some("chunk_array_textures")
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


    
    pub async fn try_make_pipeline(device: &Device, queue: &Queue) -> AnyResult<RenderPipeline> {
        use { tokio::fs, graphics::*, crate::terrain::chunk::mesh::HiResVertex };

        let (shader_src, _self_uniform) = tokio::join!(
            fs::read_to_string("src/shaders/chunks_full.wgsl"),
            ChunkArrayTextures::load(
                device, queue, "src/image/texture_atlas.png",
            ),
        );

        let shader_src = shader_src?;
        // let _self_uniform = self_uniform.ok()?;

        let common_layout = {
            let entries = CommonUniform::bind_group_layout_entries(device);
            CommonUniform::bind_group_layout(device, &entries)
        };

        let camera_layout = {
            let entries = CameraUniform::bind_group_layout_entries(device);
            CameraUniform::bind_group_layout(device, &entries)
        };

        let layout = {
            let entries = ChunkArrayTextures::bind_group_layout_entries(device);
            ChunkArrayTextures::bind_group_layout(device, &entries)
        };

        let shader = Shader::new(
            device, shader_src, vec![HiResVertex::BUFFER_LAYOUT],
        );

        let layout = PipelineLayout::new(device, &PipelineLayoutDescriptor {
            label: Some("chunk_array_pipeline_layout"),
            bind_group_layouts: &[&common_layout, &camera_layout, &layout],
            push_constant_ranges: &[],
        });

        Ok(RenderPipeline::new(device, RenderPipelineDescriptor {
            shader: &shader,
            color_states: &[
                Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }),
            ],
            primitive_state: PRIMITIVE_STATE,
            label: Some("chunk_array_render_pipeline".into()),
            layout: &layout,
        }))
    }

    pub async fn setup_pipeline(world: &mut World) -> AnyResult<()> {
        use crate::graphics::*;

        let (device, queue) = {
            let graphics = world.resource::<&Graphics>()?;
            (graphics.get_device(), graphics.get_queue())
        };

        let pipeline = try_make_pipeline(&device, &queue).await
            .context("failed to make pipeline")?;

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

        Ok(())
    }



    pub fn render(
        world: &World, encoder: &mut CommandEncoder,
        targets: &[TextureView], depth: Option<&TextureView>,
    ) -> AnyResult<()> {
        let mut query = world.query::<(&GpuMesh, &ChunkArrayPipeline)>();

        let pipeline_cache = world.resource::<&PipelineCache>()?;
        
        let mut pass = RenderPass::new(
            "chunk_array_render_pass", encoder, targets, depth,
        );

        for (_entity, (mesh, pipeline_bound)) in query.iter() {
            let pipeline = pipeline_cache.get(pipeline_bound)
                .context("failed to get find pipeline in cache")?;

            // FIXME: bind binds

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
        let array = ChunkArray::new_empty(USize3::new(2, 2, 2));
        let chunk_ref = array.chunk(Int3::ZERO).unwrap();
        let mut array = array;
        let chunk_ref2 = chunk_ref.clone();

        assert_eq!(
            2,
            array.borrow_map[ChunkArray::chunk_pos_to_linear_index(
                array.sizes, Int3::ZERO
            ).unwrap()].shared_count()
        );

        drop(array);

        eprintln!("chunk is empty: {}", chunk_ref.is_empty());
    }
}