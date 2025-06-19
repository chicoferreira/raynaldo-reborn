use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::raytracer::{camera::Camera, world::TriangleMeshGeometry};

pub fn deserialize_triangle_mesh<'de, D>(deserializer: D) -> Result<TriangleMeshGeometry, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let settings = TriangleMeshGeometrySettings::deserialize(deserializer)?;
    settings.try_into().map_err(serde::de::Error::custom)
}

/// Only for debug purposes
pub fn serialize_triangle_mesh<S>(
    _mesh: &TriangleMeshGeometry,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let path = "obj.obj";
    serializer.serialize_str(path)
}

pub fn deserialize_image<'de, D>(deserializer: D) -> Result<image::Rgba32FImage, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let path: PathBuf = Deserialize::deserialize(deserializer)?;
    Ok(image::open(path)
        .map_err(serde::de::Error::custom)?
        .into_rgba32f())
}

/// Only for debug purposes
pub fn serialize_image<S>(_image: &image::Rgba32FImage, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let path = "temp_image.png"; // Temporary path for serialization
    serializer.serialize_str(path)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSettings {
    pub position: (f32, f32, f32),
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub focus_distance: f32,
    pub defocus_angle: f32,
}

impl CameraSettings {
    pub fn to_camera(&self, width: u32, height: u32, sensibility: f32) -> Camera {
        Camera::new(
            width,
            height,
            self.fov,
            self.position.into(),
            self.yaw,
            self.pitch,
            sensibility,
            self.focus_distance,
            self.defocus_angle,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "mesh_type")]
pub enum TriangleMeshGeometrySettings {
    ObjFile {
        path: PathBuf,
    },
    Implicit {
        verts: Vec<(f32, f32, f32)>,
        indices: Vec<(u32, u32, u32)>,
        tex_coords: Vec<(f32, f32)>,
    },
}

impl TryInto<TriangleMeshGeometry> for TriangleMeshGeometrySettings {
    type Error = tobj::LoadError;

    fn try_into(self) -> Result<TriangleMeshGeometry, Self::Error> {
        Ok(match self {
            TriangleMeshGeometrySettings::ObjFile { path } => {
                let (models, _materials) = tobj::load_obj(path, &tobj::LoadOptions::default())?;
                let model = models.get(0).expect("obj has no models");

                TriangleMeshGeometry {
                    verts: model
                        .mesh
                        .positions
                        .chunks_exact(3)
                        .map(|chunk| (chunk[0], chunk[1], chunk[2]))
                        .collect(),
                    indices: model
                        .mesh
                        .indices
                        .chunks_exact(3)
                        .map(|chunk| (chunk[0], chunk[1], chunk[2]))
                        .collect(),
                    tex_coords: model
                        .mesh
                        .texcoords
                        .chunks_exact(2)
                        .map(|chunk| glam::vec2(chunk[0], chunk[1]))
                        .collect(),
                }
            }
            TriangleMeshGeometrySettings::Implicit {
                verts,
                indices,
                tex_coords,
            } => TriangleMeshGeometry {
                verts,
                indices,
                tex_coords: tex_coords.into_iter().map(Into::into).collect(),
            },
        })
    }
}
