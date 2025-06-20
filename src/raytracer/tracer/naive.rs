use crate::raytracer::tracer::TraceResult;
use crate::raytracer::world::{Geometry, GeometryType, Ray};
use glam::Vec3;
use std::f32::consts::PI;
use std::ops::RangeBounds;

pub struct NaiveTracer {
    objects: Vec<NaiveObject>,
}

impl NaiveTracer {
    pub fn new(geometry: &[Geometry]) -> Self {
        let mut objects = Vec::new();

        for (index, geom) in geometry.iter().enumerate() {
            match &geom.geometry_type {
                GeometryType::Sphere { center, radius } => {
                    objects.push(NaiveObject {
                        geometry_index: index,
                        geometry: NaiveGeometry::Sphere {
                            center: *center,
                            radius: *radius,
                        },
                    });
                }
                GeometryType::Quad { origin, u, v } => {
                    let normal = u.cross(*v).normalize();
                    let d = normal.dot(*origin);

                    objects.push(NaiveObject {
                        geometry_index: index,
                        geometry: NaiveGeometry::Quad {
                            origin: *origin,
                            u: *u,
                            v: *v,
                            normal,
                            d,
                        },
                    });
                }
                GeometryType::TriangleMesh(mesh) => {
                    for (v1, v2, v3) in mesh.indices.iter() {
                        let p1 = mesh.verts[*v1 as usize].into();
                        let p2 = mesh.verts[*v2 as usize].into();
                        let p3 = mesh.verts[*v3 as usize].into();

                        objects.push(NaiveObject {
                            geometry_index: index,
                            geometry: NaiveGeometry::Triangle { p1, p2, p3 },
                        });
                    }
                }
            }
        }

        Self { objects }
    }
}

impl NaiveTracer {
    pub fn trace(&self, ray: &Ray, range: &impl RangeBounds<f32>) -> Option<TraceResult> {
        let mut closest_hit: Option<TraceResult> = None;
        for object in self.objects.iter() {
            if let Some(hit) = object.hit(object.geometry_index, ray, range) {
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

struct NaiveObject {
    geometry_index: usize,
    geometry: NaiveGeometry,
}

enum NaiveGeometry {
    Sphere {
        center: Vec3,
        radius: f32,
    },
    Quad {
        origin: Vec3,
        u: Vec3,
        v: Vec3,
        normal: Vec3,
        d: f32,
    },
    Triangle {
        p1: Vec3,
        p2: Vec3,
        p3: Vec3,
    },
}

impl NaiveObject {
    fn hit(
        &self,
        my_index: usize,
        ray: &Ray,
        range: &impl RangeBounds<f32>,
    ) -> Option<TraceResult> {
        match &self.geometry {
            NaiveGeometry::Sphere { center, radius } => {
                Self::intersect_sphere(*center, *radius, my_index, ray, range)
            }
            NaiveGeometry::Quad {
                origin,
                u,
                v,
                normal,
                d,
            } => Self::intersect_quad(*origin, *u, *v, *normal, *d, my_index, ray, range),
            NaiveGeometry::Triangle { p1, p2, p3 } => {
                Self::intersect_triangle(*p1, *p2, *p3, my_index, ray, range)
            }
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
        origin: Vec3,
        u: Vec3,
        v: Vec3,
        normal: Vec3,
        d: f32,
        geometry_index: usize,
        ray: &Ray,
        range: &impl RangeBounds<f32>,
    ) -> Option<TraceResult> {
        let denominator = normal.dot(ray.direction);

        if denominator.abs() < 1e-6 {
            return None;
        }

        let distance = (d - normal.dot(ray.origin)) / denominator;
        if !range.contains(&distance) {
            return None;
        }

        let point = ray.at(distance);
        let vector_in_plane = point - origin;

        let u_coord = vector_in_plane.dot(u) / u.length_squared();
        let v_coord = vector_in_plane.dot(v) / v.length_squared();

        if u_coord < 0.0 || u_coord > 1.0 || v_coord < 0.0 || v_coord > 1.0 {
            return None;
        }

        let front_face = ray.direction.dot(normal) < 0.0;

        let hit_normal = if front_face { normal } else { -normal };

        Some(TraceResult {
            distance,
            point,
            normal: hit_normal,
            geometry_index,
            front_face,
            uv: (u_coord, v_coord),
        })
    }

    fn intersect_triangle(
        p1: Vec3,
        p2: Vec3,
        p3: Vec3,
        geometry_index: usize,
        ray: &Ray,
        range: &impl RangeBounds<f32>,
    ) -> Option<TraceResult> {
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let h = ray.direction.cross(e2);
        let a = e1.dot(h);
        if a.abs() < 1e-6 {
            return None;
        }

        let f = 1.0 / a;
        let s = ray.origin - p1;
        let u = f * s.dot(h);
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = s.cross(e1);
        let v = f * ray.direction.dot(q);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * e2.dot(q);
        if !range.contains(&t) {
            return None;
        }

        let point = ray.at(t);
        let normal = (e1.cross(e2)).normalize();

        let front_face = ray.direction.dot(normal) < 0.0;
        let normal = if front_face { normal } else { -normal };

        Some(TraceResult {
            distance: t,
            point,
            normal,
            geometry_index,
            front_face,
            uv: (u, v),
        })
    }
}
