use na;
use na::{Point2, Point3};
use super::{IndexBuffer, TriMesh};
use math::Point;

/// Adds a double-sided quad to the scene.
///
/// The quad is initially centered at (0, 0, 0). Its normal is the `z` axis. The quad itself is
/// composed of a user-defined number of triangles regularly spaced on a grid. This is the main way
/// to draw height maps.
///
/// # Arguments
/// * `w` - the quad width.
/// * `h` - the quad height.
/// * `usubdivs` - number of horizontal subdivisions. This correspond to the number of squares
/// which will be placed horizontally on each line. Must not be `0`.
/// * `vsubdivs` - number of vertical subdivisions. This correspond to the number of squares
/// which will be placed vertically on each line. Must not be `0`.
pub fn quad<P>(width: P::Real, height: P::Real, usubdivs: usize, vsubdivs: usize) -> TriMesh<P>
where
    P: Point,
{
    let mut quad = unit_quad::<P>(usubdivs, vsubdivs);

    let mut s = na::zero::<P::Vector>();
    s[0] = width;
    s[1] = height;

    for i in 2..na::dimension::<P::Vector>() {
        s[i] = na::one();
    }

    quad.scale_by(&s);

    quad
}

/// Adds a double-sided quad with the specified grid of vertices.
///
/// Normals are automatically computed.
///
/// # Arguments
/// * `nhpoints` - number of columns on the grid.
/// * `nvpoints` - number of lines on the grid.
pub fn quad_with_vertices<P>(vertices: &[P], nhpoints: usize, nvpoints: usize) -> TriMesh<P>
where
    P: Point,
{
    assert!(
        nhpoints > 1 && nvpoints > 1,
        "The number of points must be at least 2 in each dimension."
    );

    let mut res = unit_quad::<P>(nhpoints - 1, nvpoints - 1);

    for (dest, src) in res.coords.iter_mut().zip(vertices.iter()) {
        *dest = src.clone();
    }

    res
}

/// Adds a double-sided quad with unit size to the scene.
///
/// The quad is initially centered at (0, 0, 0). Its normal is the `z` axis. The quad itself is
/// composed of a user-defined number of triangles regularly spaced on a grid. This is the main way
/// to draw height maps.
///
/// # Arguments
/// * `usubdivs` - number of horizontal subdivisions. This correspond to the number of squares
/// which will be placed horizontally on each line. Must not be `0`.
/// * `vsubdivs` - number of vertical subdivisions. This correspond to the number of squares
/// which will be placed vertically on each line. Must not be `0`.
pub fn unit_quad<P>(usubdivs: usize, vsubdivs: usize) -> TriMesh<P>
where
    P: Point,
{
    assert!(
        usubdivs > 0 && vsubdivs > 0,
        "The number of subdivisions cannot be zero"
    );
    assert!(na::dimension::<P::Vector>() >= 2);

    let wstep = na::one::<P::Real>() / na::convert(usubdivs as f64);
    let hstep = na::one::<P::Real>() / na::convert(vsubdivs as f64);
    let cw = na::convert(0.5);
    let ch = na::convert(0.5);

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut triangles = Vec::new();
    let mut tex_coords = Vec::new();

    // create the vertices
    for i in 0usize..vsubdivs + 1 {
        for j in 0usize..usubdivs + 1 {
            let ni: P::Real = na::convert(i as f64);
            let nj: P::Real = na::convert(j as f64);

            let mut v = P::origin();
            v[0] = nj * wstep - cw;
            v[1] = ni * hstep - ch;
            vertices.push(v);
            let _1 = na::one::<P::Real>();
            tex_coords.push(Point2::new(_1 - nj * wstep, _1 - ni * hstep))
        }
    }

    // create the normals
    for _ in 0..(vsubdivs + 1) * (usubdivs + 1) {
        let mut n = na::zero::<P::Vector>();
        n[0] = na::one();
        normals.push(n)
    }

    // create triangles
    fn dl_triangle(i: u32, j: u32, ws: u32) -> Point3<u32> {
        Point3::new((i + 1) * ws + j, i * ws + j, (i + 1) * ws + j + 1)
    }

    fn ur_triangle(i: u32, j: u32, ws: u32) -> Point3<u32> {
        Point3::new(i * ws + j, i * ws + (j + 1), (i + 1) * ws + j + 1)
    }

    for i in 0usize..vsubdivs {
        for j in 0usize..usubdivs {
            // build two triangles...
            triangles.push(dl_triangle(i as u32, j as u32, (usubdivs + 1) as u32));
            triangles.push(ur_triangle(i as u32, j as u32, (usubdivs + 1) as u32));
        }
    }

    TriMesh::new(
        vertices,
        Some(normals),
        Some(tex_coords),
        Some(IndexBuffer::Unified(triangles)),
    )
}
