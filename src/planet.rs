use bevy::prelude::*;
use bevy::render::mesh::*;
use noise::{NoiseFn, SuperSimplex};

#[derive(Component)]
pub struct PlanetFace {
    pub resolution: usize,
    pub direction: Vec3,
    pub scale: f32,
    pub noise_center: Vec3,
    pub noise_strength: f32,
    pub noise_roughness: f32,
    pub noise_layers: usize,
    pub noise_persistance: f32,
    pub noise_base_roughness: f32,
}

impl Default for PlanetFace {
    fn default() -> Self {
        Self {
            resolution: 20,
            direction: Vec3::Y,
            scale: 1.75,
            noise_center: Vec3::ZERO,
            noise_strength: 0.51,
            noise_roughness: 2.31,
            noise_base_roughness: 0.91,
            noise_persistance: 0.55,
            noise_layers: 5,
        }
    }
}

fn evaluate_with_noise(face: &PlanetFace, point: Vec3) -> Vec3 {
    let noise = SuperSimplex::new();
    let mut noise_value = 0.0;
    let mut frequency = face.noise_base_roughness;
    let mut amplitude: f32 = 1.0;

    for _ in 0..face.noise_layers {
        let p = point * frequency + face.noise_center;
        let v = noise.get([p.x as f64, p.y as f64, p.z as f64]);
        noise_value += (v + 1.0) * 0.5 * amplitude as f64;
        frequency *= face.noise_roughness;
        amplitude *= face.noise_persistance;
    }
    let elevation = noise_value as f32 * face.noise_strength;
    point * (1.0 + elevation)
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
            let point_on_unit_sphere =
                evaluate_with_noise(&face, point_on_unit_cube.normalize()) * face.scale;
            positions.push(point_on_unit_sphere.to_array());

            // Add zero normal to be updated later
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
