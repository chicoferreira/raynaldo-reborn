pub mod camera;
pub mod environment;
pub mod light;
pub mod loader;
pub mod material;
pub mod tonemap;
pub mod tracer;
pub mod world;

use crate::raytracer::camera::Camera;
use crate::raytracer::environment::Environment;
use crate::raytracer::light::LightSampler;
use crate::raytracer::tonemap::Tonemapper;
use crate::raytracer::tracer::embree::EmbreeTracer;
use crate::raytracer::tracer::naive::NaiveTracer;
use crate::raytracer::world::{Ray, World};
use glam::Vec4;
use rand::rng;

pub struct Scene {
    pub camera: Camera,
    pub tracer: tracer::Tracer,
    pub world: World,
    pub light_sampler: LightSampler,
    pub tonemapper: Tonemapper,
    pub environment: Environment,
}

impl Scene {
    pub fn new(
        camera: Camera,
        world: World,
        environment: Environment,
        tracer_type: crate::TracerType,
    ) -> Self {
        let tracer = match tracer_type {
            crate::TracerType::Naive => {
                tracer::Tracer::NaiveTracer(NaiveTracer::new(&world.geometry))
            }
            crate::TracerType::Embree => {
                tracer::Tracer::EmbreeTracer(EmbreeTracer::new(&world.geometry))
            }
        };

        let light_sampler = LightSampler::new(&world.geometry);

        Self {
            camera,
            tracer,
            world,
            environment,
            tonemapper: Tonemapper::None,
            light_sampler,
        }
    }

    pub fn update_screen_size(&mut self, image_width: u32, image_height: u32) {
        self.camera.image_width = image_width;
        self.camera.image_height = image_height;
        self.camera.update_pixel_constants();
    }

    #[allow(dead_code)]
    pub fn render_pixel(&self, x: u32, y: u32, samples_per_pixel: u32, max_depth: u32) -> Vec4 {
        let mut color = Vec4::ZERO;
        for _ in 0..samples_per_pixel {
            let ray = self.camera.generate_ray(x, y, &mut rng());
            color += self.render_ray(ray, max_depth);
        }
        color / samples_per_pixel as f32
    }

    pub fn render_sample(&self, x: u32, y: u32, max_depth: u32) -> Vec4 {
        let ray = self.camera.generate_ray(x, y, &mut rng());
        self.render_ray(ray, max_depth)
    }

    fn render_ray(&self, mut ray: Ray, max_depth: u32) -> Vec4 {
        let mut final_color = Vec4::ZERO;
        let mut throughput = Vec4::ONE;

        for _ in 0..max_depth {
            if let Some(trace_result) = self.tracer.trace(&ray, &(0.0001..)) {
                let geometry = &self.world.geometry[trace_result.geometry_index];
                let material = &geometry.material;

                // Add emission
                final_color += throughput * material.emit(&trace_result);

                // Direct lighting calculation
                if let Some(light) = self.light_sampler.sample() {
                    let light_geometry = &self.world.geometry[light.geometry_index];

                    let (random_point, random_normal) =
                        light_geometry.geometry_type.sample_random_point();

                    let light_direction = random_point - trace_result.point;
                    let light_distance = light_direction.length();
                    let light_direction = light_direction.normalize();

                    let shadow_ray = Ray::new(
                        trace_result.point + light_direction * 0.001,
                        light_direction,
                    );

                    if self
                        .tracer
                        .trace(&shadow_ray, &(0.01..light_distance - 0.01))
                        .is_none()
                    {
                        let diffuse = material
                            .scatter(&ray, &trace_result)
                            .map_or(Vec4::ZERO, |s| s.attenuation);

                        let cos_alpha = light_direction.dot(trace_result.normal);
                        let cos_beta = -light_direction.dot(random_normal);

                        // pdf_select_light = (intensity * area) / sum(intensity*area)
                        // pdf_point_on_light = pdf_select_light * (1.0 / area)
                        let pdf = light.pdf * (1.0 / light.light_area);

                        let light_contribution =
                            diffuse * cos_alpha * cos_beta / (pdf * light_distance.powi(2));

                        final_color += throughput * light_contribution;
                    }
                }

                // Indirect lighting (material scattering)
                if let Some(scatter_result) = material.scatter(&ray, &trace_result) {
                    ray = scatter_result.scattered;
                    throughput *= scatter_result.attenuation;

                    // Russian roulette for path termination
                    let max_component = throughput.x.max(throughput.y).max(throughput.z);
                    if max_component < 0.1 {
                        let survival_probability = max_component;
                        if rand::random::<f32>() > survival_probability {
                            break;
                        }
                        throughput /= survival_probability;
                    }
                } else {
                    break;
                }
            } else {
                final_color += throughput * self.environment.get_environment_color(&ray);
                break;
            }
        }

        final_color
    }
}
