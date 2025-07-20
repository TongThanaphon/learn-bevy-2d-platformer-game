use bevy::{prelude::*, window::WindowResolution};
use bevy_rapier2d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    prelude::{
        Collider, KinematicCharacterController, KinematicCharacterControllerOutput, RigidBody,
    },
    render::RapierDebugRenderPlugin,
};

const WINDOW_WIDTH: f32 = 1024.0;
const WINDOW_HEIGHT: f32 = 720.0;

const WINDOW_BOTTOM_Y: f32 = WINDOW_HEIGHT / -2.0;
const WINDOW_LEFT_X: f32 = WINDOW_WIDTH / -2.0;

const COLOR_BACKGROUND: Color = Color::srgb(0.29, 0.31, 0.41);
const COLOR_PLATFORM: Color = Color::srgb(0.13, 0.13, 0.23);
const COLOR_PLAYER: Color = Color::srgb(0.6, 0.55, 0.6);
const COLOR_FLOOR: Color = Color::srgb(0.45, 0.55, 0.66);

const FLOOR_THICKNESS: f32 = 10.0;

const PLAYER_VELOCITY_X: f32 = 400.0;
const PLAYER_VELOCITY_Y: f32 = 850.0;
const MAX_JUMP_HEIGHT: f32 = 230.0;

#[derive(Bundle)]
struct PlatformBundle {
    bundle: (Sprite, Transform),
    body: RigidBody,
    collider: Collider,
}

#[derive(Component)]
struct Jump(f32);

impl PlatformBundle {
    fn new(x: f32, scale: Vec3) -> Self {
        Self {
            bundle: (
                Sprite {
                    color: COLOR_PLATFORM,
                    ..default()
                },
                Transform {
                    translation: Vec3::new(x, WINDOW_BOTTOM_Y + (scale.y / 2.0), 0.0),
                    scale,
                    ..default()
                },
            ),
            body: RigidBody::Fixed,
            collider: Collider::cuboid(0.5, 0.5),
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(COLOR_BACKGROUND))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Beavy Platformer".to_string(),
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, movement)
        .add_systems(Update, jump)
        .add_systems(Update, rise)
        .add_systems(Update, fall)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d::default());

    commands.spawn(PlatformBundle::new(-100.0, Vec3::new(75.0, 200.0, 1.0)));
    commands.spawn(PlatformBundle::new(100.0, Vec3::new(50.0, 350.0, 1.0)));
    commands.spawn(PlatformBundle::new(350.0, Vec3::new(150.0, 250.0, 1.0)));

    commands
        .spawn((
            Sprite {
                color: COLOR_FLOOR,
                ..default()
            },
            Transform {
                translation: Vec3::new(0.0, WINDOW_BOTTOM_Y + (FLOOR_THICKNESS / 2.0), 0.0),
                scale: Vec3::new(WINDOW_WIDTH, FLOOR_THICKNESS, 1.0),
                ..default()
            },
        ))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(0.5, 0.5));

    commands
        .spawn((
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(ColorMaterial::from(COLOR_PLAYER))),
            Transform {
                translation: Vec3::new(WINDOW_LEFT_X + 100.0, WINDOW_BOTTOM_Y + 60.0, 0.0),
                scale: Vec3::new(30.0, 30.0, 1.0),
                ..default()
            },
        ))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(0.5))
        .insert(KinematicCharacterController::default());
}

fn movement(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut KinematicCharacterController>,
) {
    let mut movement = 0.0;

    if input.pressed(KeyCode::ArrowRight) {
        movement += time.delta_secs() * PLAYER_VELOCITY_X;
    }

    if input.pressed(KeyCode::ArrowLeft) {
        movement += time.delta_secs() * PLAYER_VELOCITY_X * -1.0;
    }

    match query.single_mut() {
        Ok(mut player) => match player.translation {
            Some(vec) => player.translation = Some(Vec2::new(movement, vec.y)),
            None => player.translation = Some(Vec2::new(movement, 0.0)),
        },
        Err(_) => {}
    }
}

fn jump(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Query<
        (Entity, &KinematicCharacterControllerOutput),
        (With<KinematicCharacterController>, Without<Jump>),
    >,
) {
    if query.is_empty() {
        return;
    }

    match query.single() {
        Ok((player, output)) => {
            if input.pressed(KeyCode::Space) && output.grounded {
                commands.entity(player).insert(Jump(0.0));
            }
        }
        Err(_) => {
            return;
        }
    }
}

fn rise(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut KinematicCharacterController, &mut Jump)>,
) {
    if query.is_empty() {
        return;
    }

    match query.single_mut() {
        Ok((entity, mut player, mut jump)) => {
            let mut movement = time.delta().as_secs_f32() * PLAYER_VELOCITY_Y;

            if movement + jump.0 >= MAX_JUMP_HEIGHT {
                movement = MAX_JUMP_HEIGHT - jump.0;
                commands.entity(entity).remove::<Jump>();
            }

            jump.0 += movement;

            match player.translation {
                Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)),
                None => player.translation = Some(Vec2::new(0.0, movement)),
            }
        }
        Err(_) => {
            return;
        }
    }
}

fn fall(time: Res<Time>, mut query: Query<&mut KinematicCharacterController, Without<Jump>>) {
    if query.is_empty() {
        return;
    }

    let movement = time.delta().as_secs_f32() * (PLAYER_VELOCITY_Y / 1.5) * -1.0;

    if let Ok(mut player) = query.single_mut() {
        match player.translation {
            Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)),
            None => player.translation = Some(Vec2::new(0.0, movement)),
        }
    }
}
