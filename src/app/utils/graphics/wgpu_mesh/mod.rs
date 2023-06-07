pub mod shape;

use {
    crate::{
        prelude::*, Hashed, graphics::render_resource::{*, Buffer},
    },
    std::collections::BTreeMap,
    wgpu::*,
};



pub use wgpu::PrimitiveTopology;



#[derive(Debug, Clone, SmartDefault)]
#[default(Self::new(default()))]
pub struct Mesh {
    primitive_topology: PrimitiveTopology,
    attributes: BTreeMap<MeshVertexAttributeId, VertexAttributeData>,
    indices: Option<Indices>,
}
assert_impl_all!(Mesh: Send, Sync, Default);

impl Mesh {
    /// Construct a new mesh. You need to provide a [`PrimitiveTopology`] so that the
    /// renderer knows how to treat the vertex data. Most of the time this will be
    /// [`PrimitiveTopology::TriangleList`].
    pub fn new(primitive_topology: PrimitiveTopology) -> Self {
        Self {
            primitive_topology,
            attributes: default(),
            indices: None,
        }
    }

    /// Returns the topology of the mesh.
    pub fn primitive_topology(&self) -> PrimitiveTopology {
        self.primitive_topology
    }

    /// Sets the data for a vertex attribute (position, normal etc.). The name will
    /// often be one of the associated constants such as [`MeshVertexAttribute::POSITION`].
    ///
    /// # Panic
    /// 
    /// Panics when the format of the values does not match the attribute's format.
    pub fn insert_attribute(
        &mut self,
        attribute: MeshVertexAttribute,
        values: impl Into<VertexAttributeValues>,
    ) {
        let values = values.into();
        let values_format = VertexFormat::from(&values);
        if values_format != attribute.format {
            panic!(
                "Failed to insert attribute. Invalid attribute format for {}.
                Given format is {values_format:?} but expected {:?}",
                attribute.name, attribute.format
            );
        }

        self.attributes
            .insert(attribute.id, VertexAttributeData { attribute, values });
    }

    /// Removes the data for a vertex attribute
    pub fn remove_attribute(
        &mut self, attribute: impl Into<MeshVertexAttributeId>
    ) -> Option<VertexAttributeValues> {
        self.attributes
            .remove(&attribute.into())
            .map(|data| data.values)
    }

    pub fn contains_attribute(&self, id: impl Into<MeshVertexAttributeId>) -> bool {
        self.attributes.contains_key(&id.into())
    }

    /// Retrieves the data currently set to the vertex attribute with the specified `name`.
    pub fn attribute(
        &self, id: impl Into<MeshVertexAttributeId>,
    ) -> Option<&VertexAttributeValues> {
        self.attributes.get(&id.into()).map(|data| &data.values)
    }

    /// Retrieves the data currently set to the vertex attribute with the specified `name` mutably.
    pub fn attribute_mut(
        &mut self, id: impl Into<MeshVertexAttributeId>,
    ) -> Option<&mut VertexAttributeValues> {
        self.attributes
            .get_mut(&id.into())
            .map(|data| &mut data.values)
    }

    /// Returns an iterator that yields references to the data of each vertex attribute.
    pub fn attributes(&self) -> impl Iterator<Item = (MeshVertexAttributeId, &VertexAttributeValues)> {
        self.attributes.iter().map(|(id, data)| (*id, &data.values))
    }

    /// Returns an iterator that yields mutable references to the data of each vertex attribute.
    pub fn attributes_mut(&mut self) -> impl Iterator<Item = (MeshVertexAttributeId, &mut VertexAttributeValues)> {
        self.attributes
            .iter_mut()
            .map(|(id, data)| (*id, &mut data.values))
    }

    /// Sets the vertex indices of the mesh. They describe how triangles are constructed out of the
    /// vertex attributes and are therefore only useful for the [`PrimitiveTopology`] variants
    /// that use triangles.
    pub fn set_indices(&mut self, indices: Option<Indices>) {
        self.indices = indices;
    }

    /// Retrieves the vertex `indices` of the mesh.
    pub fn indices(&self) -> Option<&Indices> {
        self.indices.as_ref()
    }

    /// Retrieves the vertex `indices` of the mesh mutably.
    pub fn indices_mut(&mut self) -> Option<&mut Indices> {
        self.indices.as_mut()
    }

    /// Computes and returns the index data of the mesh as bytes.
    /// This is used to transform the index data into a GPU friendly format.
    pub fn get_index_buffer_bytes(&self) -> Option<&[u8]> {
        self.indices.as_ref().map(|indices| match &indices {
            Indices::U16(indices) => bytemuck::cast_slice(&indices[..]),
            Indices::U32(indices) => bytemuck::cast_slice(&indices[..]),
        })
    }

    /// For a given `descriptor` returns a [`VertexBufferLayout`] compatible with this mesh. If this
    /// mesh is not compatible with the given `descriptor` (ex: it is missing vertex attributes), [`None`] will
    /// be returned.
    pub fn get_mesh_vertex_buffer_layout(&self) -> MeshVertexBufferLayout {
        let mut attributes = Vec::with_capacity(self.attributes.len());
        let mut attribute_ids = Vec::with_capacity(self.attributes.len());
        let mut accumulated_offset = 0;
        for (index, data) in self.attributes.values().enumerate() {
            attribute_ids.push(data.attribute.id);
            attributes.push(VertexAttribute {
                offset: accumulated_offset,
                format: data.attribute.format,
                shader_location: index as u32,
            });
            accumulated_offset += data.attribute.format.get_size();
        }

        MeshVertexBufferLayout::new(InnerMeshVertexBufferLayout {
            layout: OwnedVertexBufferLayout {
                array_stride: accumulated_offset,
                step_mode: VertexStepMode::Vertex,
                attributes,
            },
            attribute_ids,
        })
    }

    /// Counts all vertices of the mesh.
    ///
    /// # Panics
    /// 
    /// Panics if the attributes have different vertex counts.
    pub fn count_vertices(&self) -> usize {
        let mut vertex_count: Option<usize> = None;
        for (attribute_id, attribute_data) in &self.attributes {
            let attribute_len = attribute_data.values.len();
            if let Some(previous_vertex_count) = vertex_count {
                assert_eq!(previous_vertex_count, attribute_len,
                        "{attribute_id:?} has a different vertex count ({attribute_len}) than other attributes ({previous_vertex_count}) in this mesh.");
            }
            vertex_count = Some(attribute_len);
        }

        vertex_count.unwrap_or(0)
    }

    /// Computes and returns the vertex data of the mesh as bytes.
    /// Therefore the attributes are located in the order of their [`MeshVertexAttribute::id`].
    /// This is used to transform the vertex data into a GPU friendly format.
    ///
    /// # Panics
    /// 
    /// Panics if the attributes have different vertex counts.
    pub fn get_vertex_buffer_data(&self) -> Vec<u8> {
        let mut vertex_size = 0;
        for attribute_data in self.attributes.values() {
            let vertex_format = attribute_data.attribute.format;
            vertex_size += vertex_format.get_size() as usize;
        }

        let vertex_count = self.count_vertices();
        let mut attributes_interleaved_buffer = vec![0; vertex_count * vertex_size];
        // bundle into interleaved buffers
        let mut attribute_offset = 0;
        for attribute_data in self.attributes.values() {
            let attribute_size = attribute_data.attribute.format.get_size() as usize;
            let attributes_bytes = attribute_data.values.get_bytes();
            for (vertex_index, attribute_bytes) in
                attributes_bytes.chunks_exact(attribute_size).enumerate()
            {
                let offset = vertex_index * vertex_size + attribute_offset;
                attributes_interleaved_buffer[offset..offset + attribute_size]
                    .copy_from_slice(attribute_bytes);
            }

            attribute_offset += attribute_size;
        }

        attributes_interleaved_buffer
    }

    /// Duplicates the vertex attributes so that no vertices are shared.
    ///
    /// This can dramatically increase the vertex count, so make sure this is what you want.
    /// Does nothing if no [Indices] are set.
    pub fn duplicate_vertices(&mut self) {
        fn duplicate<T: Copy>(values: &[T], indices: impl Iterator<Item = usize>) -> Vec<T> {
            indices.map(|i| values[i]).collect()
        }
        
        let Some(indices) = self.indices.take() else { return };

        for attributes in self.attributes.values_mut() {
            use VertexAttributeValues::*;

            let indices = indices.iter();
            match &mut attributes.values {
                Float32(vec)   => *vec = duplicate(vec, indices),
                Sint32(vec)    => *vec = duplicate(vec, indices),
                Uint32(vec)    => *vec = duplicate(vec, indices),
                Float32x2(vec) => *vec = duplicate(vec, indices),
                Sint32x2(vec)  => *vec = duplicate(vec, indices),
                Uint32x2(vec)  => *vec = duplicate(vec, indices),
                Float32x3(vec) => *vec = duplicate(vec, indices),
                Sint32x3(vec)  => *vec = duplicate(vec, indices),
                Uint32x3(vec)  => *vec = duplicate(vec, indices),
                Sint32x4(vec)  => *vec = duplicate(vec, indices),
                Uint32x4(vec)  => *vec = duplicate(vec, indices),
                Float32x4(vec) => *vec = duplicate(vec, indices),
                Sint16x2(vec)  => *vec = duplicate(vec, indices),
                Snorm16x2(vec) => *vec = duplicate(vec, indices),
                Uint16x2(vec)  => *vec = duplicate(vec, indices),
                Unorm16x2(vec) => *vec = duplicate(vec, indices),
                Sint16x4(vec)  => *vec = duplicate(vec, indices),
                Snorm16x4(vec) => *vec = duplicate(vec, indices),
                Uint16x4(vec)  => *vec = duplicate(vec, indices),
                Unorm16x4(vec) => *vec = duplicate(vec, indices),
                Sint8x2(vec)   => *vec = duplicate(vec, indices),
                Snorm8x2(vec)  => *vec = duplicate(vec, indices),
                Uint8x2(vec)   => *vec = duplicate(vec, indices),
                Unorm8x2(vec)  => *vec = duplicate(vec, indices),
                Sint8x4(vec)   => *vec = duplicate(vec, indices),
                Snorm8x4(vec)  => *vec = duplicate(vec, indices),
                Uint8x4(vec)   => *vec = duplicate(vec, indices),
                Unorm8x4(vec)  => *vec = duplicate(vec, indices),
            }
        }
    }

    /// Calculates the [`MeshVertexAttribute::NORMAL`] of a mesh.
    ///
    /// # Panics
    /// 
    /// Panics if [`Indices`] are set or [`MeshVertexAttribute::POSITION`] is not of type `vec3` or
    /// if the mesh has any other topology than [`PrimitiveTopology::TriangleList`].
    /// Consider calling [`Mesh::duplicate_vertices`] or export your mesh with normal attributes.
    pub fn compute_flat_normals(&mut self) {
        assert!(
            self.indices().is_none(),
            "`compute_flat_normals` can't work on indexed geometry.
            Consider calling `Mesh::duplicate_vertices`."
        );

        assert!(
            matches!(self.primitive_topology, PrimitiveTopology::TriangleList),
            "`compute_flat_normals` can only work on `TriangleList`s"
        );

        let positions = self
            .attribute(MeshVertexAttribute::POSITION)
            .unwrap()
            .as_vec3()
            .expect("`MeshVertexAttribute::POSITION` vertex attributes should be of type `vec3`");

        let normals: Vec<_> = positions
            .chunks_exact(3)
            .map(|p| triangle_face_normal(p[0], p[1], p[2]))
            .flat_map(|normal| [normal; 3])
            .collect();

        self.insert_attribute(MeshVertexAttribute::NORMAL, VertexAttributeValues::Float32x3(normals));
    }

    /// Compute the Axis-Aligned Bounding Box of the mesh vertices in model space
    pub fn compute_aabb(&self) -> Option<Aabb> {
        use VertexAttributeValues::Float32x3;

        let Some(Float32x3(values)) = self.attribute(MeshVertexAttribute::POSITION)
        else { return None };

        let mut min = vec3::all(f32::MAX);
        let mut max = vec3::all(f32::MIN);
        for &pos in values {
            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            min.z = min.z.min(pos.z);

            max.x = max.x.max(pos.x);
            max.y = max.y.max(pos.y);
            max.z = max.z.max(pos.z);
        }

        Some(Aabb::from_float3(min, max))
    }
}



fn triangle_face_normal(a: vec3, b: vec3, c: vec3) -> vec3 {
    (b - a).cross(c - a).normalized()
}



#[derive(Debug, Clone)]
pub struct MeshVertexAttribute {
    pub name: &'static str,
    pub id: MeshVertexAttributeId,
    pub format: VertexFormat,
}
assert_impl_all!(MeshVertexAttribute: Send, Sync);

impl MeshVertexAttribute {
    pub const POSITION:   Self = Self::new("position",   0, VertexFormat::Float32x3);
    pub const NORMAL:     Self = Self::new("normal",     1, VertexFormat::Float32x3);
    pub const TANGENT:    Self = Self::new("tangent",    2, VertexFormat::Float32x3);
    pub const TEXTURE_UV: Self = Self::new("texture_uv", 3, VertexFormat::Float32x2);
    pub const COLOR:      Self = Self::new("color",      4, VertexFormat::Float32x4);

    pub const fn new(name: &'static str, id: usize, format: VertexFormat) -> Self {
        Self { name, id: MeshVertexAttributeId(id), format }
    }
}



#[derive(Clone, Copy, Default, Eq, PartialEq, Hash, PartialOrd, Ord, Deref, Debug)]
pub struct MeshVertexAttributeId(usize);
assert_impl_all!(MeshVertexAttributeId: Send, Sync);



pub trait VertexFormatSize {
    fn get_size(self) -> u64;
}

impl VertexFormatSize for VertexFormat {
    fn get_size(self) -> u64 {
        use VertexFormat::*;
        match self {
            Uint8x2 => 2,
            Uint8x4 => 4,
            Sint8x2 => 2,
            Sint8x4 => 4,
            Unorm8x2 => 2,
            Unorm8x4 => 4,
            Snorm8x2 => 2,
            Snorm8x4 => 4,
            Uint16x2 => 2 * 2,
            Uint16x4 => 2 * 4,
            Sint16x2 => 2 * 2,
            Sint16x4 => 2 * 4,
            Unorm16x2 => 2 * 2,
            Unorm16x4 => 2 * 4,
            Snorm16x2 => 2 * 2,
            Snorm16x4 => 2 * 4,
            Float16x2 => 2 * 2,
            Float16x4 => 2 * 4,
            Float32 => 4,
            Float32x2 => 4 * 2,
            Float32x3 => 4 * 3,
            Float32x4 => 4 * 4,
            Uint32 => 4,
            Uint32x2 => 4 * 2,
            Uint32x3 => 4 * 3,
            Uint32x4 => 4 * 4,
            Sint32 => 4,
            Sint32x2 => 4 * 2,
            Sint32x3 => 4 * 3,
            Sint32x4 => 4 * 4,
            Float64 => 8,
            Float64x2 => 8 * 2,
            Float64x3 => 8 * 3,
            Float64x4 => 8 * 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VertexAttributeData {
    pub attribute: MeshVertexAttribute,
    pub values: VertexAttributeValues,
}


/// Contains an array where each entry describes a property of a single vertex.
/// Matches the [`VertexFormat`]s.
#[derive(Clone, Debug)]
pub enum VertexAttributeValues {
    Float32(Vec<f32>),
    Sint32(Vec<i32>),
    Uint32(Vec<u32>),
    Float32x2(Vec<vec2>),
    Sint32x2(Vec<Int2>),
    Uint32x2(Vec<UInt2>),
    Float32x3(Vec<vec3>),
    Sint32x3(Vec<Int3>),
    Uint32x3(Vec<UInt3>),
    Float32x4(Vec<vec4>),
    Sint32x4(Vec<[i32; 4]>),
    Uint32x4(Vec<[u32; 4]>),
    Sint16x2(Vec<Short2>),
    Snorm16x2(Vec<Short2>),
    Uint16x2(Vec<UShort2>),
    Unorm16x2(Vec<UShort2>),
    Sint16x4(Vec<[i16; 4]>),
    Snorm16x4(Vec<[i16; 4]>),
    Uint16x4(Vec<[u16; 4]>),
    Unorm16x4(Vec<[u16; 4]>),
    Sint8x2(Vec<Byte2>),
    Snorm8x2(Vec<Byte2>),
    Uint8x2(Vec<UByte2>),
    Unorm8x2(Vec<UByte2>),
    Sint8x4(Vec<[i8; 4]>),
    Snorm8x4(Vec<[i8; 4]>),
    Uint8x4(Vec<[u8; 4]>),
    Unorm8x4(Vec<[u8; 4]>),
}

impl VertexAttributeValues {
    /// Returns the number of vertices in this [`VertexAttributeValues`]. For a single
    /// mesh, all of the [`VertexAttributeValues`] must have the same length.
    pub fn len(&self) -> usize {
        use VertexAttributeValues::*;
        match self {
            Float32(values)   => values.len(),
            Sint32(values)    => values.len(),
            Uint32(values)    => values.len(),
            Float32x2(values) => values.len(),
            Sint32x2(values)  => values.len(),
            Uint32x2(values)  => values.len(),
            Float32x3(values) => values.len(),
            Sint32x3(values)  => values.len(),
            Uint32x3(values)  => values.len(),
            Float32x4(values) => values.len(),
            Sint32x4(values)  => values.len(),
            Uint32x4(values)  => values.len(),
            Sint16x2(values)  => values.len(),
            Snorm16x2(values) => values.len(),
            Uint16x2(values)  => values.len(),
            Unorm16x2(values) => values.len(),
            Sint16x4(values)  => values.len(),
            Snorm16x4(values) => values.len(),
            Uint16x4(values)  => values.len(),
            Unorm16x4(values) => values.len(),
            Sint8x2(values)   => values.len(),
            Snorm8x2(values)  => values.len(),
            Uint8x2(values)   => values.len(),
            Unorm8x2(values)  => values.len(),
            Sint8x4(values)   => values.len(),
            Snorm8x4(values)  => values.len(),
            Uint8x4(values)   => values.len(),
            Unorm8x4(values)  => values.len(),
        }
    }

    pub fn as_vec3(&self) -> Option<&Vec<vec3>> {
        match self {
            VertexAttributeValues::Float32x3(values) => Some(values),
            _ => None,
        }
    }

    /// Returns `true` if there are no vertices in this [`VertexAttributeValues`].
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Flattens the [`VertexAttributeValues`] into a sequence of bytes. This is
    /// useful for serialization and sending to the GPU.
    pub fn get_bytes(&self) -> &[u8] {
        use VertexAttributeValues::*;
        match self {
            Float32(values)   => bytemuck::cast_slice(values),
            Sint32(values)    => bytemuck::cast_slice(values),
            Uint32(values)    => bytemuck::cast_slice(values),
            Float32x2(values) => bytemuck::cast_slice(values),
            Sint32x2(values)  => bytemuck::cast_slice(values),
            Uint32x2(values)  => bytemuck::cast_slice(values),
            Float32x3(values) => bytemuck::cast_slice(values),
            Sint32x3(values)  => bytemuck::cast_slice(values),
            Uint32x3(values)  => bytemuck::cast_slice(values),
            Float32x4(values) => bytemuck::cast_slice(values),
            Sint32x4(values)  => bytemuck::cast_slice(values),
            Uint32x4(values)  => bytemuck::cast_slice(values),
            Sint16x2(values)  => bytemuck::cast_slice(values),
            Snorm16x2(values) => bytemuck::cast_slice(values),
            Uint16x2(values)  => bytemuck::cast_slice(values),
            Unorm16x2(values) => bytemuck::cast_slice(values),
            Sint16x4(values)  => bytemuck::cast_slice(values),
            Snorm16x4(values) => bytemuck::cast_slice(values),
            Uint16x4(values)  => bytemuck::cast_slice(values),
            Unorm16x4(values) => bytemuck::cast_slice(values),
            Sint8x2(values)   => bytemuck::cast_slice(values),
            Snorm8x2(values)  => bytemuck::cast_slice(values),
            Uint8x2(values)   => bytemuck::cast_slice(values),
            Unorm8x2(values)  => bytemuck::cast_slice(values),
            Sint8x4(values)   => bytemuck::cast_slice(values),
            Snorm8x4(values)  => bytemuck::cast_slice(values),
            Uint8x4(values)   => bytemuck::cast_slice(values),
            Unorm8x4(values)  => bytemuck::cast_slice(values),
        }
    }
}


/// An array of indices into the [`VertexAttributeValues`] for a mesh.
///
/// It describes the order in which the vertex attributes should be joined into faces.
#[derive(Debug, Clone)]
pub enum Indices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl Indices {
    /// Returns an iterator over the indices.
    pub fn iter(&self) -> IndicesIter<'_> {
        match self {
            Indices::U16(vec) => IndicesIter::U16(vec.iter()),
            Indices::U32(vec) => IndicesIter::U32(vec.iter()),
        }
    }

    /// Returns the number of indices.
    pub fn len(&self) -> usize {
        match self {
            Indices::U16(vec) => vec.len(),
            Indices::U32(vec) => vec.len(),
        }
    }

    /// Returns `true` if there are no indices.
    pub fn is_empty(&self) -> bool {
        match self {
            Indices::U16(vec) => vec.is_empty(),
            Indices::U32(vec) => vec.is_empty(),
        }
    }
}

/// An Iterator for the [`Indices`].
pub enum IndicesIter<'a> {
    U16(std::slice::Iter<'a, u16>),
    U32(std::slice::Iter<'a, u32>),
}

impl Iterator for IndicesIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IndicesIter::U16(iter) => iter.next().map(|val| *val as usize),
            IndicesIter::U32(iter) => iter.next().map(|val| *val as usize),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            IndicesIter::U16(iter) => iter.size_hint(),
            IndicesIter::U32(iter) => iter.size_hint(),
        }
    }
}

impl ExactSizeIterator for IndicesIter<'_> { }
impl std::iter::FusedIterator for IndicesIter<'_> { }

impl From<&Indices> for IndexFormat {
    fn from(indices: &Indices) -> Self {
        match indices {
            Indices::U16(_) => IndexFormat::Uint16,
            Indices::U32(_) => IndexFormat::Uint32,
        }
    }
}

impl From<MeshVertexAttribute> for MeshVertexAttributeId {
    fn from(attribute: MeshVertexAttribute) -> Self {
        attribute.id
    }
}

pub type MeshVertexBufferLayout = Hashed<InnerMeshVertexBufferLayout>;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct InnerMeshVertexBufferLayout {
    attribute_ids: Vec<MeshVertexAttributeId>,
    layout: OwnedVertexBufferLayout,
}

impl InnerMeshVertexBufferLayout {
    pub fn contains(&self, attribute_id: impl Into<MeshVertexAttributeId>) -> bool {
        self.attribute_ids.contains(&attribute_id.into())
    }

    pub fn attribute_ids(&self) -> &[MeshVertexAttributeId] {
        &self.attribute_ids
    }

    pub fn layout(&self) -> VertexBufferLayout<'_> {
        self.layout.get_borrowed()
    }

    pub fn get_layout(
        &self, attribute_descriptors: &[VertexAttributeDescriptor],
    ) -> Result<OwnedVertexBufferLayout, MissingVertexAttributeError> {
        let mut attributes = Vec::with_capacity(attribute_descriptors.len());
        for attribute_descriptor in attribute_descriptors {
            let Some(index) = self.attribute_ids.iter()
                .position(|id| *id == attribute_descriptor.id)
            else {
                bail!(MissingVertexAttributeError {
                    id: attribute_descriptor.id,
                    name: attribute_descriptor.name,
                    pipeline_type: None,
                })
            };

            let layout_attribute = &self.layout.attributes[index];
            attributes.push(VertexAttribute {
                format: layout_attribute.format,
                offset: layout_attribute.offset,
                shader_location: attribute_descriptor.shader_location,
            });
        }

        Ok(OwnedVertexBufferLayout {
            array_stride: self.layout.array_stride,
            step_mode: self.layout.step_mode,
            attributes,
        })
    }
}

#[derive(Error, Debug)]
#[error("Mesh is missing requested attribute: {name} ({id:?}, pipeline type: {pipeline_type:?})")]
pub struct MissingVertexAttributeError {
    pub pipeline_type: Option<&'static str>,
    pub id: MeshVertexAttributeId,
    pub name: &'static str,
}

pub struct VertexAttributeDescriptor {
    pub shader_location: u32,
    pub id: MeshVertexAttributeId,
    pub name: &'static str,
}

impl VertexAttributeDescriptor {
    pub const fn new(shader_location: u32, id: MeshVertexAttributeId, name: &'static str) -> Self {
        Self { shader_location, id, name }
    }
}

impl From<&VertexAttributeValues> for VertexFormat {
    fn from(values: &VertexAttributeValues) -> Self {
        use VertexAttributeValues::*;
        match values {
            Float32(_)   => VertexFormat::Float32,
            Sint32(_)    => VertexFormat::Sint32,
            Uint32(_)    => VertexFormat::Uint32,
            Float32x2(_) => VertexFormat::Float32x2,
            Sint32x2(_)  => VertexFormat::Sint32x2,
            Uint32x2(_)  => VertexFormat::Uint32x2,
            Float32x3(_) => VertexFormat::Float32x3,
            Sint32x3(_)  => VertexFormat::Sint32x3,
            Uint32x3(_)  => VertexFormat::Uint32x3,
            Float32x4(_) => VertexFormat::Float32x4,
            Sint32x4(_)  => VertexFormat::Sint32x4,
            Uint32x4(_)  => VertexFormat::Uint32x4,
            Sint16x2(_)  => VertexFormat::Sint16x2,
            Snorm16x2(_) => VertexFormat::Snorm16x2,
            Uint16x2(_)  => VertexFormat::Uint16x2,
            Unorm16x2(_) => VertexFormat::Unorm16x2,
            Sint16x4(_)  => VertexFormat::Sint16x4,
            Snorm16x4(_) => VertexFormat::Snorm16x4,
            Uint16x4(_)  => VertexFormat::Uint16x4,
            Unorm16x4(_) => VertexFormat::Unorm16x4,
            Sint8x2(_)   => VertexFormat::Sint8x2,
            Snorm8x2(_)  => VertexFormat::Snorm8x2,
            Uint8x2(_)   => VertexFormat::Uint8x2,
            Unorm8x2(_)  => VertexFormat::Unorm8x2,
            Sint8x4(_)   => VertexFormat::Sint8x4,
            Snorm8x4(_)  => VertexFormat::Snorm8x4,
            Uint8x4(_)   => VertexFormat::Uint8x4,
            Unorm8x4(_)  => VertexFormat::Unorm8x4,
        }
    }
}

/// The GPU-representation of a [`Mesh`].
/// Consists of a vertex data buffer and an optional index data buffer.
#[derive(Debug)]
pub struct GpuMesh {
    /// Contains all attribute data for each vertex.
    pub vertex_buffer: Buffer,
    pub vertex_count: usize,
    pub buffer_info: GpuBufferInfo,
    pub primitive_topology: PrimitiveTopology,
    pub layout: MeshVertexBufferLayout,
}

/// The index/vertex buffer info of a [`GpuMesh`].
#[derive(Debug)]
pub enum GpuBufferInfo {
    /// Contains all index data of a mesh.
    Indexed {
        buffer: Buffer,
        count: usize,
        index_format: IndexFormat,
    },
    NonIndexed,
}

impl Mesh {
    // type ExtractedAsset = Mesh;
    // type PreparedAsset = GpuMesh;

    // /// Clones the mesh.
    // fn extract_asset(&self) -> Self::ExtractedAsset {
    //     self.clone()
    // }

    // Converts the extracted mesh a into [`GpuMesh`].
    // fn prepare_asset(
    //     mesh: Self::ExtractedAsset,
    //     device: &Device,
    // ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
    //     let vertex_buffer_data = mesh.get_vertex_buffer_data();
    //     let vertex_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
    //         usage: BufferUsages::VERTEX,
    //         label: Some("Mesh Vertex Buffer"),
    //         contents: &vertex_buffer_data,
    //     });

    //     let buffer_info = if let Some(data) = mesh.get_index_buffer_bytes() {
    //         GpuBufferInfo::Indexed {
    //             buffer: device.create_buffer_with_data(&BufferInitDescriptor {
    //                 usage: BufferUsages::INDEX,
    //                 contents: data,
    //                 label: Some("Mesh Index Buffer"),
    //             }),
    //             count: mesh.indices().unwrap().len(),
    //             index_format: mesh.indices().unwrap().into(),
    //         }
    //     } else {
    //         GpuBufferInfo::NonIndexed
    //     };

    //     let mesh_vertex_buffer_layout = mesh.get_mesh_vertex_buffer_layout();

    //     Ok(GpuMesh {
    //         vertex_buffer,
    //         vertex_count: mesh.count_vertices(),
    //         buffer_info,
    //         primitive_topology: mesh.primitive_topology(),
    //         layout: mesh_vertex_buffer_layout,
    //     })
    // }
}