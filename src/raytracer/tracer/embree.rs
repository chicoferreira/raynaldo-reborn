use crate::raytracer::tracer::TraceResult;
use crate::raytracer::world::{Geometry, GeometryType, Ray};
use embree4_rs::geometry::SphereGeometry;
use embree4_sys::{RTCRay, RTCRayHit};
use glam::Vec3;
use std::collections::Bound;
use std::ops::RangeBounds;

pub struct EmbreeTracer {
    committed_scene: embree4_rs::CommittedScene<'static>,
}

impl EmbreeTracer {
    pub fn new(geometry: &[Geometry]) -> EmbreeTracer {
        let device = embree4_rs::Device::try_new(None).expect("Failed to create Embree device");
        let device = Box::leak(Box::new(device));

        let scene = embree4_rs::Scene::try_new(
            device,
            embree4_rs::SceneOptions {
                build_quality: embree4_sys::RTCBuildQuality::HIGH,
                flags: embree4_sys::RTCSceneFlags::ROBUST,
            },
        )
        .expect("Failed to create Embree scene");
        let scene = Box::leak(Box::new(scene));

        for geom in geometry {
            match &geom.geometry_type {
                GeometryType::Sphere { center, radius } => {
                    let embree_geom =
                        SphereGeometry::try_new(device, (center.x, center.y, center.z), *radius)
                            .expect("Failed to create sphere geometry");

                    scene
                        .attach_geometry(&embree_geom)
                        .expect("Failed to attach sphere geometry");
                }
                GeometryType::Quad { origin, u, v } => {
                    let vertices = [
                        origin.clone().into(),
                        (origin + u).into(),
                        (origin + u + v).into(),
                        (origin + v).into(),
                    ];

                    let indices = [(2, 1, 0), (0, 3, 2)];

                    let embree_geom = embree4_rs::geometry::TriangleMeshGeometry::try_new(
                        device, &vertices, &indices,
                    )
                    .expect("Failed to create quad geometry");

                    scene
                        .attach_geometry(&embree_geom)
                        .expect("Failed to attach quad geometry");
                }
                GeometryType::TriangleMesh(mesh) => {
                    let embree_geom = embree4_rs::geometry::TriangleMeshGeometry::try_new(
                        device,
                        &mesh.verts,
                        &mesh.indices,
                    )
                    .expect("Failed to create triangle mesh geometry");

                    scene
                        .attach_geometry(&embree_geom)
                        .expect("Failed to attach triangle mesh geometry");
                }
                GeometryType::Box { origin, u, v, w } => {
                    // Convert oriented box to triangle mesh
                    // Define 8 vertices of the oriented box
                    let vertices = [
                        (*origin).into(),                // 0: origin
                        (*origin + *u).into(),           // 1: origin + u
                        (*origin + *u + *v).into(),      // 2: origin + u + v
                        (*origin + *v).into(),           // 3: origin + v
                        (*origin + *w).into(),           // 4: origin + w
                        (*origin + *u + *w).into(),      // 5: origin + u + w
                        (*origin + *u + *v + *w).into(), // 6: origin + u + v + w
                        (*origin + *v + *w).into(),      // 7: origin + v + w
                    ];

                    // Define 12 triangles (2 per face, 6 faces)
                    #[rustfmt::skip]
                    let indices = [
                        // Bottom face (no w component) - normal pointing down (-v direction)
                        (0, 1, 2), (0, 2, 3),
                        // Top face (w component) - normal pointing up (+v direction)
                        (4, 7, 6), (4, 6, 5),
                        // Left face (no u component) - normal pointing left (-u direction)
                        (0, 3, 7), (0, 7, 4),
                        // Right face (u component) - normal pointing right (+u direction)
                        (1, 5, 6), (1, 6, 2),
                        // Front face (no v component) - normal pointing down (-v direction)
                        (0, 4, 5), (0, 5, 1),
                        // Back face (v component) - normal pointing up (+v direction)
                        (3, 2, 6), (3, 6, 7),
                    ];

                    let embree_geom = embree4_rs::geometry::TriangleMeshGeometry::try_new(
                        device, &vertices, &indices,
                    )
                    .expect("Failed to create box geometry");

                    scene
                        .attach_geometry(&embree_geom)
                        .expect("Failed to attach box geometry");
                }
            }
        }

        let committed_scene = scene.commit().expect("Failed to commit scene");

        EmbreeTracer { committed_scene }
    }

    pub fn trace(&self, ray: &Ray, ray_bounds: &impl RangeBounds<f32>) -> Option<TraceResult> {
        let tnear = match ray_bounds.start_bound() {
            Bound::Included(&v) => v,
            Bound::Excluded(&v) => v,
            Bound::Unbounded => 0.0,
        };

        let tfar = match ray_bounds.end_bound() {
            Bound::Included(&v) => v,
            Bound::Excluded(&v) => v,
            Bound::Unbounded => f32::INFINITY,
        };

        self.committed_scene
            .intersect_1(RTCRay {
                org_x: ray.origin.x,
                org_y: ray.origin.y,
                org_z: ray.origin.z,
                dir_x: ray.direction.x,
                dir_y: ray.direction.y,
                dir_z: ray.direction.z,
                tnear,
                tfar,
                ..Default::default()
            })
            .expect("Device error while intersecting ray")
            .map(Into::into)
    }
}

impl From<Ray> for RTCRay {
    fn from(value: Ray) -> Self {
        RTCRay {
            org_x: value.origin.x,
            org_y: value.origin.y,
            org_z: value.origin.z,
            dir_x: value.direction.x,
            dir_y: value.direction.y,
            dir_z: value.direction.z,
            ..Default::default()
        }
    }
}

impl From<RTCRayHit> for TraceResult {
    fn from(value: RTCRayHit) -> Self {
        let origin = Vec3::new(value.ray.org_x, value.ray.org_y, value.ray.org_z);
        let dir = Vec3::new(value.ray.dir_x, value.ray.dir_y, value.ray.dir_z).normalize();
        let point = origin + dir * value.ray.tfar;

        let mut normal = Vec3::new(value.hit.Ng_x, value.hit.Ng_y, value.hit.Ng_z).normalize();

        let front_face = dir.dot(normal) < 0.0;
        if !front_face {
            normal = -normal;
        }

        TraceResult {
            distance: value.ray.tfar,
            normal,
            front_face,
            geometry_index: value.hit.geomID as usize,
            point,
            uv: (value.hit.u, value.hit.v),
        }
    }
}
