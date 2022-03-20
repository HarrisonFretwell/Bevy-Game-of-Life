use bevy::{core::FixedTimestep, ecs::schedule::ShouldRun, prelude::*, utils::HashMap};

const ARENA_WIDTH: i32 = 16;
const ARENA_HEIGHT: i32 = 16;
const ALIVE_COLOUR: Color = Color::rgb(0.75, 0.85, 0.5);
const DEAD_COLOUR: Color = Color::rgb(0.3, 0.3, 0.3);
const BORDER_SIZE: f32 = 25.0;
#[derive(Component, Clone, Copy, PartialEq, Eq, std::hash::Hash, Debug)]
struct Position {
    x: i32,
    y: i32,
}
#[derive(Component, Clone, Copy, PartialEq, Eq, std::hash::Hash, Debug)]
struct CellState {
    is_currently_alive: bool,
    will_be_alive: bool,
}

#[derive(Debug, PartialEq)]
enum RunState {
    Paused,
    Running,
}

#[derive(Debug, PartialEq)]
struct AppState(RunState);

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Snake!".to_string(),
            width: 1000.,
            height: 1000.,
            ..Default::default()
        })
        .add_startup_system(setup)
        .add_startup_system(startup_spawn)
        .insert_resource(AppState(RunState::Paused))
        .add_system(user_input)
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        .add_plugins(DefaultPlugins)
        .add_system_set(
            SystemSet::new()
                .with_system(update_cells)
                .with_run_criteria(FixedTimestep::step(0.3)),
        )
        .run();
}

fn startup_spawn(mut commands: Commands) {
    for x in 0..ARENA_WIDTH {
        for y in 0..ARENA_HEIGHT {
            spawn_tile(&mut commands, x, y, x == 2);
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn user_input(keyboard_input: Res<Input<KeyCode>>, mut app_state: ResMut<AppState>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if (*app_state).0 == RunState::Paused {
            (*app_state).0 = RunState::Running;
        } else {
            (*app_state).0 = RunState::Paused;
        }
    }
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
        .insert(CellState {
            is_currently_alive: alive,
            will_be_alive: alive,
        })
        .id()
}
fn number_of_alive_neighbours(hash_map: &HashMap<Position, CellState>, position: Position) -> i32 {
    let mut count = 0;
    for x in (position.x - 1)..(position.x + 2) {
        for y in (position.y - 1)..(position.y + 2) {
            if x == position.x && y == position.y {
                continue;
            }
            if !(0..ARENA_WIDTH).contains(&x) || !(0..ARENA_HEIGHT).contains(&y) {
                continue;
            }
            if let Some(cell) = hash_map.get(&Position { x, y }) {
                if cell.is_currently_alive {
                    count += 1;
                }
            }
        }
    }
    count
}

fn update_cells(mut q: Query<(&mut CellState, &Position, &mut Sprite)>, app_state: Res<AppState>) {
    if (*app_state).0 == RunState::Paused {
        return;
    }
    let mut cell_map: HashMap<Position, CellState> = HashMap::default();
    q.iter().for_each(|(cell_state, position, _sprite)| {
        cell_map.insert(*position, *cell_state);
    });
    for (mut cell_state, position, _sprite) in q.iter_mut() {
        let alive_neighbours = number_of_alive_neighbours(&cell_map, *position);
        if alive_neighbours == 2 || alive_neighbours == 3 {
            cell_state.will_be_alive = true;
        } else {
            cell_state.will_be_alive = false;
        }
    }
    for (mut cell_state, _position, mut sprite) in q.iter_mut() {
        if cell_state.will_be_alive {
            sprite.color = ALIVE_COLOUR;
        } else {
            sprite.color = DEAD_COLOUR;
        }
        cell_state.is_currently_alive = cell_state.will_be_alive;
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
                (window.width() - BORDER_SIZE) as f32,
                ARENA_WIDTH as f32,
            ),
            convert_coords_to_screen_space(
                pos.y as f32,
                (window.height() - BORDER_SIZE) as f32,
                ARENA_HEIGHT as f32,
            ),
            0.0,
        )
    }
}
