use glam::{Vec3, Vec4, vec4};
use serde::{Deserialize, Serialize};

use crate::raytracer::world::Ray;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Environment {
    Color { color: Vec3 },
    #[default]
    Sky,
}

impl Environment {
    pub fn get_environment_color(&self, ray: &Ray) -> Vec4 {
        match self {
            Environment::Color {
                color: environment_color,
            } => environment_color.extend(1.0),
            Environment::Sky => {
                let t = 0.5 * (ray.direction.normalize().y + 1.0);
                vec4(1.0, 1.0, 1.0, 1.0) * (1.0 - t) + vec4(0.5, 0.7, 1.0, 1.0) * t
            }
        }
    }
}
