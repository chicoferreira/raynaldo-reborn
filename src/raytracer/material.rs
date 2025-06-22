use crate::raytracer::material::texture::Texture;
use crate::raytracer::tracer::TraceResult;
use crate::raytracer::world::Ray;
use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};

pub struct ScatterResult {
    pub attenuation: Vec4,
    pub scattered: Ray,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "material")]
pub enum MaterialType {
    Lambertian {
        #[serde(flatten)]
        texture: Texture,
    },
    Metal {
        albedo: Vec4,
        fuzziness: f32,
    },
    Dielectric {
        refractive_index: f32,
    },
    Emissive {
        color: Vec4,
        intensity: f32,
    },
}

impl MaterialType {
    pub fn emit(&self, trace_result: &TraceResult) -> Vec4 {
        match self {
            MaterialType::Emissive { color, intensity } => {
                if trace_result.front_face {
                    *color * *intensity
                } else {
                    Vec4::ZERO
                }
            }
            _ => Vec4::ZERO,
        }
    }

    pub fn scatter(&self, ray: &Ray, trace_result: &TraceResult) -> Option<ScatterResult> {
        match self {
            MaterialType::Lambertian { texture } => {
                let scatter_direction = sample_cos_hemisphere(trace_result.normal);

                Some(ScatterResult {
                    attenuation: texture.sample(trace_result.uv),
                    scattered: Ray::new(trace_result.point, scatter_direction),
                })
            }
            MaterialType::Metal { albedo, fuzziness } => {
                let reflected = ray.direction.reflect(trace_result.normal);
                let reflected = reflected.normalize() + fuzziness * random_unit_vector();
                let scattered = Ray::new(trace_result.point, reflected);

                if scattered.direction.dot(trace_result.normal) <= 0.0 {
                    return None;
                }

                Some(ScatterResult {
                    attenuation: *albedo,
                    scattered,
                })
            }
            MaterialType::Dielectric { refractive_index } => {
                let refraction_ratio = if trace_result.front_face {
                    1.0 / refractive_index
                } else {
                    *refractive_index
                };

                let unit_direction = ray.direction.normalize();

                let cos_theta = (-unit_direction).dot(trace_result.normal).min(1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

                let cannot_refract = refraction_ratio * sin_theta > 1.0;

                let direction = if cannot_refract
                    || reflectance(cos_theta, refraction_ratio) > rand::random()
                {
                    unit_direction.reflect(trace_result.normal)
                } else {
                    unit_direction.refract(trace_result.normal, refraction_ratio)
                };

                Some(ScatterResult {
                    attenuation: Vec4::ONE,
                    scattered: Ray::new(trace_result.point, direction),
                })
            }
            MaterialType::Emissive { .. } => None,
        }
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
        let random_vec = random_vec * 2.0 - Vec3::ONE;
        let length = random_vec.length_squared();
        if length <= 1.0 && length > 1e-8 {
            return random_vec / length.sqrt();
        }
    }
}

fn sample_cos_hemisphere(normal: Vec3) -> Vec3 {
    // Two random numbers in [0, 1)
    let e1: f32 = rand::random();
    let e2: f32 = rand::random();

    // Cosine-weighted sampling in local space (normal = (0, 0, 1))
    let r = e1.sqrt(); // Radius on the unit disk
    let phi = 2.0 * std::f32::consts::PI * e2; // Azimuthal angle
    let x = r * phi.cos();
    let y = r * phi.sin();
    let z = (1.0 - e1).sqrt(); // Ensures unit length and cosine weighting

    // Local direction
    let local_dir = Vec3::new(x, y, z);

    // Transform to world space using an orthonormal basis
    let (u, v, w) = orthonormal_basis(normal);
    u * local_dir.x + v * local_dir.y + w * local_dir.z
}

fn orthonormal_basis(normal: Vec3) -> (Vec3, Vec3, Vec3) {
    let w = normal; // Normal is already normalized
    let a = if w.x.abs() > 0.9 { Vec3::Y } else { Vec3::X }; // Avoid parallel vectors
    let v = w.cross(a).normalize();
    let u = w.cross(v).normalize();
    (u, v, w)
}

pub mod texture {
    use glam::Vec4;
    use image::Rgba32FImage;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case", tag = "texture")]
    pub enum Texture {
        Solid {
            color: Vec4,
        },
        Checker {
            color1: Vec4,
            color2: Vec4,
            scale: f32,
        },
        Image {
            #[serde(
                deserialize_with = "crate::raytracer::loader::deserialize_image",
                serialize_with = "crate::raytracer::loader::serialize_image"
            )]
            image: Rgba32FImage,
        },
    }

    impl Texture {
        pub fn sample(&self, (u, v): (f32, f32)) -> Vec4 {
            match self {
                Texture::Solid { color } => *color,
                Texture::Checker {
                    color1,
                    color2,
                    scale,
                } => {
                    let x = (u / scale).floor() as i32;
                    let y = (v / scale).floor() as i32;

                    if (x + y) % 2 == 0 { *color1 } else { *color2 }
                }
                Texture::Image { image } => {
                    let pixel = image::imageops::sample_bilinear(image, u, 1.0 - v)
                        .unwrap_or([0.0, 0.0, 0.0, 0.0].into());
                    Vec4::new(pixel[0], pixel[1], pixel[2], pixel[3])
                }
            }
        }
    }
}
