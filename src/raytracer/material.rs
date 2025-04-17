use crate::raytracer::material::texture::Texture;
use crate::raytracer::tracer::TraceResult;
use crate::raytracer::world::Ray;
use enum_dispatch::enum_dispatch;
use glam::{Vec3, Vec4};

pub struct ScatterResult {
    pub attenuation: Vec4,
    pub scattered: Ray,
}

#[enum_dispatch]
pub trait Material {
    fn scatter(&self, ray: &Ray, trace_result: &TraceResult) -> Option<ScatterResult>;
}

#[enum_dispatch(Material)]
#[derive(Clone)]
pub enum MaterialType {
    Lambertian,
    Metal,
    Dielectric,
}

#[derive(Clone)]
pub struct Lambertian {
    pub texture: Texture,
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, trace_result: &TraceResult) -> Option<ScatterResult> {
        let mut scatter_direction = trace_result.normal + random_unit_vector();

        // Catch degenerate scatter direction
        if scatter_direction.x < 1e-8 && scatter_direction.y < 1e-8 && scatter_direction.z < 1e-8 {
            scatter_direction = trace_result.normal;
        }

        Some(ScatterResult {
            attenuation: self.texture.sample(trace_result.uv),
            scattered: Ray::new(trace_result.point, scatter_direction),
        })
    }
}

#[derive(Clone)]
pub struct Metal {
    pub albedo: Vec4,
    pub fuzziness: f32,
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, trace_result: &TraceResult) -> Option<ScatterResult> {
        let reflected = ray.direction.reflect(trace_result.normal);
        let reflected = reflected.normalize() + self.fuzziness * random_unit_vector();
        let scattered = Ray::new(trace_result.point, reflected);

        if scattered.direction.dot(trace_result.normal) <= 0.0 {
            return None;
        }

        Some(ScatterResult {
            attenuation: self.albedo,
            scattered,
        })
    }
}

#[derive(Clone)]
pub struct Dielectric {
    pub refractive_index: f32,
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, trace_result: &TraceResult) -> Option<ScatterResult> {
        let refraction_ratio = if trace_result.front_face {
            1.0 / self.refractive_index
        } else {
            self.refractive_index
        };

        let unit_direction = ray.direction.normalize();

        let cos_theta = (-unit_direction).dot(trace_result.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction =
            if cannot_refract || reflectance(cos_theta, refraction_ratio) > rand::random() {
                unit_direction.reflect(trace_result.normal)
            } else {
                unit_direction.refract(trace_result.normal, refraction_ratio)
            };

        Some(ScatterResult {
            attenuation: Vec4::ONE,
            scattered: Ray::new(trace_result.point, direction),
        })
    }
}

fn reflectance(cosine: f32, refractive_index: f32) -> f32 {
    let r0 = (1.0 - refractive_index) / (1.0 + refractive_index);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

fn random_unit_vector() -> Vec3 {
    loop {
        let random_vec: Vec3 = rand::random();
        let length = random_vec.length_squared();
        if length <= 1.0 && length > 0.1 {
            return random_vec / length.sqrt();
        }
    }
}

pub mod texture {
    use glam::Vec4;

    #[derive(Clone)]
    pub enum Texture {
        Solid {
            color: Vec4,
        },
        Checker {
            color1: Vec4,
            color2: Vec4,
            scale: f32,
        },
    }

    impl Texture {
        pub fn sample(&self, uv: (f32, f32)) -> Vec4 {
            match self {
                Texture::Solid { color } => *color,
                Texture::Checker {
                    color1,
                    color2,
                    scale,
                } => {
                    let x = (uv.0 / scale).floor() as i32;
                    let y = (uv.1 / scale).floor() as i32;

                    if (x + y) % 2 == 0 { *color1 } else { *color2 }
                }
            }
        }
    }
}
