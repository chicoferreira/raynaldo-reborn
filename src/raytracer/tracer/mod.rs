pub mod embree;
pub mod naive;

use crate::raytracer::tracer::naive::NaiveTracer;
use crate::raytracer::world::Ray;
use glam::Vec3;
use std::ops::RangeBounds;

pub struct TraceResult {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub geometry_index: usize,
    pub front_face: bool,
    pub uv: (f32, f32),
}

pub enum TracerType {
    NaiveTracer(NaiveTracer),
    EmbreeTracer(embree::EmbreeRayTracer),
}

impl TracerType {
    pub fn trace(&self, ray: &Ray, bounds: &impl RangeBounds<f32>) -> Option<TraceResult> {
        match self {
            TracerType::NaiveTracer(tracer) => tracer.trace(ray, bounds),
            TracerType::EmbreeTracer(tracer) => tracer.trace(ray, bounds),
        }
    }
}
