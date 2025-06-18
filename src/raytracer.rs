pub mod camera;
pub mod material;
pub mod tracer;
pub mod world;

use crate::raytracer::camera::Camera;
use crate::raytracer::tracer::embree::EmbreeRayTracer;
use crate::raytracer::tracer::naive::NaiveTracer;
use crate::raytracer::world::{Ray, World};
use glam::{vec4, Vec4};
use rand::rng;

pub struct Scene {
    pub camera: Camera,
    pub tracer: tracer::TracerType,
    pub world: World,
}

impl Scene {
    pub fn new(camera: Camera, world: World, tracer_type: crate::TracerTypeArg) -> Self {
        let tracer = match tracer_type {
            crate::TracerTypeArg::Naive => {
                tracer::TracerType::NaiveTracer(NaiveTracer::new(&world.geometry))
            }
            crate::TracerTypeArg::Embree => {
                tracer::TracerType::EmbreeTracer(EmbreeRayTracer::new(&world.geometry))
            }
        };

        Self {
            camera,
            tracer,
            world,
        }
    }

    pub fn update_screen_size(&mut self, image_width: u32, image_height: u32) {
        self.camera.image_width = image_width;
        self.camera.image_height = image_height;
        self.camera.update_pixel_constants();
    }

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
        if max_depth == 0 {
            return Vec4::ZERO;
        }

        let mut color = Vec4::ONE;

        for _ in 0..max_depth {
            if let Some(result) = self.tracer.trace(&ray, &(0.0001..)) {
                let geometry = &self.world.geometry[result.geometry_index];
                let material = &geometry.material;

                if let Some(scatter_result) = material.scatter(&ray, &result) {
                    ray = scatter_result.scattered;
                    color *= scatter_result.attenuation;
                } else {
                    return Vec4::ZERO;
                }
            } else {
                return color * self.get_environment_color(&ray);
            }
        }

        Vec4::ZERO
    }

    fn get_environment_color(&self, ray: &Ray) -> Vec4 {
        let t = 0.5 * (ray.direction.normalize().y + 1.0);
        vec4(1.0, 1.0, 1.0, 1.0) * (1.0 - t) + vec4(0.5, 0.7, 1.0, 1.0) * t
    }
}
