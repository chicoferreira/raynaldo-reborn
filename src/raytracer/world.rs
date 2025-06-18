use crate::raytracer::material::MaterialType;
use glam::{Vec2, Vec3};

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

#[derive(Clone)]
pub struct World {
    pub geometry: Vec<Geometry>,
}

#[derive(Clone)]
pub struct Geometry {
    pub geometry_type: GeometryType,
    pub material: MaterialType,
}

#[derive(Clone)]
pub enum GeometryType {
    Sphere { center: Vec3, radius: f32 },
    Quad { origin: Vec3, u: Vec3, v: Vec3 },
    TriangleMesh(TriangleMeshGeometry),
}

#[derive(Clone)]
pub struct TriangleMeshGeometry {
    pub verts: Vec<(f32, f32, f32)>,
    pub indices: Vec<(u32, u32, u32)>,
    pub tex_coords: Vec<Vec2>,
}
