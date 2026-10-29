use bevy::prelude::*;

use bevy::text::FontSourceTemplate;
use bevy::{
    app::AppExit,
    ui::widget::NodeImageMode,
    ui_widgets::{Activate, Button},
};

use crate::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, (load_font, load_graphics))
            .add_systems(OnEnter(GameState::MainMenu), setup)
            .add_systems(OnExit(GameState::MainMenu), despawn_ui);
    }
}

#[derive(Component, Clone, Copy, Default)]
struct DespawnOnExit;

fn despawn_ui(mut commands: Commands, to_despawn: Query<Entity, With<DespawnOnExit>>) {
    to_despawn.iter().for_each(|e| commands.entity(e).despawn());
}

fn setup(mut commands: Commands, font: Res<FontHandle>, button: Res<ButtonNinePatch>) {
    commands.spawn_scene(bsn! {
        DespawnOnExit
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::top(Val::Percent(25.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
        }
        Children [
            @green_button_with_text(
                font.0.clone(),
                &button,
                "Start Game".to_string(),
            )
            on(|_: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
                next_state.set(GameState::LoadingLevel);
            })
            --
            @green_button_with_text(
                font.0.clone(),
                &button,
                "Exit".to_string(),
            )
            on(|_: On<Activate>, mut m_exit: MessageWriter<AppExit>| {
                m_exit.write(AppExit::Success);
            })
        ]
    });
}

fn green_button_with_text(
    font: Handle<Font>,
    ninepatch: &ButtonNinePatch,
    text: String,
) -> impl Scene {
    let image = ninepatch.texture.clone();
    let ninepatch = ninepatch.ninepatch.clone();
    bsn! {
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(10.0)),
        }
        ImageNode {
            image,
            image_mode: NodeImageMode::Sliced(ninepatch)
        }
        Button
        Children [
            Node {
                padding: UiRect::all(Val::Px(8.0)),
            }
            Children [
                Text::new(text)
                TextFont {
                    font: FontSourceTemplate::Handle(font),
                    font_size: FontSize::Px(40.0),
                }
                TextColor(Color::WHITE)
                TextLayout::justify(Justify::Center)
                bevy::sprite::Anchor::CENTER
            ]
        ]
    }
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
            min_inset: (6.0, 6.0).into(),
            max_inset: (6.0, 10.0).into(),
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
