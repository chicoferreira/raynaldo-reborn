use crate::raytracer::{material::MaterialType, world::Geometry};

pub struct LightSampler {
    lights: Vec<LightCdf>,
}

#[derive(Clone, Copy)]
pub struct LightCdf {
    pub geometry_index: usize,
    cdf: f32,
}

impl LightSampler {
    pub fn new(world: &[Geometry]) -> Self {
        let mut lights = Vec::new();

        for (index, geometry) in world.iter().enumerate() {
            let material = &geometry.material;
            let MaterialType::Emissive { intensity, .. } = material else {
                continue;
            };

            let area = match &geometry.geometry_type {
                crate::raytracer::world::GeometryType::Sphere { radius, .. } => {
                    4.0 * std::f32::consts::PI * radius.powi(2)
                }
                crate::raytracer::world::GeometryType::Quad { u, v, .. } => u.length() * v.length(),
                crate::raytracer::world::GeometryType::TriangleMesh(_mesh) => {
                    todo!("Area for triangle mesh not implemented yet")
                }
                crate::raytracer::world::GeometryType::Box { u, v, w, .. } => {
                    2.0 * (u.length() * v.length()
                        + u.length() * w.length()
                        + v.length() * w.length())
                }
            };

            let pdf = intensity * area;

            lights.push(LightCdf {
                geometry_index: index,
                // Keep PDF for now, will be converted to CDF later
                cdf: pdf,
            });
        }

        // Normalize PDFs
        let sum = lights.iter().map(|light| light.cdf).sum::<f32>();
        for light in &mut lights {
            light.cdf /= sum;
        }

        // Convert PDFs to CDFs
        for i in 1..lights.len() {
            lights[i].cdf += lights[i - 1].cdf;
        }

        LightSampler { lights }
    }

    pub fn sample(&self) -> Option<LightCdf> {
        if self.lights.is_empty() {
            return None;
        }

        let random_value: f32 = rand::random_range(0.0..1.0);
        let mut low = 0;
        let mut high = self.lights.len() - 1;

        while low < high {
            let mid = (low + high) / 2;
            if self.lights[mid].cdf < random_value {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        Some(self.lights[low])
    }
}
