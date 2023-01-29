use super::*;
use bevy::prelude::*;
use std::collections::HashSet;

pub struct Highlight(Position);
pub struct Highlights {
    pub selected_move: Option<Position>,
    pub selected_piece_moves: HashSet<Position>,
}
pub struct HighlightTextures {
    none: Handle<ColorMaterial>,
    selected: Handle<ColorMaterial>,
    possible_move: Handle<ColorMaterial>,
    hovering_move: Handle<ColorMaterial>,
    last_move: Handle<ColorMaterial>,
    capture_move: Handle<ColorMaterial>,
    in_check: Handle<ColorMaterial>,
}

pub fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(Highlights {
        selected_move: None,
        selected_piece_moves: HashSet::new(),
    });

    let yellow = materials.add(asset_server.load("textures/highlights/yellow.png").into());

    commands.insert_resource(HighlightTextures {
        none: materials.add(Color::NONE.into()),
        selected: yellow.clone(),
        possible_move: materials.add(asset_server.load("textures/highlights/move.png").into()),
        hovering_move: materials.add(asset_server.load("textures/highlights/hover.png").into()),
        last_move: yellow,
        capture_move: materials.add(asset_server.load("textures/highlights/capture.png").into()),
        in_check: materials.add(asset_server.load("textures/highlights/check.png").into()),
    });

    let temp = SpriteBundle {
        sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
        visible: Visible {
            is_visible: true,
            is_transparent: true,
        },
        ..Default::default()
    };

    for y in 0..8 {
        for x in 0..8 {
            let mut sprite = temp.clone();
            sprite.transform = from_xy(x, y, 1.0);
            commands
                .spawn_bundle(sprite)
                .insert(Highlight(Position::from_xy(x, y).unwrap()));
        }
    }
}

pub fn update(
    mut highlights: Query<(&Highlight, &mut Handle<ColorMaterial>)>,
    materials: Res<HighlightTextures>,
    states: Res<BoardStates>,
    mut highlight_state: ResMut<Highlights>,
    selected: Res<Selected>,
    piece_entities: Res<PieceEntities>,
) {
    let king = states.active().state.king_in_check();

    if highlight_state.is_changed() || selected.is_changed() || states.is_changed() {
        highlight_state.selected_piece_moves = HashSet::new();
        if let Some(pos) = selected.0 {
            highlight_state.selected_piece_moves =
                states.active().piece_moves.get(&pos).unwrap().clone();
        }

        for (Highlight(pos), mut material) in highlights.iter_mut() {
            if let Some(m) = highlight_state.selected_move {
                if m == *pos {
                    *material = materials.hovering_move.clone();
                    continue;
                }
            }
            if let Some(s) = selected.0 {
                if s == *pos {
                    *material = materials.selected.clone();
                    continue;
                }
            }
            if let Some(k) = king {
                if k == *pos {
                    *material = materials.in_check.clone();
                    continue;
                }
            }
            if highlight_state.selected_piece_moves.contains(pos) {
                if piece_entities.0.contains_key(pos) {
                    *material = materials.capture_move.clone();
                } else {
                    *material = materials.possible_move.clone();
                }
                continue;
            }
            if let Some((one, two)) = states.active().last_move {
                if one == *pos || two == *pos {
                    *material = materials.last_move.clone();
                    continue;
                }
            }
            *material = materials.none.clone();
        }
    }
}
