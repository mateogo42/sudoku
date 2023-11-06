mod colors;
use bevy::{
    input::{common_conditions::input_just_pressed, keyboard::KeyboardInput, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};
use colors::*;
use rand::{seq::SliceRandom, Rng};

const HEIGHT: f32 = 400.0;
const WIDTH: f32 = 400.0;
const BORDER_SIZE: f32 = 2.0;
const CELL_SIZE: f32 = WIDTH / 9.0;

#[derive(Component)]
struct Cell {
    row: usize,
    col: usize,
    valid: bool,
}

#[derive(Resource)]
struct Board([[u8; 9]; 9]);

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct Fixed;

fn main() {
    App::new()
        .insert_resource(ClearColor(WHITE))
        .insert_resource(Board([[0; 9]; 9]))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (WIDTH + BORDER_SIZE * 4., HEIGHT + BORDER_SIZE * 4.).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            select_cell.run_if(input_just_pressed(MouseButton::Left)),
        )
        .add_systems(Update, (handle_keys, check_finish))
        .run();
}

fn setup(mut commands: Commands, mut board: ResMut<Board>) {
    commands.spawn(Camera2dBundle::default());
    // Draw grid
    board.0 = init_board();
    for i in 0..9 {
        for j in 0..9 {
            let padding_x = if (j) % 3 == 0 {
                BORDER_SIZE * 2.
            } else {
                BORDER_SIZE
            };
            let padding_y = if (i) % 3 == 0 {
                BORDER_SIZE * 2.
            } else {
                BORDER_SIZE
            };
            let x = (-WIDTH + CELL_SIZE) / 2. + j as f32 * CELL_SIZE;
            let y = (HEIGHT - CELL_SIZE) / 2. - i as f32 * CELL_SIZE;
            let value = if board.0[i][j] != 0 {
                board.0[i][j].to_string()
            } else {
                "".to_string()
            };
            let color = if value.is_empty() { WHITE } else { BLUE };
            let cell = commands
                .spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: BLACK,
                            custom_size: Some(Vec2::new(
                                CELL_SIZE - padding_x / 2.,
                                CELL_SIZE - padding_y / 2.,
                            )),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(
                            x + padding_x / 2.,
                            y - padding_y / 2.,
                            0.,
                        )),
                        ..default()
                    },
                    Cell {
                        row: i,
                        col: j,
                        valid: true,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(Text2dBundle {
                        text: Text::from_section(
                            value.clone(),
                            TextStyle {
                                font_size: 30.,
                                color,
                                ..default()
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                        transform: Transform::from_translation(Vec3::Z),
                        ..default()
                    });
                })
                .id();
            if !value.is_empty() {
                commands.entity(cell).insert(Fixed);
            }
        }
    }
}

fn init_board() -> [[u8; 9]; 9] {
    let mut board = [[0; 9]; 9];
    let mut rng = rand::thread_rng();

    // Fill diagonals
    for diag in 0..3 {
        let mut choices: [u8; 9] = core::array::from_fn(|i| i as u8 + 1);
        for i in 0..3 {
            for j in 0..3 {
                loop {
                    let num = choices.choose(&mut rng).unwrap();
                    if *num != 0 {
                        board[3 * diag + i][3 * diag + j] = *num;
                        choices[*num as usize - 1] = 0;
                        break;
                    }
                }
            }
        }
    }
    fill_rest(&mut board, 0, 0);

    for _ in 0..1 {
        loop {
            let i = rng.gen_range(0, 9);
            let j = rng.gen_range(0, 9);
            if board[i][j] != 0 {
                board[i][j] = 0;
                break;
            }
        }
    }
    board
}

fn fill_rest(board: &mut [[u8; 9]; 9], mut x: usize, mut y: usize) -> bool {
    if x == 8 && y == 9 {
        return true;
    }
    if y == 9 {
        x += 1;
        y = 0;
    }
    if board[x][y] != 0 {
        return fill_rest(board, x, y + 1);
    }

    for n in 1..=9 {
        if is_valid(board, x, y, n) {
            board[x][y] = n;
            if fill_rest(board, x, y + 1) {
                return true;
            }
            board[x][y] = 0;
        }
    }

    false
}

fn is_valid(board: &mut [[u8; 9]; 9], x: usize, y: usize, value: u8) -> bool {
    let row = board[x];
    let col = board.map(|row| row[y]);
    let block = get_block_values(board, x, y);
    if !row.contains(&value) && !col.contains(&value) && !block.contains(&value) {
        return true;
    }

    false
}

fn get_block_values(board: &mut [[u8; 9]; 9], x: usize, y: usize) -> [u8; 9] {
    let mut block = [0; 9];
    let row_start = 3 * (x / 3);
    let col_start = 3 * (y / 3);
    for i in 0..3 {
        for j in 0..3 {
            block[3 * i + j] = board[row_start + i][col_start + j];
        }
    }
    block
}

fn select_cell(
    mut commands: Commands,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(Entity, &Cell, &Transform, &mut Sprite), (Without<Selected>, Without<Fixed>)>,
    mut selected_cell: Query<(Entity, &mut Sprite), With<Selected>>,
) {
    let position = q_windows.single().cursor_position().unwrap();
    let pos_x = position.x - WIDTH / 2.;
    let pos_y = -position.y + HEIGHT / 2.;
    let mut change_selected = false;
    for (entity, cell, transform, mut sprite) in query.iter_mut() {
        let cell_left = transform.translation.x - CELL_SIZE / 2.;
        let cell_right = transform.translation.x + CELL_SIZE / 2.;
        let cell_up = transform.translation.y + CELL_SIZE / 2.;
        let cell_down = transform.translation.y - CELL_SIZE / 2.;
        if (pos_x > cell_left && pos_x < cell_right) && (pos_y < cell_up && pos_y > cell_down) {
            sprite.color = if cell.valid { YELLOW } else { RED };
            commands.entity(entity).insert(Selected);
            change_selected = true;
        }

        if change_selected {
            if let Ok((old_entity, mut old_sprite)) = selected_cell.get_single_mut() {
                commands.entity(old_entity).remove::<Selected>();
                old_sprite.color = BLACK;
            }
        }
    }
}

fn key_to_value(key_option: Option<KeyCode>) -> Option<u8> {
    if let Some(key) = key_option {
        return match key {
            KeyCode::Delete | KeyCode::Back => Some(0),
            KeyCode::Numpad1 | KeyCode::Key1 => Some(1),
            KeyCode::Numpad2 | KeyCode::Key2 => Some(2),
            KeyCode::Numpad3 | KeyCode::Key3 => Some(3),
            KeyCode::Numpad4 | KeyCode::Key4 => Some(4),
            KeyCode::Numpad5 | KeyCode::Key5 => Some(5),
            KeyCode::Numpad6 | KeyCode::Key6 => Some(6),
            KeyCode::Numpad7 | KeyCode::Key7 => Some(7),
            KeyCode::Numpad8 | KeyCode::Key8 => Some(8),
            KeyCode::Numpad9 | KeyCode::Key9 => Some(9),
            _ => None,
        };
    }
    None
}

fn handle_keys(
    mut board: ResMut<Board>,
    mut key_evr: EventReader<KeyboardInput>,
    mut query: Query<(&mut Cell, &mut Sprite, &Children), With<Selected>>,
    mut text_query: Query<&mut Text>,
) {
    if let Ok((mut cell, mut sprite, children)) = query.get_single_mut() {
        for ev in key_evr.iter() {
            if ev.state == ButtonState::Pressed {
                for &child in children.iter() {
                    let mut text = text_query.get_mut(child).unwrap();
                    if let Some(value) = key_to_value(ev.key_code) {
                        if value == 0 {
                            text.sections[0].value = "".to_string();
                            cell.valid = true;
                        } else {
                            text.sections[0].value = value.to_string();
                            cell.valid = is_valid(&mut board.0, cell.row, cell.col, value);
                        }
                        if cell.valid {
                            cell.valid = true;
                            sprite.color = YELLOW;
                        } else {
                            cell.valid = false;
                            sprite.color = RED;
                        }
                        board.0[cell.row][cell.col] = value;
                    }
                }
            }
        }
    }
}

fn check_finish(
    mut commands: Commands,
    board: ResMut<Board>,
    mut query: Query<(Entity, &Cell, &mut Sprite, &Children, Option<&Selected>)>,
    mut text_query: Query<&mut Text>,
) {
    let mut win = true;
    for (_, cell, _, _, _) in query.iter() {
        win &= board.0[cell.row][cell.col] != 0 && cell.valid;
    }
    if win {
        for (entity, _, mut sprite, children, selected) in query.iter_mut() {
            if selected.is_some() {
                commands.entity(entity).remove::<Selected>();
            }
            for &child in children.iter() {
                let mut text = text_query.get_mut(child).unwrap();
                text.sections[0].style.color = WHITE;
            }
            sprite.color = GREEN;
            commands.entity(entity).insert(Fixed);
        }
    }
}
