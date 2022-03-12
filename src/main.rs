use bevy::render::mesh::*;
use bevy::render::render_resource::*;
use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
};

use planet::make_planet_face_meshes;

mod orbit_camera;
mod planet;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .add_system(increment_resolution)
        .add_system(increment_roughness)
        .add_system(toggle_wireframe)
        .add_system(move_center)
        // Only do expensive planet update 30 times a second
        .add_stage_after(
            CoreStage::Update,
            FixedUpdateStage,
            SystemStage::parallel()
                .with_run_criteria(bevy::core::FixedTimestep::step(1.0 / 30.0))
                .with_system(make_planet_face_meshes),
        )
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
                material: materials.add(StandardMaterial {
                    metallic: 0.1,
                    reflectance: 0.2,
                    perceptual_roughness: 0.9,
                    ..Default::default()
                }),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            })
            .insert(planet::PlanetFace {
                direction,
                ..Default::default()
            });
    }

    //light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            range: 100.,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..Default::default()
    });
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 750.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(-4.0, -8.0, -4.0),
        ..Default::default()
    });

    orbit_camera::spawn_camera(&mut commands);
}

pub fn increment_resolution(mut query: Query<&mut planet::PlanetFace>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::R) {
        for mut face in query.iter_mut() {
            face.resolution += 1;
        }
    }
    if keys.just_pressed(KeyCode::F) {
        for mut face in query.iter_mut() {
            face.resolution -= 1;
        }
    }
}

fn toggle_wireframe(
    mut commands: Commands,
    faces_with_wireframe: Query<Entity, (With<planet::PlanetFace>, With<Wireframe>)>,
    faces_without_wireframe: Query<Entity, (With<planet::PlanetFace>, Without<Wireframe>)>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for entt in faces_with_wireframe.iter() {
            commands.entity(entt).remove::<Wireframe>();
        }

        for entt in faces_without_wireframe.iter() {
            commands.entity(entt).insert(Wireframe);
        }
    }
}

pub fn increment_roughness(mut query: Query<&mut planet::PlanetFace>, keys: Res<Input<KeyCode>>) {
    if keys.pressed(KeyCode::T) {
        for mut face in query.iter_mut() {
            face.noise_roughness += 0.1;
        }
    }
    if keys.pressed(KeyCode::G) {
        for mut face in query.iter_mut() {
            face.noise_roughness -= 0.1;
        }
    }
}

pub fn move_center(
    time: Res<Time>,
    mut query: Query<&mut planet::PlanetFace>,
    keys: Res<Input<KeyCode>>,
) {
    let mut input = Vec3::ZERO;
    if keys.pressed(KeyCode::Left) {
        input.x -= 1.0;
    }
    if keys.pressed(KeyCode::Right) {
        input.x += 1.0;
    }
    if keys.pressed(KeyCode::Up) {
        input.y += 1.0;
    }
    if keys.pressed(KeyCode::Down) {
        input.y -= 1.0;
    }
    if input.length_squared() > 0.0 {
        for mut face in query.iter_mut() {
            face.noise_center += input * time.delta_seconds();
        }
    }
}
