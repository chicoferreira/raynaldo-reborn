use crate::raytracer::material::MaterialType;
use glam::Vec3;
use serde::{Deserialize, Serialize};

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct World {
    pub geometry: Vec<Geometry>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Geometry {
    #[serde(flatten)]
    pub geometry_type: GeometryType,
    #[serde(flatten)]
    pub material: MaterialType,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum GeometryType {
    Sphere {
        center: Vec3,
        radius: f32,
    },
    Quad {
        origin: Vec3,
        u: Vec3,
        v: Vec3,
    },
    #[serde(
        deserialize_with = "crate::raytracer::loader::deserialize_triangle_mesh",
        serialize_with = "crate::raytracer::loader::serialize_triangle_mesh"
    )]
    TriangleMesh(TriangleMeshGeometry),
    Box {
        origin: Vec3,
        u: Vec3,
        v: Vec3,
        w: Vec3,
    },
}

impl GeometryType {
    /// Samples a random point on the geometry.
    /// Returns (point, normal_at_that_point)
    pub fn sample_random_point(&self) -> (Vec3, Vec3) {
        match self {
            GeometryType::Sphere { center, radius } => {
                let theta = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
                let phi = rand::random::<f32>() * std::f32::consts::PI;
                let normal = Vec3::new(
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                );
                let point = center + radius * normal;
                (point, normal)
            }
            GeometryType::Quad { origin, u, v } => {
                let point = origin + u * rand::random::<f32>() + v * rand::random::<f32>();
                let normal = u.cross(*v).normalize();
                (point, normal)
            }
            GeometryType::TriangleMesh(mesh) => {
                // NOT THE CORRECT WAY TO SAMPLE A TRIANGLE MESH WHEN TRIANGLES ARE NOT UNIFORM
                let index = rand::random_range(0..mesh.indices.len());
                let (i0, i1, i2) = mesh.indices[index];
                let v0 = Vec3::from(mesh.verts[i0 as usize]);
                let v1 = Vec3::from(mesh.verts[i1 as usize]);
                let v2 = Vec3::from(mesh.verts[i2 as usize]);

                let u = rand::random::<f32>();
                let v = rand::random::<f32>();
                let point = v0 * u + v1 * v + v2 * (1.0 - u - v);
                let normal = (v1 - v0).cross(v2 - v0).normalize();
                (point, normal)
            }
            GeometryType::Box { origin, u, v, w } => {
                // NOT THE CORRECT WAY TO SAMPLE A BOX WHEN FACES ARE NOT UNIFORM
                let (s1, s2) = (rand::random::<f32>(), rand::random::<f32>());
                match rand::random_range(0..6) {
                    0 => (origin + u * s1 + v * s2, u.cross(*v).normalize()),
                    1 => (origin + u * s1 + w * s2, u.cross(*w).normalize()),
                    2 => (origin + v * s1 + w * s2, v.cross(*w).normalize()),
                    3 => (origin + u * s1 - v * s2, u.cross(-*v).normalize()),
                    4 => (origin + v * s1 - w * s2, v.cross(-*w).normalize()),
                    _ => (origin + w * s1 - v * s2, w.cross(-*v).normalize()),
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct TriangleMeshGeometry {
    pub verts: Vec<(f32, f32, f32)>,
    pub indices: Vec<(u32, u32, u32)>,
    // TODO: Add texture coordinates to uv mapping
    // pub tex_coords: Vec<Vec2>,
}
