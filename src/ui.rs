use bevy::prelude::*;

use bevy::app::AppExit;

use crate::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, (load_font, load_graphics))
            .add_systems(OnEnter(GameState::MainMenu), setup)
            .add_systems(
                Update,
                detect_button_presses.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnExit(GameState::MainMenu), despawn_ui);
    }
}

#[derive(Component)]
enum MainMenuButton {
    Start,
    Exit,
}

fn detect_button_presses(
    buttons: Query<(&MainMenuButton, &Interaction)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for button in buttons
        .iter()
        .filter(|(_, state)| **state == Interaction::Pressed)
        .map(|b| b.0)
    {
        match button {
            MainMenuButton::Start => next_state.set(GameState::LoadingLevel),
            MainMenuButton::Exit => {
                exit.send(AppExit::Success);
            }
        }
    }
}

#[derive(Component)]
struct DespawnOnExit;

fn despawn_ui(mut commands: Commands, to_despawn: Query<Entity, With<DespawnOnExit>>) {
    to_despawn
        .iter()
        .for_each(|e| commands.entity(e).despawn_recursive());
}

fn setup(mut commands: Commands, font: Res<FontHandle>, button: Res<ButtonNinePatch>) {
    let start_button = spawn_green_button_with_text(&mut commands, &font, &button, "Start Game");
    let start_button = commands
        .entity(start_button)
        .insert(MainMenuButton::Start)
        .id();

    let exit_button = spawn_green_button_with_text(&mut commands, &font, &button, "Exit");
    let exit_button = commands
        .entity(exit_button)
        .insert(MainMenuButton::Exit)
        .id();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::top(Val::Percent(25.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..default()
            },
            DespawnOnExit,
        ))
        .add_child(start_button)
        .add_child(exit_button);
}

fn spawn_green_button_with_text(
    commands: &mut Commands,
    font: &Res<FontHandle>,
    ninepatch: &Res<ButtonNinePatch>,
    text: &str,
) -> Entity {
    let mut cmd = commands.spawn((
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(10.0)),
            ..default()
        },
        Sprite {
            image: ninepatch.texture.clone(),
            image_mode: SpriteImageMode::Sliced(ninepatch.ninepatch.clone()),
            ..default()
        },
        Interaction::None,
    ));

    cmd.with_children(|builder| {
        builder.spawn((
            Text2d::new(text),
            TextFont {
                font: font.0.clone(),
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(JustifyText::Center),
            bevy::sprite::Anchor::Center,
        ));
    });

    cmd.id()
}

#[derive(Resource)]
struct ButtonNinePatch {
    texture: Handle<Image>,
    ninepatch: TextureSlicer,
}

fn load_graphics(mut commands: Commands, asset_server: Res<AssetServer>) {
    // NOTE: I am manually scaling the 9-patch assets to 200%. It may end up being beneficial to do
    // it programatically, for example dynamic window sizing.
    let button_ninepatch_texture = asset_server.load("ui/green-button.png");
    let button_ninepatch_slicer = TextureSlicer {
        border: BorderRect {
            top: 6.0,
            bottom: 10.0,
            left: 6.0,
            right: 6.0,
        },
        ..default()
    };

    commands.insert_resource(ButtonNinePatch {
        texture: button_ninepatch_texture,
        ninepatch: button_ninepatch_slicer,
    });
}

#[derive(Resource)]
struct FontHandle(Handle<Font>);

fn load_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("ui/ark-pixel-16px-latin.ttf");
    commands.insert_resource(FontHandle(font));
}
