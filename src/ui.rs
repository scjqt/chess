use bevy::{prelude::*, ui::FocusPolicy};
use super::*;

pub struct UIEntity(Entity);
pub struct PromoteTo(Variant);

pub struct UIMaterials {
    background: Handle<ColorMaterial>,
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    greyed: Handle<ColorMaterial>,
}

#[derive(PartialEq, Eq)]
pub enum ButtonType {
    Undo,
    Redo,
    Restart,
}

pub fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    font: Res<FontAsset>,
) {
    let ui_materials = UIMaterials {
        background: materials.add(Color::rgba(0.1, 0.1, 0.1, 0.7).into()),
        normal: materials.add(Color::rgba(0.4, 0.4, 0.4, 1.0).into()),
        hovered: materials.add(Color::rgba(0.6, 0.6, 0.6, 1.0).into()),
        greyed: materials.add(Color::rgba(0.21, 0.2, 0.19, 1.0).into()),
    };

    commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Px(SQUARE_SIZE * 8.0), Val::Px(60.0)),
            position_type: PositionType::Absolute,
            position: Rect { 
                left: Val::Px(SCREEN_WIDTH / 2.0 + CENTRE_X - SQUARE_SIZE * 4.0),
                bottom: Val::Px(SCREEN_HEIGHT / 2.0 + CENTRE_Y - SQUARE_SIZE * 4.0 - 90.0),
                ..Default::default()
            },
            align_items: AlignItems::Center,
            ..Default::default()
        },
        material: materials.add(Color::NONE.into()),
        ..Default::default()
    }).with_children(|parent| {
        let style = TextStyle {
            font: font.0.clone(),
            font_size: 30.0,
            color: Color::rgba(1.0, 1.0, 1.0, 0.7),
        };
        button(parent, &ui_materials, SQUARE_SIZE * 0.0, style.clone(), "Restart", ButtonType::Restart);
        button(parent, &ui_materials, SQUARE_SIZE * 3.5, style.clone(), "Undo", ButtonType::Undo);
        button(parent, &ui_materials, SQUARE_SIZE * 5.5, style, "Redo", ButtonType::Redo);
    });

    commands.insert_resource(ui_materials);
}

pub fn update_buttons(
    mut states: ResMut<BoardStates>,
    materials: Res<UIMaterials>,
    mut buttons: Query<(&Interaction, &mut Handle<ColorMaterial>, &ButtonType), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut material, button_type) in buttons.iter_mut() {
        update_button(&mut material, &materials, interaction, button_type, &states);
        if *interaction == Interaction::Clicked {
            match *button_type {
                ButtonType::Undo => states.undo(),
                ButtonType::Redo => states.redo(),
                ButtonType::Restart => states.reset(),
            }
        }
    }
}

pub fn update_greyed(
    states: Res<BoardStates>,
    mut buttons: Query<(&Interaction, &mut Handle<ColorMaterial>, &ButtonType)>,
    materials: Res<UIMaterials>,
) {
    if states.is_changed() {
        for (interaction, mut material, button_type) in buttons.iter_mut() {
            update_button(&mut material, &materials, interaction, button_type, &states);
        }
    }
}

fn update_button(
    material: &mut Handle<ColorMaterial>,
    materials: &UIMaterials,
    interaction: &Interaction,
    button_type: &ButtonType,
    states: &BoardStates,
) {
    if (*button_type == ButtonType::Undo && states.at_start()) 
    || (*button_type == ButtonType::Redo && states.at_end())
    || (*button_type == ButtonType::Restart && states.at_start()) {
        *material = materials.greyed.clone();
    } else {
        match *interaction {
            Interaction::Clicked => (),
            Interaction::Hovered => {
                *material = materials.hovered.clone();
            }
            Interaction::None => {
                *material = materials.normal.clone();
            }
        }
    }
}

fn button(
    parent: &mut ChildBuilder,
    materials: &UIMaterials,
    position: f32,
    style: TextStyle,
    text: &str,
    button_type: ButtonType,
) {
    parent.spawn_bundle(ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(100.0), Val::Px(40.0)),
            position_type: PositionType::Absolute,
            position: Rect {
                left: Val::Px(position),
                ..Default::default()
            },
            margin: Rect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        material: materials.greyed.clone(),
        ..Default::default()
    }).with_children(|parent| {
        parent.spawn_bundle(TextBundle {
            text: Text::with_section(
                text,
                style, 
                Default::default()
            ),
            ..Default::default()
        }).insert(FocusPolicy::Pass);
    }).insert(button_type);
}

pub fn setup_promotion(
    mut commands: Commands,
    textures: Res<Textures>,
    materials: Res<UIMaterials>,
    states: Res<BoardStates>,
    mut buttons: Query<&mut Handle<ColorMaterial>, With<ButtonType>>,
) {
    let colour = states.active().state.get_turn();

    let entity = commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Px(320.0), Val::Px(88.0)),
            position_type: PositionType::Absolute,
            position: Rect { 
                left: Val::Px(SCREEN_WIDTH / 2.0 + CENTRE_X - 160.0),
                bottom: Val::Px(SCREEN_HEIGHT / 2.0 + CENTRE_Y - 44.0),
                ..Default::default()
            },
            border: Rect::all(Val::Px(5.0)),
            ..Default::default()
        },
        material: materials.background.clone(),
        ..Default::default()
    }).with_children(|parent| {
        promotion_button(parent, &materials, Rect::all(Val::Auto), &textures.0, colour, Knight);
        promotion_button(parent, &materials, Rect::all(Val::Auto), &textures.0, colour, Bishop);
        promotion_button(parent, &materials, Rect::all(Val::Auto), &textures.0, colour, Rook);
        promotion_button(parent, &materials, Rect::all(Val::Auto), &textures.0, colour, Queen);
    }).id();
    commands.insert_resource(UIEntity(entity));

    for mut material in buttons.iter_mut() {
        *material = materials.greyed.clone();
    }
}

fn promotion_button(
    parent: &mut ChildBuilder,
    materials: &UIMaterials,
    margin: Rect<Val>,
    textures: &HashMap<Piece, Handle<ColorMaterial>>,
    colour: Colour,
    variant: Variant,
) {
    parent.spawn_bundle(ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(SQUARE_SIZE), Val::Px(SQUARE_SIZE)),
            border: Rect::all(Val::Auto),
            margin,
            ..Default::default()
        },
        material: materials.normal.clone(),
        ..Default::default()
    }).with_children(|parent| {
        parent.spawn_bundle(ImageBundle {
            style: Style {
                size: Size::new(Val::Px(60.0), Val::Px(60.0)),
                margin: Rect::all(Val::Auto),
                ..Default::default()
            },
            material: textures.get(&Piece { colour, variant }).unwrap().clone(),
            ..Default::default()
        }).insert(FocusPolicy::Pass);
    })
    .insert(PromoteTo(variant));
}

pub fn update_promotion(
    mut game_state: ResMut<State<GameState>>,
    mut states: ResMut<BoardStates>,
    materials: Res<UIMaterials>,
    mut buttons: Query<(&Interaction, &mut Handle<ColorMaterial>, &PromoteTo), (Changed<Interaction>, With<Button>, Without<ButtonType>)>,
) {
    for (interaction, mut material, to) in buttons.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                states.promote(to.0);
                if states.active().state.ended().is_some() {
                    game_state.set(GameState::End).unwrap();
                } else {
                    game_state.set(GameState::Playing).unwrap();
                }
            }
            Interaction::Hovered => {
                *material = materials.hovered.clone();
            }
            Interaction::None => {
                *material = materials.normal.clone();
            }
        }
    }
}

pub fn destruct_promotion(
    mut commands: Commands,
    entity: Res<UIEntity>,
) {
    commands.entity(entity.0).despawn_recursive();
}

pub fn setup_end_screen(
    mut commands: Commands,
    materials: Res<UIMaterials>,
    states: Res<BoardStates>,
    font: Res<FontAsset>,
    mut buttons: Query<(&mut Handle<ColorMaterial>, &ButtonType)>,
) {
    let text = match states.active().state.ended().clone().unwrap() {
        EndState::Checkmate(White) => "White wins by checkmate",
        EndState::Checkmate(Black) => "Black wins by checkmate",
        EndState::Stalemate => "Draw by stalemate",
        EndState::InsufficientMaterial => "Draw by insufficient material",
        EndState::ThreefoldRepetition => "Draw by threefold repetition",
    };
    
    let entity = commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Px(340.0), Val::Px(100.0)),
            position_type: PositionType::Absolute,
            position: Rect { 
                left: Val::Px(SCREEN_WIDTH / 2.0 + CENTRE_X - 170.0),
                bottom: Val::Px(SCREEN_HEIGHT / 2.0 + CENTRE_Y - 50.0),
                ..Default::default()
            },
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        material: materials.background.clone(),
        ..Default::default()
    }).with_children(|parent| {
        parent.spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect { top: Val::Px(10.0), bottom: Val::Auto, right: Val::Auto, left: Val::Auto },
                ..Default::default()
            },
            text: Text::with_section(
                text, 
                TextStyle {
                    font: font.0.clone(),
                    font_size: 30.0,
                    color: Color::rgba(1.0, 1.0, 1.0, 0.7),
                },
                Default::default()
            ),
            ..Default::default()
        });
        parent.spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(160.0), Val::Px(40.0)),
                position_type: PositionType::Absolute,
                position: Rect { top: Val::Auto, bottom: Val::Px(10.0), right: Val::Auto, left: Val::Auto },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: materials.normal.clone(),
            ..Default::default()
        }).with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Review game", 
                    TextStyle {
                        font: font.0.clone(),
                        font_size: 30.0,
                        color: Color::rgba(1.0, 1.0, 1.0, 0.7),
                    }, 
                    Default::default()
                ),
                ..Default::default()
            });
        });
    }).id();
    commands.insert_resource(UIEntity(entity));

    for (mut material, button_type) in buttons.iter_mut() {
        match *button_type {
            ButtonType::Restart => *material = materials.normal.clone(),
            ButtonType::Undo | ButtonType::Redo => *material = materials.greyed.clone(),
        }
    }
}

pub fn update_end_screen(
    mut states: ResMut<BoardStates>,
    mut game_state: ResMut<State<GameState>>,
    materials: Res<UIMaterials>,
    mut buttons: Query<(&Interaction, &mut Handle<ColorMaterial>, &ButtonType), Changed<Interaction>>,
    mut review_game: Query<(&Interaction, &mut Handle<ColorMaterial>), (Changed<Interaction>, With<Button>, Without<ButtonType>)>,
) {
    if let Ok((interaction, mut material)) = review_game.single_mut() {
        match *interaction {
            Interaction::Clicked => {
                game_state.set(GameState::Playing).unwrap();
            }
            Interaction::Hovered => {
                *material = materials.hovered.clone();
            }
            Interaction::None => {
                *material = materials.normal.clone();
            }
        }
    }

    for (interaction, mut material, button_type) in buttons.iter_mut() {
        if *button_type == ButtonType::Restart {
            match *interaction {
                Interaction::Clicked => {
                    states.reset();
                    game_state.set(GameState::Playing).unwrap();
                }
                Interaction::Hovered => {
                    *material = materials.hovered.clone();
                }
                Interaction::None => {
                    *material = materials.normal.clone();
                }
            }
        }
    }
}

pub fn destruct_end_screen(
    mut commands: Commands,
    entity: Res<UIEntity>,
) {
    commands.entity(entity.0).despawn_recursive();
}