use crate::raytracer::tracer::{TraceResult, Tracer};
use crate::raytracer::world::{GeometryType, Ray};
use enum_dispatch::enum_dispatch;
use glam::Vec3;
use std::ops::RangeBounds;

pub struct NaiveTracer {
    objects: Vec<NaiveObject>,
}

impl NaiveTracer {
    pub fn new(objects: &[GeometryType]) -> Self {
        let objects = objects
            .into_iter()
            .map(|o| match o {
                GeometryType::Sphere { center, radius } => NaiveObject::NaiveSphere(NaiveSphere {
                    center: *center,
                    radius: *radius,
                }),
                GeometryType::Plane { point, normal } => NaiveObject::NaivePlane(NaivePlane {
                    point: *point,
                    normal: *normal,
                }),
            })
            .collect();
        Self { objects }
    }
}

impl Tracer for NaiveTracer {
    fn trace(&self, ray: &Ray, range: &impl RangeBounds<f32>) -> Option<TraceResult> {
        let mut closest_hit: Option<TraceResult> = None;
        for (index, object) in self.objects.iter().enumerate() {
            if let Some(hit) = object.hit(index, ray, range) {
                if let Some(ref mut closest_hit) = closest_hit {
                    if hit.distance < closest_hit.distance {
                        *closest_hit = hit;
                    }
                } else {
                    closest_hit = Some(hit);
                }
            }
        }
        closest_hit
    }
}

#[enum_dispatch(Hittable)]
enum NaiveObject {
    NaiveSphere,
    NaivePlane,
}

impl Into<NaiveObject> for GeometryType {
    fn into(self) -> NaiveObject {
        match self {
            GeometryType::Sphere { center, radius } => {
                NaiveObject::NaiveSphere(NaiveSphere { center, radius })
            }
            GeometryType::Plane { point, normal } => {
                NaiveObject::NaivePlane(NaivePlane { point, normal })
            }
        }
    }
}

#[enum_dispatch]
trait Hittable {
    fn hit(&self, my_index: usize, ray: &Ray, range: &impl RangeBounds<f32>)
    -> Option<TraceResult>;
}

struct NaiveSphere {
    center: Vec3,
    radius: f32,
}

struct NaivePlane {
    point: Vec3,
    normal: Vec3,
}

impl Hittable for NaiveSphere {
    fn hit(&self, index: usize, ray: &Ray, range: &impl RangeBounds<f32>) -> Option<TraceResult> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }
        let positive_solution = (-b + discriminant.sqrt()) / (2.0 * a);
        let negative_solution = (-b - discriminant.sqrt()) / (2.0 * a);

        let t = if range.contains(&negative_solution) {
            negative_solution
        } else if range.contains(&positive_solution) {
            positive_solution
        } else {
            return None;
        };

        let point = ray.at(t);
        let normal = (point - self.center).normalize();

        let front_face = ray.direction.dot(normal) < 0.0;
        let normal = if front_face { normal } else { -normal };

        Some(TraceResult {
            distance: t,
            point,
            normal,
            geometry_index: index,
            front_face,
        })
    }
}

impl Hittable for NaivePlane {
    fn hit(&self, index: usize, ray: &Ray, range: &impl RangeBounds<f32>) -> Option<TraceResult> {
        let denom = self.normal.dot(ray.direction);
        if denom.abs() < 1e-6 {
            return None; // Ray is parallel to the plane
        }
        let t = (self.point - ray.origin).dot(self.normal) / denom;
        if !range.contains(&t) {
            return None; // Intersection is outside the range
        }
        let point = ray.at(t);
        let front_face = ray.direction.dot(self.normal) < 0.0;
        Some(TraceResult {
            distance: t,
            point,
            normal: self.normal,
            geometry_index: index,
            front_face,
        })
    }
}
