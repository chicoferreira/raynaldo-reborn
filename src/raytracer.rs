use crate::raytracer::tracer::Tracer;
pub mod camera;
pub mod material;
pub mod tracer;
pub mod world;

use crate::raytracer::camera::Camera;
use crate::raytracer::material::Material;
use crate::raytracer::world::{Ray, World};
use glam::{vec4, Vec4};
use rand::rng;
use tracer::naive::NaiveTracer;

pub struct Scene {
    pub camera: Camera,
    pub tracer: tracer::TracerType,
    pub world: World,
}

impl Scene {
    pub fn new(camera: Camera, world: World) -> Self {
        let geometry_types: Vec<_> = world
            .geometry
            .iter()
            .map(|g| g.geometry_type.clone())
            .collect();

        Self {
            camera,
            tracer: tracer::TracerType::NaiveTracer(NaiveTracer::new(&geometry_types)),
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
            color += self.render_ray(&ray, max_depth);
        }
        color / samples_per_pixel as f32
    }

    fn render_ray(&self, ray: &Ray, max_depth: u32) -> Vec4 {
        if max_depth == 0 {
            return Vec4::ZERO;
        }

        if let Some(result) = self.tracer.trace(ray, &(0.0001..)) {
            let geometry = &self.world.geometry[result.geometry_index];
            let material = &geometry.material;

            if let Some(scatter_result) = material.scatter(ray, &result) {
                let scattered = scatter_result.scattered;
                let attenuation = scatter_result.attenuation;
                return attenuation * self.render_ray(&scattered, max_depth - 1);
            }

            return Vec4::ZERO;
        }
        let t = 0.5 * (ray.direction.normalize().y + 1.0);
        let color = vec4(1.0, 1.0, 1.0, 1.0) * (1.0 - t) + vec4(0.5, 0.7, 1.0, 1.0) * t;
        color
    }
}
