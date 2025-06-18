use crate::app::CameraSettings;
use crate::raytracer::material::texture::Texture;
use crate::raytracer::material::MaterialType;
use crate::raytracer::world::GeometryType::{Quad, Sphere};
use crate::raytracer::world::{Geometry, World};
use glam::Vec3;

mod app;
mod raytracer;

fn main() {
    let mut geometry = vec![];

    let ground_material = MaterialType::Lambertian {
        texture: Texture::Solid {
            color: glam::vec4(0.5, 0.5, 0.5, 1.0),
        },
    };

    geometry.push(Geometry {
        geometry_type: Quad {
            origin: Vec3::new(-100.0, 0.0, -100.0),
            u: Vec3::new(200.0, 0.0, 0.0),
            v: Vec3::new(0.0, 0.0, 200.0),
        },
        material: ground_material,
    });

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rand::random::<f32>();
            let center = Vec3::new(
                a as f32 + 0.9 * rand::random::<f32>(),
                0.2,
                b as f32 + 0.9 * rand::random::<f32>(),
            );

            if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let sphere_material: MaterialType;

                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = glam::vec4(
                        rand::random::<f32>(),
                        rand::random::<f32>(),
                        rand::random::<f32>(),
                        1.0,
                    ) * glam::vec4(
                        rand::random::<f32>(),
                        rand::random::<f32>(),
                        rand::random::<f32>(),
                        1.0,
                    );
                    sphere_material = MaterialType::Lambertian {
                        texture: Texture::Solid { color: albedo },
                    };
                    geometry.push(Geometry {
                        geometry_type: Sphere {
                            center,
                            radius: 0.2,
                        },
                        material: sphere_material,
                    });
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = glam::vec4(
                        rand::random::<f32>() * 0.5 + 0.5,
                        rand::random::<f32>() * 0.5 + 0.5,
                        rand::random::<f32>() * 0.5 + 0.5,
                        1.0,
                    );
                    let fuzz = rand::random::<f32>() * 0.5;
                    sphere_material = MaterialType::Metal {
                        albedo,
                        fuzziness: fuzz,
                    };
                    geometry.push(Geometry {
                        geometry_type: Sphere {
                            center,
                            radius: 0.2,
                        },
                        material: sphere_material,
                    });
                } else {
                    // glass
                    sphere_material = MaterialType::Dielectric {
                        refractive_index: 1.5,
                    };
                    geometry.push(Geometry {
                        geometry_type: Sphere {
                            center,
                            radius: 0.2,
                        },
                        material: sphere_material,
                    });
                }
            }
        }
    }

    let material1 = MaterialType::Dielectric {
        refractive_index: 1.5,
    };
    geometry.push(Geometry {
        geometry_type: Sphere {
            center: Vec3::new(0.0, 1.0, 0.0),
            radius: 1.0,
        },
        material: material1,
    });

    let material2 = MaterialType::Lambertian {
        texture: Texture::Checker {
            color1: glam::vec4(0.2, 0.3, 0.1, 1.0),
            color2: glam::vec4(0.9, 0.9, 0.9, 1.0),
            scale: 10.0,
        },
    };
    geometry.push(Geometry {
        geometry_type: Sphere {
            center: Vec3::new(-4.0, 1.0, 0.0),
            radius: 1.0,
        },
        material: material2,
    });
    let material3 = MaterialType::Metal {
        albedo: glam::vec4(0.7, 0.6, 0.5, 1.0),
        fuzziness: 0.0,
    };
    geometry.push(Geometry {
        geometry_type: Sphere {
            center: Vec3::new(4.0, 1.0, 0.0),
            radius: 1.0,
        },
        material: material3,
    });

    let camera = CameraSettings {
        position: Vec3::new(-13.0, 2.0, 3.0),
        yaw: 0.0,
        pitch: 0.0,
        fov: 60.0,
        focus_distance: 10.0,
        defocus_angle: 0.6,
    };

    app::run(World { geometry }, camera);
}
