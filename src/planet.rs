use bevy::prelude::*;
use bevy::render::mesh::*;
use noise::{NoiseFn, SuperSimplex};

#[derive(Component)]
pub struct PlanetFace {
    pub resolution: usize,
    pub direction: Vec3,
    pub noise_center: Vec3,
    pub noise_roughness: f32,
    pub scale: f32,
}

impl Default for PlanetFace {
    fn default() -> Self {
        Self {
            resolution: 12,
            direction: Vec3::Y,
            noise_center: Vec3::ZERO,
            noise_roughness: 1.0,
            scale: 1.0,
        }
    }
}

fn evaluate_with_noise(point: Vec3) -> Vec3 {
    let perlin = SuperSimplex::new();
    let elevation = (perlin.get([point.x as f64, point.y as f64, point.z as f64]) + 1.0) * 0.5;
    point * (1.0 + elevation) as f32
}

fn recompute_face_mesh(mesh: &mut Mesh, face: &PlanetFace) {
    if face.resolution < 2 {
        panic!("resolution must be >= 2")
    }
    let axis_a = Vec3::new(face.direction.y, face.direction.z, face.direction.x);
    let axis_b = face.direction.cross(axis_a);
    let mut indicies: Vec<u32> =
        Vec::with_capacity((face.resolution - 1) * (face.resolution - 1) * 6);
    let mut positions = Vec::with_capacity(face.resolution * face.resolution);
    let mut normals = Vec::with_capacity(face.resolution * face.resolution);
    let mut uvs = Vec::with_capacity(face.resolution * face.resolution);

    for y in 0..face.resolution {
        for x in 0..face.resolution {
            uvs.push([0.0, 0.0]); // TODO: what to do here

            // Generate vertex position
            let percent = Vec2::new(x as f32, y as f32) / (face.resolution - 1) as f32;
            let point_on_unit_cube = face.direction
                + (percent.x - 0.5) * 2.0 * axis_a
                + (percent.y - 0.5) * 2.0 * axis_b;
            let mut point_on_unit_sphere =
                point_on_unit_cube.normalize() * face.noise_roughness + face.noise_center;
            point_on_unit_sphere = evaluate_with_noise(point_on_unit_sphere);
            point_on_unit_sphere *= face.scale;
            positions.push(point_on_unit_sphere.to_array());

            normals.push(Vec3::ZERO);

            // generate index
            let i = (x + y * face.resolution) as u32;
            if x != (face.resolution - 1) && y != (face.resolution - 1) {
                indicies.push(i);
                indicies.push(i + face.resolution as u32 + 1);
                indicies.push(i + face.resolution as u32);
                indicies.push(i);
                indicies.push(i + 1);
                indicies.push(i + face.resolution as u32 + 1);
            }
        }
    }

    // Recalculate normals
    for chunk in indicies.chunks(3) {
        let p1 = Vec3::from(positions[chunk[0] as usize]);
        let p2 = Vec3::from(positions[chunk[1] as usize]);
        let p3 = Vec3::from(positions[chunk[2] as usize]);
        let u = p1 - p2;
        let v = p1 - p3;
        let norm = u.cross(v);
        normals[chunk[0] as usize] += norm;
        normals[chunk[1] as usize] += norm;
        normals[chunk[2] as usize] += norm;
    }

    let normals: Vec<[f32; 3]> = normals
        .iter()
        .map(|norm| norm.normalize().to_array())
        .collect();

    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indicies)));
}

pub fn make_planet_face_meshes(
    query: Query<(&PlanetFace, &Handle<Mesh>), Changed<PlanetFace>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (face, mesh) in query.iter() {
        if let Some(mesh) = meshes.get_mut(mesh) {
            recompute_face_mesh(mesh, face);
        }
    }
}
