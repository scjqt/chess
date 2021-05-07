#![windows_subsystem = "windows"]

mod chess;
mod states;
mod board;
mod highlights;
mod ui;

use std::collections::{HashMap, HashSet};
use bevy::{prelude::*, render::pass::ClearColor};
use chess::{Piece, Position, Colour, Variant, Colour::*, Variant::*, EndState};
use board::{Textures, PieceEntities, Drag};
use states::BoardStates;
use highlights::Highlights;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Playing,
    Promoting,
    End,
}

const SQUARE_SIZE: f32 = 64.0;
const SCREEN_WIDTH: f32 = 600.0;
const SCREEN_HEIGHT: f32 = 660.0;
const CENTRE_X: f32 = 0.0;
const CENTRE_Y: f32 = 30.0;

fn main() {
    App::build()
    .insert_resource(WindowDescriptor {
        title: "Chess".to_string(),
        resizable: false,
        width: SCREEN_WIDTH,
        height: SCREEN_HEIGHT,
        vsync: false,
        ..Default::default()
    })
    .insert_resource(ClearColor(Color::rgb(0.19, 0.18, 0.17)))
    .add_plugins(DefaultPlugins)
    .init_resource::<FontAsset>()
    .add_state(GameState::Playing)
    .add_startup_system(setup.system().label("setup"))
    .add_startup_system(ui::setup.system().after("setup"))
    .add_startup_system(highlights::setup.system())
    .add_startup_system(board::setup.system())
    .add_system_set(
        SystemSet::on_update(GameState::Playing)
        .with_system(ui::update_greyed.system().before("buttons"))
        .with_system(ui::update_buttons.system().label("buttons").before("update"))
        .with_system(update.system().label("update"))
        .with_system(highlights::update.system().after("update"))
        .with_system(board::update.system().label("pieces").after("update"))
        .with_system(board::update_drag.system().after("pieces"))
    )
    .add_system_set(
        SystemSet::on_enter(GameState::Promoting)
        .with_system(ui::setup_promotion.system())
    )
    .add_system_set(
        SystemSet::on_update(GameState::Promoting)
        .with_system(ui::update_promotion.system())
    )
    .add_system_set(
        SystemSet::on_exit(GameState::Promoting)
        .with_system(ui::destruct_promotion.system())
        .with_system(board::update.system())
    )
    .add_system_set(
        SystemSet::on_enter(GameState::End)
        .with_system(ui::setup_end_screen.system())
    )
    .add_system_set(
        SystemSet::on_update(GameState::End)
        .with_system(ui::update_end_screen.system())
    )
    .add_system_set(
        SystemSet::on_exit(GameState::End)
        .with_system(ui::destruct_end_screen.system())
    )
    .run();
}

pub struct Selected(Option<Position>);
pub struct FontAsset(Handle<Font>);
struct Toggle(bool);

impl FromWorld for FontAsset {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        FontAsset(asset_server.load("fonts/Lato-Regular.ttf"))
    }
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    commands.insert_resource(BoardStates::new());
    commands.insert_resource(Toggle(false));
    commands.insert_resource(Selected(None));
}

fn update(
    mut game_state: ResMut<State<GameState>>,
    mut states: ResMut<BoardStates>,
    mut selected: ResMut<Selected>,
    mut highlights: ResMut<Highlights>,
    mut drag: ResMut<Drag>,
    mut toggle: ResMut<Toggle>,
    windows: Res<Windows>,
    mouse_input: Res<Input<MouseButton>>,
) {
    let window = windows.get_primary().unwrap();
    let mouse_pos = if let Some(pos) = window.cursor_position() { pos - Vec2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0) } else { return };

    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(pos) = to_board_pos(mouse_pos) {
            if let Some(s) = selected.0 {
                if try_move(&mut states, &mut game_state, s, pos) {
                    selected.0 = None;
                } else if s == pos {
                    toggle.0 = true;
                } else if states.active().state.get_piece_moves().contains_key(&pos) {
                    selected.0 = Some(pos);
                } else {
                    selected.0 = None;
                }
            } else {
                if states.active().state.get_piece_moves().contains_key(&pos) {
                    selected.0 = Some(pos);
                }
            }
        } else {
            if selected.0.is_some() {
                selected.0 = None;
            }
        }
    } else if mouse_input.pressed(MouseButton::Left) {
        if let Some(s) = selected.0 {
            *drag = Drag::Mouse(mouse_pos);
            let mut new = None;
            if let Some(pos) = to_board_pos(mouse_pos) {
                if states.active().state.get_piece_moves().get(&s).unwrap().contains(&pos) {
                    new = Some(pos);
                }
            }
            if highlights.selected_move != new {
                highlights.selected_move = new;
            }
        }
    } else if mouse_input.just_released(MouseButton::Left) {
        if let Some(s) = selected.0 {
            if let Some(pos) = to_board_pos(mouse_pos) {
                if try_move(&mut states, &mut game_state, s, pos) {
                    selected.0 = None;
                } else if pos == s {
                    *drag = Drag::Reset(s);
                    if toggle.0 {
                        selected.0 = None;
                    }
                } else {
                    *drag = Drag::Reset(s);
                }
            } else {
                *drag = Drag::Reset(s);
            }
            toggle.0 = false;
            if highlights.selected_move.is_some() {
                highlights.selected_move = None;
            }
        }
    } else {
        if let Some(s) = selected.0 {
            *drag = Drag::Reset(s);
            toggle.0 = false;
            if highlights.selected_move.is_some() {
                highlights.selected_move = None;
            }
        }
    }
    if mouse_input.just_pressed(MouseButton::Right) {
        if let Some(s) = selected.0 {
            *drag = Drag::Reset(s);
            selected.0 = None;
            highlights.selected_move = None;
            toggle.0 = false;
        }
    }
}

fn try_move(
    states: &mut BoardStates,
    game_state: &mut State<GameState>,
    from: Position,
    to: Position,
) -> bool {
    if let Some(new) = states.active().try_move(from, to) {
        states.add(new);
        if states.active().state.promoting() {
            game_state.set(GameState::Promoting).unwrap();
        } else if states.active().state.ended().is_some() {
            game_state.set(GameState::End).unwrap();
        } 
        return true;
    }
    false
}

fn to_board_pos(pos: Vec2) -> Option<Position> {
    Position::from_xy(((pos.x - CENTRE_X) / SQUARE_SIZE + 5.0) as i8 - 1, ((pos.y - CENTRE_Y) / SQUARE_SIZE + 5.0) as i8 - 1)
}

fn from_board_pos(pos: Position, z: f32) -> Transform {
    Transform::from_xyz((pos.get_x() as f32 - 3.5) * SQUARE_SIZE + CENTRE_X, (pos.get_y() as f32 - 3.5) * SQUARE_SIZE + CENTRE_Y, z)
}

fn from_xy(x: i8, y: i8, z: f32) -> Transform {
    Transform::from_xyz((x as f32 - 3.5) * SQUARE_SIZE + CENTRE_X, (y as f32 - 3.5) * SQUARE_SIZE + CENTRE_Y, z)
}