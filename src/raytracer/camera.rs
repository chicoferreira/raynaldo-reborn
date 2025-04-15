use crate::raytracer::world::Ray;
use glam::{Vec2, Vec3};
use rand::Rng;

#[derive(Clone)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub image_width: u32,
    pub image_height: u32,
    pub camera_sensibility: f32,
    pub defocus_angle: f32,
    pub focus_distance: f32,
    pixel_00_loc: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn update_pixel_constants(&mut self) {
        *self = Self::new(
            self.image_width,
            self.image_height,
            self.fov,
            self.position,
            self.yaw,
            self.pitch,
            self.camera_sensibility,
            self.focus_distance,
            self.defocus_angle,
        )
    }

    pub fn new(
        image_width: u32,
        image_height: u32,
        fov: f32,
        position: Vec3,
        yaw: f32,
        pitch: f32,
        camera_sensibility: f32,
        focus_distance: f32,
        defocus_angle: f32,
    ) -> Self {
        let yaw_rad = yaw.to_radians();
        let pitch_rad = pitch.to_radians();

        let direction = Vec3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        );

        let viewport_height = 2.0 * (fov.to_radians() / 2.0).tan() * focus_distance;
        let viewport_width = viewport_height * (image_width as f32 / image_height as f32);

        let right = direction.cross(Vec3::Y);
        let up = direction.cross(right);

        let viewport_u = viewport_width * right;
        let viewport_v = viewport_height * -up;

        let pixel_delta_u = viewport_u / image_width as f32;
        let pixel_delta_v = viewport_v / image_height as f32;

        let viewport_upper_left =
            position + (focus_distance * direction) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel_00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let defocus_radius = focus_distance * (defocus_angle / 2.0).to_radians().tan();

        let defocus_disk_u = defocus_radius * -right;
        let defocus_disk_v = defocus_radius * -up;

        Self {
            position,
            yaw,
            pitch,
            image_width,
            image_height,
            fov,
            pixel_00_loc,
            pixel_delta_u,
            pixel_delta_v,
            camera_sensibility,
            defocus_angle,
            focus_distance,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    pub fn process_input(
        &mut self,
        right_input: f32,
        up_input: f32,
        forward_input: f32,
        yaw_input: f32,
        pitch_input: f32,
        delta_time_seconds: f32,
    ) {
        self.yaw += yaw_input * self.camera_sensibility * delta_time_seconds;
        self.pitch -= pitch_input * self.camera_sensibility * delta_time_seconds;

        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        let forward = Vec3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        );

        let left = Vec3::Y.cross(forward);
        let up = forward.cross(left);

        self.position += (right_input * -left + up_input * up + forward_input * forward)
            .normalize_or_zero()
            * delta_time_seconds;
        self.update_pixel_constants();
    }

    pub fn generate_ray(&self, x: u32, y: u32, random: &mut impl Rng) -> Ray {
        let pixel_center = self.pixel_00_loc
            + ((x as f32 + random.random_range(-0.5..0.5)) * self.pixel_delta_u)
            + ((y as f32 + random.random_range(-0.5..0.5)) * self.pixel_delta_v);

        let ray_origin = if self.defocus_angle > 0.0 {
            let random_disk = loop {
                let random_vec: Vec2 = rand::random();
                if random_vec.length_squared() < 1.0 {
                    break random_vec;
                }
            };

            self.position
                + random_disk.x * self.defocus_disk_u
                + random_disk.y * self.defocus_disk_v
        } else {
            self.position
        };

        let ray_direction = (pixel_center - ray_origin).normalize();

        Ray::new(ray_origin, ray_direction)
    }
}
