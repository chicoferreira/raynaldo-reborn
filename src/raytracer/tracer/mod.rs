pub mod embree;
pub mod naive;

use crate::raytracer::tracer::embree::EmbreeTracer;
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

pub enum Tracer {
    NaiveTracer(NaiveTracer),
    EmbreeTracer(EmbreeTracer),
}

impl Tracer {
    pub fn trace(&self, ray: &Ray, bounds: &impl RangeBounds<f32>) -> Option<TraceResult> {
        match self {
            Tracer::NaiveTracer(tracer) => tracer.trace(ray, bounds),
            Tracer::EmbreeTracer(tracer) => tracer.trace(ray, bounds),
        }
    }
}
