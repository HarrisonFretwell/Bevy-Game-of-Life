use bevy::{core::FixedTimestep, prelude::*, utils::HashMap};

const ARENA_WIDTH: i32 = 32;
const ARENA_HEIGHT: i32 = 32;
const ALIVE_COLOUR: Color = Color::rgb(0.75, 0.85, 0.5);
const DEAD_COLOUR: Color = Color::rgb(0.3, 0.3, 0.3);
#[derive(Component, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
struct Position {
    x: i32,
    y: i32,
}
#[derive(Component, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
struct CellState {
    alive: bool,
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Snake!".to_string(),
            width: 1000.,
            height: 1000.,
            ..Default::default()
        })
        .add_startup_system(setup_camera)
        .add_startup_system(startup_spawn)
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        .add_plugins(DefaultPlugins)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.1))
                .with_system(update_cells),
        )
        .run();
}
fn startup_spawn(mut commands: Commands) {
    for x in 0..ARENA_WIDTH {
        for y in 0..ARENA_HEIGHT {
            spawn_tile(&mut commands, x, y, x == 4 && (y % 3 == 0 || y == 2));
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn spawn_tile(commands: &mut Commands, x: i32, y: i32, alive: bool) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: if alive { ALIVE_COLOUR } else { DEAD_COLOUR },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Position { x, y })
        .insert(CellState { alive })
        .id()
}
fn number_of_alive_neighbours(hash_map: &HashMap<Position, CellState>, position: Position) -> i32 {
    let mut count = 0;
    for x in position.x - 1..=position.x + 1 {
        for y in position.y - 1..=position.y + 1 {
            if x == position.x && y == position.y {
                //Skip self
                continue;
            }
            if let Some(cell_state) = hash_map.get(&Position { x, y }) {
                if cell_state.alive {
                    count += 1;
                }
            }
        }
    }
    count
}

fn update_cells(mut commands: Commands, mut q: Query<(&mut CellState, &Position, &mut Sprite)>) {
    let mut cell_map: HashMap<Position, CellState> = HashMap::default();
    q.iter().for_each(|(cell_state, position, _sprite)| {
        cell_map.insert(*position, *cell_state);
    });
    for (mut cell_state, position, mut sprite) in q.iter_mut() {
        let alive_neighbours = number_of_alive_neighbours(&cell_map, *position);
        if alive_neighbours == 2 || alive_neighbours == 3 {
            cell_state.alive = true;
            sprite.color = ALIVE_COLOUR;
        } else {
            cell_state.alive = false;
            sprite.color = DEAD_COLOUR;
        }
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<&mut Transform, With<Position>>) {
    let window = windows.get_primary().unwrap();
    const TILE_SIZE: f32 = 0.9;
    for mut transform in q.iter_mut() {
        transform.scale = Vec3::new(
            TILE_SIZE / ARENA_WIDTH as f32 * window.width() as f32,
            TILE_SIZE / ARENA_HEIGHT as f32 * window.height() as f32,
            1.0,
        )
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert_coords_to_screen_space(pos: f32, window_size: f32, game_size: f32) -> f32 {
        let tile_size = window_size / game_size;
        pos / game_size * window_size - (window_size / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut t) in q.iter_mut() {
        t.translation = Vec3::new(
            convert_coords_to_screen_space(
                pos.x as f32,
                window.width() as f32,
                ARENA_HEIGHT as f32,
            ),
            convert_coords_to_screen_space(
                pos.y as f32,
                window.height() as f32,
                ARENA_HEIGHT as f32,
            ),
            0.0,
        )
    }
}
