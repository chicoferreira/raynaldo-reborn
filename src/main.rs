use crate::app::CameraSettings;
use crate::raytracer::material::texture::Texture;
use crate::raytracer::material::{Lambertian, MaterialType};
use crate::raytracer::world::{Geometry, GeometryType, World};
use glam::{Vec3, Vec4};

mod app;
mod raytracer;

fn main() {
    let left_red = MaterialType::Lambertian(Lambertian {
        texture: Texture::Solid {
            color: Vec4::new(1.0, 0.2, 0.2, 1.0),
        },
    });
    let back_green = MaterialType::Lambertian(Lambertian {
        texture: Texture::Checker {
            color1: Vec4::new(0.2, 1.0, 0.2, 1.0),
            color2: Vec4::new(1.0, 1.0, 1.0, 1.0),
            scale: 0.5,
        },
    });
    let right_blue = MaterialType::Lambertian(Lambertian {
        texture: Texture::Solid {
            color: Vec4::new(0.2, 0.2, 1.0, 1.0),
        },
    });
    let upper_orange = MaterialType::Lambertian(Lambertian {
        texture: Texture::Solid {
            color: Vec4::new(1.0, 0.5, 0.0, 1.0),
        },
    });
    let lower_teal = MaterialType::Lambertian(Lambertian {
        texture: Texture::Solid {
            color: Vec4::new(0.2, 0.8, 0.8, 1.0),
        },
    });

    let mut geometry = vec![];
    geometry.push(Geometry {
        geometry_type: GeometryType::Quad {
            origin: Vec3::new(-3.0, -2.0, 5.0),
            u: Vec3::new(0.0, 0.0, -4.0),
            v: Vec3::new(0.0, 4.0, 0.0),
        },
        material: left_red,
    });

    geometry.push(Geometry {
        geometry_type: GeometryType::Quad {
            origin: Vec3::new(-2.0, -2.0, 0.0),
            u: Vec3::new(4.0, 0.0, 0.0),
            v: Vec3::new(0.0, 4.0, 0.0),
        },
        material: back_green,
    });

    geometry.push(Geometry {
        geometry_type: GeometryType::Quad {
            origin: Vec3::new(3.0, -2.0, 1.0),
            u: Vec3::new(0.0, 0.0, 4.0),
            v: Vec3::new(0.0, 4.0, 0.0),
        },
        material: right_blue,
    });

    geometry.push(Geometry {
        geometry_type: GeometryType::Quad {
            origin: Vec3::new(-2.0, 3.0, 1.0),
            u: Vec3::new(4.0, 0.0, 0.0),
            v: Vec3::new(0.0, 0.0, 4.0),
        },
        material: upper_orange,
    });

    geometry.push(Geometry {
        geometry_type: GeometryType::Quad {
            origin: Vec3::new(-2.0, -3.0, 5.0),
            u: Vec3::new(4.0, 0.0, 0.0),
            v: Vec3::new(0.0, 0.0, -4.0),
        },
        material: lower_teal,
    });

    let camera = CameraSettings {
        position: Vec3::new(0.0, 0.0, 9.0),
        yaw: -90.0,
        pitch: 0.0,
        fov: 80.0,
        focus_distance: 10.0,
        defocus_angle: 0.0,
        sensibility: 2.0,
    };

    app::run(World { geometry }, camera);
}
