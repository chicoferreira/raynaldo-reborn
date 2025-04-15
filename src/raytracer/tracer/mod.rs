pub mod naive;

use crate::raytracer::tracer::naive::NaiveTracer;
use crate::raytracer::world::Ray;
use enum_dispatch::enum_dispatch;
use glam::Vec3;
use std::ops::RangeBounds;

pub struct TraceResult {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub geometry_index: usize,
    pub front_face: bool,
}

#[enum_dispatch]
pub trait Tracer {
    fn trace(&self, ray: &Ray, bounds: &impl RangeBounds<f32>) -> Option<TraceResult>;
}

#[enum_dispatch(Tracer)]
pub enum TracerType {
    NaiveTracer,
}
