use crate::bounding_volume::{BoundingSphere, BoundingVolume, HasBoundingVolume};
use crate::math::Isometry;
use na::Real;
use crate::shape::Compound;

impl<N: Real> HasBoundingVolume<N, BoundingSphere<N>> for Compound<N> {
    #[inline]
    fn bounding_volume(&self, m: &Isometry<N>) -> BoundingSphere<N> {
        self.aabb().bounding_sphere().transform_by(m)
    }
}
