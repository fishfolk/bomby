use bevy::prelude::*;
use bevy_ninepatch::*;
use iyes_loopless::prelude::*;

use bevy::app::AppExit;

use crate::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NinePatchPlugin::<()>::default())
            .add_startup_system_to_stage(StartupStage::PreStartup, load_font)
            .add_startup_system_to_stage(StartupStage::PreStartup, load_graphics)
            .add_enter_system(GameState::MainMenu, setup)
            .add_system(detect_button_presses.run_in_state(GameState::MainMenu))
            .add_exit_system(GameState::MainMenu, despawn_ui);
    }
}

#[derive(Component)]
enum MainMenuButton {
    Start,
    Exit,
}

fn detect_button_presses(
    mut commands: Commands,
    buttons: Query<(&MainMenuButton, &Interaction)>,
    mut exit: EventWriter<AppExit>,
) {
    for button in buttons
        .iter()
        .filter(|(_, state)| **state == Interaction::Clicked)
        .map(|b| b.0)
    {
        match button {
            MainMenuButton::Start => commands.insert_resource(NextState(GameState::LoadingLevel)),
            MainMenuButton::Exit => exit.send(AppExit),
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
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                padding: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Percent(25.0), Val::Px(0.0)),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .insert(DespawnOnExit)
        .add_child(start_button)
        .add_child(exit_button);
}

fn spawn_green_button_with_text(
    commands: &mut Commands,
    font: &Res<FontHandle>,
    ninepatch: &Res<ButtonNinePatch>,
    text: &str,
) -> Entity {
    let button_content = commands
        .spawn(TextBundle::from_section(
            text,
            TextStyle {
                font: font.0.clone(),
                font_size: 40.0,
                color: Color::WHITE,
            },
        ))
        .id();

    commands
        .spawn(NinePatchBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(10.0), Val::Px(0.0)),
                ..default()
            },
            nine_patch_data: NinePatchData::with_single_content(
                ninepatch.texture.clone(),
                ninepatch.ninepatch.clone(),
                button_content,
            ),
            ..default()
        })
        .insert(Interaction::None)
        .id()
}

#[derive(Resource)]
struct ButtonNinePatch {
    texture: Handle<Image>,
    ninepatch: Handle<NinePatchBuilder<()>>,
}

fn load_graphics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut nine_patches: ResMut<Assets<NinePatchBuilder<()>>>,
) {
    // NOTE: I am manually scaling the 9-patch assets to 200%. It may end up being beneficial to do
    // it programatically, for example dynamic window sizing.
    let button_ninepatch_texture = asset_server.load("ui/green-button.png");
    let button_ninepatch_handle = nine_patches.add(NinePatchBuilder::by_margins(6, 10, 6, 6));

    commands.insert_resource(ButtonNinePatch {
        texture: button_ninepatch_texture,
        ninepatch: button_ninepatch_handle,
    })
}

#[derive(Resource)]
struct FontHandle(Handle<Font>);

fn load_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("ui/ark-pixel-16px-latin.ttf");
    commands.insert_resource(FontHandle(font));
}
