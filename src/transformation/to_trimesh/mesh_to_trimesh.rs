use super::ToTriMesh;
use crate::procedural::{IndexBuffer, TriMesh, TriMesh3};
use crate::shape;
use na;
use na::Point3;
use simba::scalar::RealField;

impl<N: RealField> ToTriMesh<Point3<N>, ()> for shape::TriMesh3<N> {
    fn to_trimesh(&self, _: ()) -> TriMesh3<N> {
        TriMesh::new(
            (**self.vertices()).clone(),
            self.normals().as_ref().map(|ns| (**ns).clone()),
            self.uvs().as_ref().map(|ns| (**ns).clone()),
            Some(IndexBuffer::Unified(
                (**self.indices()).iter().map(|e| na::convert(*e)).collect(),
            )),
        )
    }
}
