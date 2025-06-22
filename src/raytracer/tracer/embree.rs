use crate::raytracer::tracer::TraceResult;
use crate::raytracer::world::{Geometry, GeometryType, Ray};
use embree4_rs::geometry::SphereGeometry;
use embree4_sys::{RTCRay, RTCRayHit};
use glam::Vec3;
use std::collections::Bound;
use std::f32::consts::PI;
use std::ops::RangeBounds;

pub struct EmbreeTracer {
    committed_scene: embree4_rs::CommittedScene<'static>,
    geometry: Vec<Geometry>,
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

                    let indices = [(0, 1, 2), (2, 3, 0)];

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
                        // Bottom face (no w component) - normal pointing down (-w direction)
                        (0, 2, 1), (0, 3, 2),
                        // Top face (w component) - normal pointing up (+w direction)
                        (4, 5, 6), (4, 6, 7),
                        // Left face (no u component) - normal pointing left (-u direction)
                        (0, 7, 3), (0, 4, 7),
                        // Right face (u component) - normal pointing right (+u direction)
                        (1, 2, 6), (1, 6, 5),
                        // Front face (no v component) - normal pointing forward (-v direction)
                        (0, 1, 5), (0, 5, 4),
                        // Back face (v component) - normal pointing backward (+v direction)
                        (3, 6, 2), (3, 7, 6),
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

        EmbreeTracer {
            committed_scene,
            geometry: geometry.to_vec(),
        }
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
            .map(|hit| self.convert_hit_to_trace_result(hit))
    }

    fn convert_hit_to_trace_result(&self, hit: RTCRayHit) -> TraceResult {
        let origin = Vec3::new(hit.ray.org_x, hit.ray.org_y, hit.ray.org_z);
        let dir = Vec3::new(hit.ray.dir_x, hit.ray.dir_y, hit.ray.dir_z).normalize();
        let point = origin + dir * hit.ray.tfar;

        let mut normal = Vec3::new(hit.hit.Ng_x, hit.hit.Ng_y, hit.hit.Ng_z).normalize();

        let front_face = dir.dot(normal) < 0.0;
        if !front_face {
            normal = -normal;
        }

        let geometry_index = hit.hit.geomID as usize;

        let uv = if geometry_index < self.geometry.len() {
            self.calculate_uv(&self.geometry[geometry_index].geometry_type, point, &hit)
        } else {
            (hit.hit.u, hit.hit.v)
        };

        TraceResult {
            distance: hit.ray.tfar,
            normal,
            front_face,
            geometry_index,
            point,
            uv,
        }
    }

    fn calculate_uv(&self, geometry: &GeometryType, point: Vec3, hit: &RTCRayHit) -> (f32, f32) {
        match geometry {
            // UV from spheres are broken in embree, so we calculate them manually
            GeometryType::Sphere { center, .. } => {
                let sphere_point = (point - center).normalize();
                let theta = (-sphere_point.y).acos();
                let phi = (-sphere_point.z).atan2(sphere_point.x) + PI;
                let u = phi / (2.0 * PI);
                let v = theta / PI;
                (u, v)
            }
            // For quads, calculate UV based on position within the quad
            GeometryType::Quad { origin, u, v } => {
                let vector_in_plane = point - origin;
                let u_coord = vector_in_plane.dot(*u) / u.length_squared();
                let v_coord = vector_in_plane.dot(*v) / v.length_squared();
                (u_coord, v_coord)
            }
            // For triangle meshes, interpolate texture coordinates using barycentric coordinates
            GeometryType::TriangleMesh(mesh) => {
                let prim_id = hit.hit.primID as usize;
                if prim_id < mesh.indices.len() && !mesh.tex_coords.is_empty() {
                    let (i0, i1, i2) = mesh.indices[prim_id];
                    let uv0 = mesh.tex_coords[i0 as usize];
                    let uv1 = mesh.tex_coords[i1 as usize];
                    let uv2 = mesh.tex_coords[i2 as usize];
                    
                    // Interpolate using barycentric coordinates
                    // hit.hit.u and hit.hit.v are barycentric coordinates for the triangle
                    let w = 1.0 - hit.hit.u - hit.hit.v; // barycentric coordinate for vertex 0
                    let interpolated_uv = w * uv0 + hit.hit.u * uv1 + hit.hit.v * uv2;
                    (interpolated_uv.x, interpolated_uv.y)
                } else {
                    // Fallback to barycentric coordinates if no texture coordinates available
                    (hit.hit.u, hit.hit.v)
                }
            }
            // For boxes, calculate UV based on which face was hit and local coordinates
            GeometryType::Box { origin, u, v, w } => {
                // Transform hit point to local box coordinates
                let ray_hit_local = point - origin;
                
                // Calculate the inverse transformation matrix for the box
                let det = u.dot(v.cross(*w));
                if det.abs() < 1e-8 {
                    return (0.0, 0.0); // Degenerate box
                }
                
                let inv_det = 1.0 / det;
                let local_hit = Vec3::new(
                    ray_hit_local.dot(v.cross(*w)) * inv_det,
                    ray_hit_local.dot(w.cross(*u)) * inv_det,
                    ray_hit_local.dot(u.cross(*v)) * inv_det,
                );
                
                // Determine which face we hit by finding the coordinate closest to 0 or 1
                let distances_to_faces = [
                    local_hit.x.abs(),           // Left face (x = 0)
                    (local_hit.x - 1.0).abs(),   // Right face (x = 1)
                    local_hit.y.abs(),           // Bottom face (y = 0)
                    (local_hit.y - 1.0).abs(),   // Top face (y = 1)
                    local_hit.z.abs(),           // Front face (z = 0)
                    (local_hit.z - 1.0).abs(),   // Back face (z = 1)
                ];
                
                let closest_face = distances_to_faces
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(index, _)| index)
                    .unwrap_or(0);
                
                // Map local coordinates to UV based on the closest face
                match closest_face {
                    0 | 1 => (1.0 - local_hit.z, local_hit.y), // X faces - use Z and flipped Y
                    2 | 3 => (1.0 - local_hit.x, local_hit.z), // Y faces - use X and flipped Z
                    _ => (1.0 - local_hit.x, local_hit.y),     // Z faces - use X and flipped Y
                }
            }
        }
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
