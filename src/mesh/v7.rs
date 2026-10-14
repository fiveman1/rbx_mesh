// based on https://devforum.roblox.com/t/roblox-mesh-format/326114
// and observations of version 7.00 meshes: every one seen has COREMESH and LODS chunks,
// and some have SKINNING and FACS chunks after them.
use super::v2::{Face2, Vertex2};
use super::v3::Lod3;
use super::v4::LodType4;

#[binrw::binrw]
#[brw(little)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Revision7 {
	#[brw(magic = b"version 7.00\n")]
	Version700,
}

/// Vertices and faces stored as they are, like a version 4 mesh without its header.
#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct CoreMeshRaw7 {
	#[br(temp)]
	#[bw(try_calc=vertices.len().try_into())]
	pub vertex_count: u32,
	#[br(count=vertex_count)]
	pub vertices: Vec<Vertex2>,
	#[br(temp)]
	#[bw(try_calc=faces.len().try_into())]
	pub face_count: u32,
	#[br(count=face_count)]
	pub faces: Vec<Face2>,
}
impl CoreMeshRaw7 {
	/// The serialization length of this struct in bytes.
	fn len(&self) -> usize {
		size_of::<u32>()
			+ self.vertices.len() * size_of::<Vertex2>()
			+ size_of::<u32>()
			+ self.faces.len() * size_of::<Face2>()
	}
}

/// Vertices and faces compressed with Google's Draco.
/// Use [CoreMeshDraco7::decode] (feature `mesh-v7-draco`) to get them.
#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct CoreMeshDraco7 {
	#[br(temp)]
	#[bw(try_calc=data.len().try_into())]
	pub data_len: u32,
	/// A Draco mesh bitstream
	#[br(count=data_len)]
	pub data: Vec<u8>,
}
impl CoreMeshDraco7 {
	/// The serialization length of this struct in bytes.
	fn len(&self) -> usize {
		size_of::<u32>() + self.data.len()
	}
}

/// Which faces make up each level of detail.
/// Level i is faces lods[i]..lods[i+1], the first being the most detailed.
#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct Lods7 {
	pub lod_type: LodType4,
	pub lod_hq_count: u8,
	#[br(temp)]
	#[bw(try_calc=lods.len().try_into())]
	pub lod_count: u32,
	#[br(count=lod_count)]
	pub lods: Vec<Lod3>,
}
impl Lods7 {
	/// The serialization length of this struct in bytes.
	fn len(&self) -> usize {
		size_of::<u16>() + size_of::<u8>() + size_of::<u32>() + self.lods.len() * size_of::<Lod3>()
	}
}

/// A chunk is an 8 byte name, a version and a size, followed by that many bytes.
#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub enum Chunk7 {
	#[brw(magic = b"COREMESH\x01\x00\x00\x00")]
	CoreMeshRaw {
		#[br(temp)]
		#[bw(try_calc=core_mesh.len().try_into())]
		size: u32,
		#[br(pad_size_to=size)]
		core_mesh: CoreMeshRaw7,
	},
	#[brw(magic = b"COREMESH\x02\x00\x00\x00")]
	CoreMeshDraco {
		#[br(temp)]
		#[bw(try_calc=core_mesh.len().try_into())]
		size: u32,
		#[br(pad_size_to=size)]
		core_mesh: CoreMeshDraco7,
	},
	#[brw(magic = b"LODS\x00\x00\x00\x00\x01\x00\x00\x00")]
	Lods {
		#[br(temp)]
		#[bw(try_calc=lods.len().try_into())]
		size: u32,
		#[br(pad_size_to=size)]
		lods: Lods7,
	},
	/// Chunks not parsed yet, like SKINNING and FACS, or unknown versions of the others.
	Other {
		name: [u8; 8],
		version: u32,
		#[br(temp)]
		#[bw(try_calc=data.len().try_into())]
		size: u32,
		#[br(count=size)]
		data: Vec<u8>,
	},
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct Mesh7 {
	pub revision: Revision7,
	#[br(parse_with=binrw::helpers::until_eof)]
	pub chunks: Vec<Chunk7>,
}
impl Mesh7 {
	/// The first COREMESH chunk, which holds the vertices and faces.
	pub fn core_mesh(&self) -> Option<CoreMesh7<'_>> {
		self.chunks.iter().find_map(|chunk| match chunk {
			Chunk7::CoreMeshRaw { core_mesh } => Some(CoreMesh7::Raw(core_mesh)),
			Chunk7::CoreMeshDraco { core_mesh } => Some(CoreMesh7::Draco(core_mesh)),
			_ => None,
		})
	}
	/// The first LODS chunk.
	pub fn lods(&self) -> Option<&Lods7> {
		self.chunks.iter().find_map(|chunk| match chunk {
			Chunk7::Lods { lods } => Some(lods),
			_ => None,
		})
	}
}

/// A COREMESH chunk in either of its versions.
#[derive(Debug, Clone, Copy)]
pub enum CoreMesh7<'a> {
	Raw(&'a CoreMeshRaw7),
	Draco(&'a CoreMeshDraco7),
}
#[cfg(feature = "mesh-v7-draco")]
impl<'a> CoreMesh7<'a> {
	/// The vertices and faces, decompressing them if needed.
	pub fn decode(self) -> Result<std::borrow::Cow<'a, CoreMeshRaw7>, DracoError7> {
		match self {
			CoreMesh7::Raw(core_mesh) => Ok(std::borrow::Cow::Borrowed(core_mesh)),
			CoreMesh7::Draco(core_mesh) => Ok(std::borrow::Cow::Owned(core_mesh.decode()?)),
		}
	}
}

#[cfg(feature = "mesh-v7-draco")]
#[derive(Debug)]
pub enum DracoError7 {
	Draco(draco_core::DracoError),
	/// Every Roblox Draco mesh seen has positions, and a mesh without them has nothing to draw.
	MissingPosition,
}
#[cfg(feature = "mesh-v7-draco")]
impl std::fmt::Display for DracoError7 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self:?}")
	}
}
#[cfg(feature = "mesh-v7-draco")]
impl std::error::Error for DracoError7 {}

#[cfg(feature = "mesh-v7-draco")]
impl CoreMeshDraco7 {
	/// Decompress the vertices and faces.
	///
	/// Roblox stores the same vertex as raw meshes do, as Draco attributes:
	/// position (float x3), normal (generic float x3), texture coordinate (float x2),
	/// tangent (generic u8 x4, the bytes of the signed tangent) and color (u8 x4).
	/// Attributes other than position that are missing are filled with zero.
	pub fn decode(&self) -> Result<CoreMeshRaw7, DracoError7> {
		use draco_core::{DataType, FaceIndex, GeometryAttributeType, MeshDecoder};

		let mut buffer = draco_core::DecoderBuffer::new(&self.data);
		let mut mesh = draco_core::Mesh::new();
		MeshDecoder::new()
			.decode(&mut buffer, &mut mesh)
			.map_err(DracoError7::Draco)?;

		let point_count = mesh.num_points();
		// the values of each point for the first attribute that matches, in point order
		let read = |matches: &dyn Fn(&draco_core::PointAttribute) -> bool, components: usize| {
			(0..mesh.num_attributes())
				.map(|attribute_id| mesh.attribute(attribute_id))
				.find(|attribute| matches(attribute))
				.map(|attribute| attribute.read_f32s(point_count, components))
		};
		let generic = |components: u8, data_type: DataType| {
			move |attribute: &draco_core::PointAttribute| {
				attribute.attribute_type() == GeometryAttributeType::Generic
					&& attribute.num_components() == components
					&& attribute.data_type() == data_type
			}
		};
		let named = |attribute_type: GeometryAttributeType| {
			move |attribute: &draco_core::PointAttribute| attribute.attribute_type() == attribute_type
		};
		let positions = Some(read(&named(GeometryAttributeType::Position), 3).ok_or(DracoError7::MissingPosition)?);
		let normals = read(&generic(3, DataType::Float32), 3);
		let tex_coords = read(&named(GeometryAttributeType::TexCoord), 2);
		let tangents = read(&generic(4, DataType::Uint8), 4);
		let colors = read(&named(GeometryAttributeType::Color), 4);

		// the attribute's components for one point, or zero when the mesh doesn't have it
		fn point<const N: usize>(values: &Option<Vec<f32>>, point_id: usize) -> [f32; N] {
			match values {
				Some(values) => core::array::from_fn(|i| values[point_id * N + i]),
				None => [0.0; N],
			}
		}
		let vertices = (0..point_count)
			.map(|point_id| Vertex2 {
				pos: point(&positions, point_id),
				norm: point(&normals, point_id),
				tex: point(&tex_coords, point_id),
				tangent: point(&tangents, point_id).map(|value: f32| value as u8 as i8),
				color: point(&colors, point_id).map(|value: f32| value as u8),
			})
			.collect();
		let faces = (0..mesh.num_faces())
			.map(|face_id| {
				let face = mesh.face(FaceIndex(face_id as u32));
				Face2(face.map(|point_id| super::v2::VertexId2(point_id.0)))
			})
			.collect();
		Ok(CoreMeshRaw7 { vertices, faces })
	}
}
