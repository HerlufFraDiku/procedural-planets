use bevy::render::mesh::*;
use bevy::render::render_resource::*;
use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
};

use planet::make_planet_face_meshes;

mod orbit_camera;
mod planet;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .add_system(make_planet_face_meshes)
        .add_system(increment_resolution)
        .add_system(orbit_camera::pan_orbit_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cardinals: [Vec3; 6] = [
        -1.0 * Vec3::X,
        Vec3::X,
        -1.0 * Vec3::Y,
        Vec3::Y,
        -1.0 * Vec3::Z,
        Vec3::Z,
    ];

    // Planet mesh
    for direction in cardinals {
        let planet_face_mesh = meshes.add(Mesh::new(PrimitiveTopology::TriangleList));
        commands
            .spawn_bundle(PbrBundle {
                mesh: planet_face_mesh,
                material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            })
            .insert(Wireframe)
            .insert(planet::PlanetFace::new(2, direction));
    }

    //light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    orbit_camera::spawn_camera(&mut commands);
}

pub fn increment_resolution(mut query: Query<&mut planet::PlanetFace>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        for mut face in query.iter_mut() {
            face.resolution += 1;
        }
    }
}
