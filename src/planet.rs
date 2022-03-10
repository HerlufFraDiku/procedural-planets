use bevy::prelude::*;
use bevy::render::mesh::*;

#[derive(Component)]
pub struct PlanetFace {
    pub resolution: usize,
    pub direction: Vec3,
}

impl PlanetFace {
    pub fn new(resolution: usize, direction: Vec3) -> Self {
        Self {
            resolution,
            direction,
        }
    }
}

fn recompute_face_mesh(mesh: &mut Mesh, local_up: Vec3, resolution: usize) {
    if resolution < 2 {
        panic!("resolution must be >= 2")
    }
    let axis_a = Vec3::new(local_up.y, local_up.z, local_up.x);
    let axis_b = local_up.cross(axis_a);
    let mut indicies: Vec<u32> = Vec::with_capacity((resolution - 1) * (resolution - 1) * 6);
    let mut positions = Vec::with_capacity(resolution * resolution);
    let mut normals = Vec::with_capacity(resolution * resolution);
    let mut uvs = Vec::with_capacity(resolution * resolution);

    for y in 0..resolution {
        for x in 0..resolution {
            uvs.push([0.0, 0.0]); // TODO: what to do here

            // Generate vertex position
            let percent = Vec2::new(x as f32, y as f32) / (resolution - 1) as f32;
            let point_on_unit_cube =
                local_up + (percent.x - 0.5) * 2.0 * axis_a + (percent.y - 0.5) * 2.0 * axis_b;
            let point_on_unit_sphere = point_on_unit_cube.normalize() * 2.0;
            positions.push(point_on_unit_sphere.to_array());

            normals.push(Vec3::ZERO);

            // generate index
            let i = (x + y * resolution) as u32;
            if x != (resolution - 1) && y != (resolution - 1) {
                indicies.push(i);
                indicies.push(i + resolution as u32 + 1);
                indicies.push(i + resolution as u32);
                indicies.push(i);
                indicies.push(i + 1);
                indicies.push(i + resolution as u32 + 1);
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
            recompute_face_mesh(mesh, face.direction, face.resolution);
        }
    }
}
