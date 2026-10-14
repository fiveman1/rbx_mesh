use super::roundtrip;
use crate::mesh::{Mesh2, Mesh3, Mesh4, Mesh5, Revision2, Revision3, Revision4, Revision5};
use std::fs::read;

#[cfg(feature = "mesh-v1")]
#[test]
fn mesh_100() {
	use crate::mesh::{Mesh1, Revision1};
	let bytes = read("meshes/158071912").unwrap();
	let mesh = super::readonly::<Mesh1>(bytes).unwrap();
	assert_eq!(mesh.revision, Revision1::Version100);
	insta::assert_debug_snapshot!(mesh);
}
#[test]
fn mesh_200() {
	let bytes = read("meshes/torso.mesh").unwrap();
	let mesh = roundtrip::<Mesh2>(bytes).unwrap();
	assert_eq!(mesh.revision, Revision2::Version200);
	insta::assert_debug_snapshot!(mesh);
}
#[test]
fn mesh_300() {
	let bytes = read("meshes/5115672913").unwrap();
	let mesh = roundtrip::<Mesh3>(bytes).unwrap();
	assert_eq!(mesh.revision, Revision3::Version300);
	insta::assert_debug_snapshot!(mesh);
}
#[test]
fn mesh_301() {
	let bytes = read("meshes/5648093777").unwrap();
	let mesh = roundtrip::<Mesh3>(bytes).unwrap();
	assert_eq!(mesh.revision, Revision3::Version301);
	insta::assert_debug_snapshot!(mesh);
}
#[test]
fn mesh_401() {
	let bytes = read("meshes/sphere.mesh").unwrap();
	let mesh = roundtrip::<Mesh4>(bytes).unwrap();
	assert_eq!(mesh.revision, Revision4::Version401);
	insta::assert_debug_snapshot!(mesh);
}
#[test]
fn mesh_401_random_padding() {
	let bytes = read("meshes/7665777615").unwrap();
	let mesh = roundtrip::<Mesh4>(bytes).unwrap();
	assert_eq!(mesh.revision, Revision4::Version401);
	insta::assert_debug_snapshot!(mesh);
}
//the only three v5.00 meshes I could find in bhop and surf
#[test]
fn mesh_500() {
	let bytes = read("meshes/13674780763").unwrap();
	let mesh = roundtrip::<Mesh5>(bytes).unwrap();
	assert_eq!(mesh.revision, Revision5::Version500);
	insta::assert_debug_snapshot!(mesh);
}
#[test]
fn mesh_500_alt1() {
	let bytes = read("meshes/14818281896").unwrap();
	let mesh = roundtrip::<Mesh5>(bytes).unwrap();
	assert_eq!(mesh.revision, Revision5::Version500);
	insta::assert_debug_snapshot!(mesh);
}
#[test]
fn mesh_500_alt2() {
	let bytes = read("meshes/15256456161").unwrap();
	let mesh = roundtrip::<Mesh5>(bytes).unwrap();
	assert_eq!(mesh.revision, Revision5::Version500);
	insta::assert_debug_snapshot!(mesh);
}
//also tested against ~2500 meshes from bhop and surf maps
#[test]
fn mesh_700() {
	use crate::mesh::{Chunk7, CoreMesh7, Mesh7, Revision7};
	let bytes = read("meshes/mesh7_127279296594138.bin").unwrap();
	let mesh = roundtrip::<Mesh7>(bytes).unwrap();
	assert_eq!(mesh.revision, Revision7::Version700);
	assert!(matches!(mesh.core_mesh(), Some(CoreMesh7::Draco(_))));
	let lods = mesh.lods().unwrap();
	assert_eq!(lods.lods.first().unwrap().0, 0);
	assert!(mesh.chunks.iter().all(|chunk| !matches!(chunk, Chunk7::Other { .. })));
	#[cfg(feature = "mesh-v7-draco")]
	{
		let core_mesh = mesh.core_mesh().unwrap().decode().unwrap();
		// most meshes' last offset is the face count, but some, like this one, have offsets 0, 0
		assert_eq!(lods.lods.iter().map(|lod| lod.0).collect::<Vec<_>>(), [0, 0]);
		assert!(!core_mesh.faces.is_empty());
		let vertex_count = core_mesh.vertices.len() as u32;
		assert!(core_mesh.faces.iter().all(|face| face.0.iter().all(|vertex| vertex.0 < vertex_count)));
	}
}
#[test]
fn mesh_700_raw() {
	use crate::mesh::{CoreMesh7, Mesh7, Revision7};
	let bytes = read("meshes/mesh7_raw_100045467357727.bin").unwrap();
	let mesh = roundtrip::<Mesh7>(bytes).unwrap();
	assert_eq!(mesh.revision, Revision7::Version700);
	let Some(CoreMesh7::Raw(core_mesh)) = mesh.core_mesh() else {
		panic!("expected an uncompressed COREMESH");
	};
	assert_eq!(core_mesh.vertices.len(), 3);
	assert_eq!(core_mesh.faces.len(), 1);
	insta::assert_debug_snapshot!(mesh);
}
