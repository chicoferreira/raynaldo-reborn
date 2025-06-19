use crate::raytracer::material::MaterialType;
use glam::{Vec2, Vec3};
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
}

#[derive(Clone)]
pub struct TriangleMeshGeometry {
    pub verts: Vec<(f32, f32, f32)>,
    pub indices: Vec<(u32, u32, u32)>,
    // TODO: Add texture coordinates to uv mapping
    pub tex_coords: Vec<Vec2>,
}
