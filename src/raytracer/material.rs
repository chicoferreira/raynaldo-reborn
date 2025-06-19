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
}

impl MaterialType {
    pub fn scatter(&self, ray: &Ray, trace_result: &TraceResult) -> Option<ScatterResult> {
        match self {
            MaterialType::Lambertian { texture } => {
                let mut scatter_dir = trace_result.normal + random_unit_vector();

                if scatter_dir.x < 1e-8 && scatter_dir.y < 1e-8 && scatter_dir.z < 1e-8 {
                    scatter_dir = trace_result.normal;
                }

                Some(ScatterResult {
                    attenuation: texture.sample(trace_result.uv),
                    scattered: Ray::new(trace_result.point, scatter_dir),
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
        let length = random_vec.length_squared();
        if length <= 1.0 && length > 0.1 {
            return random_vec / length.sqrt();
        }
    }
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
                        .expect("UV is inbounds");
                    Vec4::new(pixel[0], pixel[1], pixel[2], pixel[3])
                }
            }
        }
    }
}
