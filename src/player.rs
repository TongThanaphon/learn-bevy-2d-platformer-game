use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::{
    Collider, KinematicCharacterController, KinematicCharacterControllerOutput, RigidBody,
};

use crate::{WINDOW_BOTTOM_Y, WINDOW_LEFT_X, animation::Animation};

const PLAYER_VELOCITY_X: f32 = 400.0;
const PLAYER_VELOCITY_Y: f32 = 850.0;

const MAX_JUMP_HEIGHT: f32 = 230.0;

const SPRITESHEET_COLS: u32 = 7;
const SPRITESHEET_ROWS: u32 = 8;
const SPRITE_TILE_WIDTH: u32 = 128;
const SPRITE_TILE_HEIGHT: u32 = 256;
const SPRITE_IDX_STAND: usize = 28;
const SPRITE_RENDER_WIDTH: f32 = 64.0;
const SPRITE_RENDER_HEIGHT: f32 = 128.0;

const SPRITE_IDX_WALKING: &[usize] = &[7, 0];
const SPRITE_IDX_JUMPING: usize = 35;

const CYCLE_DELAY: Duration = Duration::from_millis(70);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, movement)
            .add_systems(Update, jump)
            .add_systems(Update, rise)
            .add_systems(Update, fall)
            .add_systems(Update, apply_movement_animation)
            .add_systems(Update, apply_idle_sprite)
            .add_systems(Update, apply_jump_sprite)
            .add_systems(Update, update_direction)
            .add_systems(Update, update_sprite_direction);
    }
}

#[derive(Component)]
struct Jump(f32);

#[derive(Component)]
enum Direction {
    Right,
    Left,
}

fn setup(
    mut commands: Commands,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
    server: Res<AssetServer>,
) {
    let image_handle: Handle<Image> = server.load("spritesheets/spritesheet_players.png");
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(SPRITE_TILE_WIDTH, SPRITE_TILE_HEIGHT),
        SPRITESHEET_COLS,
        SPRITESHEET_ROWS,
        None,
        None,
    );
    let layout_handle = atlases.add(layout);

    commands
        .spawn((
            Sprite::from_atlas_image(
                image_handle,
                TextureAtlas {
                    layout: layout_handle,
                    index: SPRITE_IDX_STAND,
                },
            ),
            Transform {
                translation: Vec3::new(WINDOW_LEFT_X + 100.0, WINDOW_BOTTOM_Y + 300.0, 0.0),
                scale: Vec3::new(
                    SPRITE_RENDER_WIDTH / SPRITE_TILE_WIDTH as f32,
                    SPRITE_RENDER_HEIGHT / SPRITE_TILE_HEIGHT as f32,
                    1.0,
                ),
                ..default()
            },
        ))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(
            SPRITE_TILE_WIDTH as f32 / 2.0,
            SPRITE_TILE_HEIGHT as f32 / 2.0,
        ))
        .insert(KinematicCharacterController::default())
        .insert(Direction::Right);
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

fn apply_movement_animation(
    mut commands: Commands,
    query: Query<(Entity, &KinematicCharacterControllerOutput), Without<Animation>>,
) {
    if query.is_empty() {
        return;
    }

    if let Ok((player, output)) = query.single() {
        if output.desired_translation.x != 0.0 && output.grounded {
            commands
                .entity(player)
                .insert(Animation::new(SPRITE_IDX_WALKING, CYCLE_DELAY));
        }
    }
}

fn apply_idle_sprite(
    mut commands: Commands,
    mut query: Query<(Entity, &KinematicCharacterControllerOutput, &mut Sprite)>,
) {
    if query.is_empty() {
        return;
    }

    if let Ok((player, output, mut sprite)) = query.single_mut() {
        if let Some(current_sprite) = sprite.texture_atlas.as_mut() {
            if output.desired_translation.x == 0.0 && output.grounded {
                commands.entity(player).remove::<Animation>();
                current_sprite.index = SPRITE_IDX_STAND;
            }
        }
    }
}

fn apply_jump_sprite(
    mut commands: Commands,
    mut query: Query<(Entity, &KinematicCharacterControllerOutput, &mut Sprite)>,
) {
    if query.is_empty() {
        return;
    }

    if let Ok((player, output, mut sprite)) = query.single_mut() {
        if let Some(current_sprite) = sprite.texture_atlas.as_mut() {
            if !output.grounded {
                commands.entity(player).remove::<Animation>();
                current_sprite.index = SPRITE_IDX_JUMPING;
            }
        }
    }
}

fn update_direction(
    mut commands: Commands,
    query: Query<(Entity, &KinematicCharacterControllerOutput)>,
) {
    if query.is_empty() {
        return;
    }

    if let Ok((player, output)) = query.single() {
        if output.desired_translation.x > 0.0 {
            commands.entity(player).insert(Direction::Right);
        } else if output.desired_translation.x < 0.0 {
            commands.entity(player).insert(Direction::Left);
        }
    }
}

fn update_sprite_direction(mut query: Query<(&mut Sprite, &Direction)>) {
    if query.is_empty() {
        return;
    }

    if let Ok((mut sprit, direction)) = query.single_mut() {
        if let Some(_) = sprit.texture_atlas.as_mut() {
            match direction {
                Direction::Right => {
                    sprit.flip_x = false;
                }
                Direction::Left => {
                    sprit.flip_x = true;
                }
            }
        }
    }
}
