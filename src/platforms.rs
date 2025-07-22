use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, RigidBody};

use crate::WINDOW_BOTTOM_Y;

const COLOR_PLATFORM: Color = Color::srgb(0.13, 0.13, 0.23);

#[derive(Bundle)]
struct PlatformBundle {
    bundle: (Sprite, Transform),
    body: RigidBody,
    collider: Collider,
}

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

pub struct PlatformsPlugin;

impl Plugin for PlatformsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(PlatformBundle::new(-100.0, Vec3::new(75.0, 200.0, 1.0)));
    commands.spawn(PlatformBundle::new(100.0, Vec3::new(50.0, 350.0, 1.0)));
    commands.spawn(PlatformBundle::new(350.0, Vec3::new(150.0, 170.0, 1.0)));
}
