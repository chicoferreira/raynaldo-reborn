use crate::raytracer::tracer::TraceResult;
use crate::raytracer::world::{GeometryType, Ray};
use glam::Vec3;
use std::f32::consts::PI;
use std::ops::RangeBounds;

pub struct NaiveTracer {
    objects: Vec<NaiveObject>,
}

impl NaiveTracer {
    pub fn new(objects: &[GeometryType]) -> Self {
        let objects = objects
            .into_iter()
            .map(|o| match o {
                GeometryType::Sphere { center, radius } => NaiveObject::NaiveSphere {
                    center: *center,
                    radius: *radius,
                },
                GeometryType::Quad {
                    origin: point,
                    u,
                    v,
                } => NaiveObject::NaiveQuad(NaiveQuad::new(*point, *u, *v)),
            })
            .collect();
        Self { objects }
    }
}

impl NaiveTracer {
    pub fn trace(&self, ray: &Ray, range: &impl RangeBounds<f32>) -> Option<TraceResult> {
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

enum NaiveObject {
    NaiveSphere { center: Vec3, radius: f32 },
    NaiveQuad(NaiveQuad),
}

impl NaiveObject {
    fn hit(
        &self,
        my_index: usize,
        ray: &Ray,
        range: &impl RangeBounds<f32>,
    ) -> Option<TraceResult> {
        match self {
            NaiveObject::NaiveSphere { center, radius } => {
                Self::intersect_sphere(*center, *radius, my_index, ray, range)
            }
            NaiveObject::NaiveQuad(quad) => Self::intersect_quad(quad, my_index, ray, range),
        }
    }

    fn intersect_sphere(
        center: Vec3,
        radius: f32,
        geometry_index: usize,
        ray: &Ray,
        range: &impl RangeBounds<f32>,
    ) -> Option<TraceResult> {
        let oc = ray.origin - center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * oc.dot(ray.direction);
        let c = oc.dot(oc) - radius * radius;
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
        let normal = (point - center).normalize();

        let front_face = ray.direction.dot(normal) < 0.0;
        let normal = if front_face { normal } else { -normal };

        let theta = (-point.y).acos();
        let phi = (-point.z).atan2(point.x) + PI;

        let u = phi / (2.0 * PI);
        let v = theta / PI;

        Some(TraceResult {
            distance: t,
            point,
            normal,
            geometry_index,
            front_face,
            uv: (u, v),
        })
    }

    fn intersect_quad(
        quad: &NaiveQuad,
        geometry_index: usize,
        ray: &Ray,
        range: &impl RangeBounds<f32>,
    ) -> Option<TraceResult> {
        let denominator = quad.normal.dot(ray.direction);

        if denominator.abs() < 1e-6 {
            return None;
        }

        let distance = (quad.d - quad.normal.dot(ray.origin)) / denominator;
        if !range.contains(&distance) {
            return None;
        }

        let point = ray.at(distance);
        let vector_in_plane = point - quad.origin;

        let u_coord = vector_in_plane.dot(quad.u) / quad.u.length_squared();
        let v_coord = vector_in_plane.dot(quad.v) / quad.v.length_squared();

        if u_coord < 0.0 || u_coord > 1.0 || v_coord < 0.0 || v_coord > 1.0 {
            return None;
        }

        let front_face = ray.direction.dot(quad.normal) < 0.0;

        let hit_normal = if front_face {
            quad.normal
        } else {
            -quad.normal
        };

        Some(TraceResult {
            distance,
            point,
            normal: hit_normal,
            geometry_index,
            front_face,
            uv: (u_coord, v_coord),
        })
    }
}

struct NaiveQuad {
    origin: Vec3,
    u: Vec3,
    v: Vec3,
    normal: Vec3,
    d: f32,
}

impl NaiveQuad {
    fn new(origin: Vec3, u: Vec3, v: Vec3) -> Self {
        let normal = u.cross(v).normalize();
        let d = normal.dot(origin);
        Self {
            origin,
            u,
            v,
            normal,
            d,
        }
    }
}
