use bevy::{prelude::*, window::WindowResolution};
use serde::Deserialize;
use std::collections::HashMap;

const TILE_SIZE: f32 = 32.0;
const MAP_WIDTH: f32 = 20.0 * TILE_SIZE;
const MAP_HEIGHT: f32 = 10.0 * TILE_SIZE;

#[derive(Debug, Deserialize)]
struct TileMapData {
    legend: HashMap<char, String>,
    tiles: Vec<String>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "2D Platformer Game".to_string(),
                resolution: WindowResolution::new(MAP_WIDTH, MAP_HEIGHT),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    let map_str = std::fs::read_to_string("assets/map.json").expect("Failed to read map file");
    let tile_map_data: TileMapData =
        serde_json::from_str(&map_str).expect("Failed to parse map data");

    let image_handle: Handle<Image> = asset_server.load("spritesheets/terrain.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 21, 11, None, None);
    let layout_handle = atlases.add(layout);

    for (y, row) in tile_map_data.tiles.iter().rev().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let entity_type = tile_map_data.legend.get(&ch);

            let texture = match entity_type.map(|s| s.as_str()) {
                Some("ground") => 7,
                _ => 0,
            };

            commands.spawn((
                Sprite::from_atlas_image(
                    image_handle.clone(),
                    TextureAtlas {
                        layout: layout_handle.clone(),
                        index: texture,
                    },
                ),
                Transform {
                    translation: Vec3::new(
                        (x as f32 * TILE_SIZE) - (MAP_WIDTH / 2.0) + (TILE_SIZE / 2.0), // Center horizontally
                        (y as f32 * TILE_SIZE) - (MAP_HEIGHT / 2.0) + (TILE_SIZE / 2.0), // Center vertically
                        1.0,
                    ),
                    scale: Vec3::new(2.0, 2.0, 1.0), // Scale to match tile size
                    ..default()
                },
            ));
        }
    }

    commands.spawn(Camera2d::default());
}
