use super::*;

pub struct Textures(pub HashMap<Piece, Handle<ColorMaterial>>);
pub struct PieceEntities(pub HashMap<Position, Entity>);
pub struct PieceEntity;
pub enum Drag {
    None,
    Mouse(Vec2),
    Reset(Position),
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    font: Res<FontAsset>,
) {
    let mut textures = HashMap::new();

    textures.insert(
        Piece {
            colour: White,
            variant: Pawn,
        },
        materials.add(asset_server.load("textures/pieces/white_pawn.png").into()),
    );
    textures.insert(
        Piece {
            colour: White,
            variant: Knight,
        },
        materials.add(asset_server.load("textures/pieces/white_knight.png").into()),
    );
    textures.insert(
        Piece {
            colour: White,
            variant: Bishop,
        },
        materials.add(asset_server.load("textures/pieces/white_bishop.png").into()),
    );
    textures.insert(
        Piece {
            colour: White,
            variant: Rook,
        },
        materials.add(asset_server.load("textures/pieces/white_rook.png").into()),
    );
    textures.insert(
        Piece {
            colour: White,
            variant: Queen,
        },
        materials.add(asset_server.load("textures/pieces/white_queen.png").into()),
    );
    textures.insert(
        Piece {
            colour: White,
            variant: King,
        },
        materials.add(asset_server.load("textures/pieces/white_king.png").into()),
    );
    textures.insert(
        Piece {
            colour: Black,
            variant: Pawn,
        },
        materials.add(asset_server.load("textures/pieces/black_pawn.png").into()),
    );
    textures.insert(
        Piece {
            colour: Black,
            variant: Knight,
        },
        materials.add(asset_server.load("textures/pieces/black_knight.png").into()),
    );
    textures.insert(
        Piece {
            colour: Black,
            variant: Bishop,
        },
        materials.add(asset_server.load("textures/pieces/black_bishop.png").into()),
    );
    textures.insert(
        Piece {
            colour: Black,
            variant: Rook,
        },
        materials.add(asset_server.load("textures/pieces/black_rook.png").into()),
    );
    textures.insert(
        Piece {
            colour: Black,
            variant: Queen,
        },
        materials.add(asset_server.load("textures/pieces/black_queen.png").into()),
    );
    textures.insert(
        Piece {
            colour: Black,
            variant: King,
        },
        materials.add(asset_server.load("textures/pieces/black_king.png").into()),
    );

    commands.insert_resource(Textures(textures));
    commands.insert_resource(PieceEntities(HashMap::new()));
    commands.insert_resource(Drag::None);

    let black = materials.add(Color::rgb(0.46, 0.59, 0.335).into());
    let white = materials.add(Color::rgb(0.93, 0.93, 0.82).into());

    for y in 0..8 {
        for x in 0..8 {
            commands.spawn_bundle(SpriteBundle {
                material: if (x + y) % 2 == 0 {
                    black.clone()
                } else {
                    white.clone()
                },
                sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                transform: from_xy(x, y, 0.0),
                ..Default::default()
            });
        }
    }

    let style = TextStyle {
        font: font.0.clone(),
        font_size: 20.0,
        color: Color::rgb(0.95, 0.95, 0.95),
    };
    for (i, c) in ('a'..='h').enumerate() {
        text(
            &mut commands,
            style.clone(),
            &c.to_string(),
            Vec2::new(
                (i as f32 - 3.5) * SQUARE_SIZE + CENTRE_X,
                CENTRE_Y + SQUARE_SIZE * 4.0 + 13.0,
            ),
        );
        text(
            &mut commands,
            style.clone(),
            &c.to_string(),
            Vec2::new(
                (i as f32 - 3.5) * SQUARE_SIZE + CENTRE_X,
                CENTRE_Y - SQUARE_SIZE * 4.0 - 10.0,
            ),
        );
    }
    for i in 0..8 {
        text(
            &mut commands,
            style.clone(),
            &(i + 1).to_string(),
            Vec2::new(
                CENTRE_X + SQUARE_SIZE * 4.0 + 12.0,
                (i as f32 - 3.5) * SQUARE_SIZE + CENTRE_Y,
            ),
        );
        text(
            &mut commands,
            style.clone(),
            &(i + 1).to_string(),
            Vec2::new(
                CENTRE_X - SQUARE_SIZE * 4.0 - 12.0,
                (i as f32 - 3.5) * SQUARE_SIZE + CENTRE_Y,
            ),
        );
    }
}

fn text(commands: &mut Commands, style: TextStyle, text: &str, pos: Vec2) {
    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section(
            text,
            style,
            TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            },
        ),
        transform: Transform::from_xyz(pos.x, pos.y, 0.0),
        ..Default::default()
    });
}

pub fn update(
    mut commands: Commands,
    mut pieces: ResMut<PieceEntities>,
    existing: Query<Entity, With<PieceEntity>>,
    textures: Res<Textures>,
    states: Res<BoardStates>,
) {
    if states.is_changed() {
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }

        pieces.0 = HashMap::new();

        for (pos, piece) in states.active().state.get_pieces().iter() {
            pieces.0.insert(
                *pos,
                commands
                    .spawn_bundle(SpriteBundle {
                        material: textures.0.get(piece).unwrap().clone(),
                        transform: from_board_pos(*pos, 2.0),
                        ..Default::default()
                    })
                    .insert(PieceEntity)
                    .id(),
            );
        }
    }
}

pub fn update_drag(
    mut draggable: Query<&mut Transform, With<PieceEntity>>,
    pieces: Res<PieceEntities>,
    selected: Res<Selected>,
    mut drag: ResMut<Drag>,
) {
    if drag.is_changed() {
        match *drag {
            Drag::None => (),
            Drag::Mouse(pos) => {
                if let Ok(mut transform) = draggable.get_mut(pieces.0[&selected.0.unwrap()]) {
                    *transform = Transform::from_xyz(pos.x, pos.y, 3.0);
                }
            }
            Drag::Reset(pos) => {
                if let Ok(mut transform) = draggable.get_mut(pieces.0[&pos]) {
                    *transform = from_board_pos(pos, 2.0);
                }
                *drag = Drag::None;
            }
        }
    }
}
