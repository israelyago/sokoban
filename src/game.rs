use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::{Path, PathBuf},
};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use winit::window::Icon;

use crate::{
    coord::Direction,
    level::{Level, LevelPack},
    paths::PathConfig,
    rules::try_step,
    state::GameState,
};

const UNDO_LIMIT: usize = 10_000;
const TILE_SIZE: f32 = 36.0;
const COLLECTION_MENU_MIN_VISIBLE_ROWS: usize = 3;
const COLLECTION_NAME_MAX_CHARS: usize = 64;
const GAME_BACKGROUND_COLOR: Color = Color::srgb(0.18, 0.18, 0.18);
const BUILD_VERSION: &str = env!("CARGO_PKG_VERSION");
const MOVE_REPEAT_INITIAL_DELAY_SECS: f32 = 0.35;
const MOVE_REPEAT_INTERVAL_SECS: f32 = 0.03;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum UiKey {
    Arrows,
    ArrowsVertical,
    FlairArrowsUp,
    FlairArrowsDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Enter,
    Space,
    Escape,
    Delete,
    Backspace,
    KeyA,
    KeyD,
    KeyF2,
    KeyI,
    KeyN,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyU,
    KeyW,
    KeyY,
}

#[derive(Debug, Clone, Copy)]
struct IconVariant {
    default_path: &'static str,
    outline_path: &'static str,
}

#[derive(Resource, Debug, Clone)]
struct KeyIconRegistry {
    variants: HashMap<UiKey, IconVariant>,
}

impl KeyIconRegistry {
    fn icon_path(&self, key: UiKey) -> &'static str {
        self.icon_path_with_outline(key, false)
    }

    fn icon_path_with_outline(&self, key: UiKey, use_outline: bool) -> &'static str {
        let Some(icon) = self.variants.get(&key) else {
            return "assets/icons/keyboard_question.png";
        };

        if use_outline {
            icon.outline_path
        } else {
            icon.default_path
        }
    }
}

impl Default for KeyIconRegistry {
    fn default() -> Self {
        let mut variants = HashMap::new();
        // Non-outline paths are the default; outline is opt-in only.
        variants.insert(
            UiKey::Arrows,
            IconVariant {
                default_path: "icons/keyboard_arrows.png",
                outline_path: "icons/keyboard_arrows_outline.png",
            },
        );
        variants.insert(
            UiKey::ArrowsVertical,
            IconVariant {
                default_path: "icons/keyboard_arrows_vertical.png",
                outline_path: "icons/keyboard_arrows_vertical_outline.png",
            },
        );
        variants.insert(
            UiKey::FlairArrowsUp,
            IconVariant {
                default_path: "icons/flair_arrows_up.png",
                outline_path: "icons/flair_arrows_up.png",
            },
        );
        variants.insert(
            UiKey::FlairArrowsDown,
            IconVariant {
                default_path: "icons/flair_arrows_down.png",
                outline_path: "icons/flair_arrows_down.png",
            },
        );
        variants.insert(
            UiKey::ArrowUp,
            IconVariant {
                default_path: "icons/keyboard_arrow_up.png",
                outline_path: "icons/keyboard_arrow_up_outline.png",
            },
        );
        variants.insert(
            UiKey::ArrowDown,
            IconVariant {
                default_path: "icons/keyboard_arrow_down.png",
                outline_path: "icons/keyboard_arrow_down_outline.png",
            },
        );
        variants.insert(
            UiKey::ArrowLeft,
            IconVariant {
                default_path: "icons/keyboard_arrow_left.png",
                outline_path: "icons/keyboard_arrow_left_outline.png",
            },
        );
        variants.insert(
            UiKey::ArrowRight,
            IconVariant {
                default_path: "icons/keyboard_arrow_right.png",
                outline_path: "icons/keyboard_arrow_right_outline.png",
            },
        );
        variants.insert(
            UiKey::Enter,
            IconVariant {
                default_path: "icons/keyboard_enter.png",
                outline_path: "icons/keyboard_enter_outline.png",
            },
        );
        variants.insert(
            UiKey::Space,
            IconVariant {
                default_path: "icons/keyboard_space.png",
                outline_path: "icons/keyboard_space_outline.png",
            },
        );
        variants.insert(
            UiKey::Escape,
            IconVariant {
                default_path: "icons/keyboard_escape.png",
                outline_path: "icons/keyboard_escape_outline.png",
            },
        );
        variants.insert(
            UiKey::Delete,
            IconVariant {
                default_path: "icons/keyboard_delete.png",
                outline_path: "icons/keyboard_delete_outline.png",
            },
        );
        variants.insert(
            UiKey::Backspace,
            IconVariant {
                default_path: "icons/keyboard_backspace.png",
                outline_path: "icons/keyboard_backspace_outline.png",
            },
        );
        variants.insert(
            UiKey::KeyA,
            IconVariant {
                default_path: "icons/keyboard_a.png",
                outline_path: "icons/keyboard_a_outline.png",
            },
        );
        variants.insert(
            UiKey::KeyD,
            IconVariant {
                default_path: "icons/keyboard_d.png",
                outline_path: "icons/keyboard_d_outline.png",
            },
        );
        variants.insert(
            UiKey::KeyF2,
            IconVariant {
                default_path: "icons/keyboard_f2.png",
                outline_path: "icons/keyboard_f2_outline.png",
            },
        );
        variants.insert(
            UiKey::KeyI,
            IconVariant {
                default_path: "icons/keyboard_i.png",
                outline_path: "icons/keyboard_i_outline.png",
            },
        );
        variants.insert(
            UiKey::KeyN,
            IconVariant {
                default_path: "icons/keyboard_n.png",
                outline_path: "icons/keyboard_n_outline.png",
            },
        );
        variants.insert(
            UiKey::KeyP,
            IconVariant {
                default_path: "icons/keyboard_p.png",
                outline_path: "icons/keyboard_p_outline.png",
            },
        );
        variants.insert(
            UiKey::KeyQ,
            IconVariant {
                default_path: "icons/keyboard_q.png",
                outline_path: "icons/keyboard_q_outline.png",
            },
        );
        variants.insert(
            UiKey::KeyR,
            IconVariant {
                default_path: "icons/keyboard_r.png",
                outline_path: "icons/keyboard_r_outline.png",
            },
        );
        variants.insert(
            UiKey::KeyS,
            IconVariant {
                default_path: "icons/keyboard_s.png",
                outline_path: "icons/keyboard_s_outline.png",
            },
        );
        variants.insert(
            UiKey::KeyU,
            IconVariant {
                default_path: "icons/keyboard_u.png",
                outline_path: "icons/keyboard_u_outline.png",
            },
        );
        variants.insert(
            UiKey::KeyW,
            IconVariant {
                default_path: "icons/keyboard_w.png",
                outline_path: "icons/keyboard_w_outline.png",
            },
        );
        variants.insert(
            UiKey::KeyY,
            IconVariant {
                default_path: "icons/keyboard_y.png",
                outline_path: "icons/keyboard_y_outline.png",
            },
        );
        Self { variants }
    }
}

fn validate_key_icon_registry(registry: Res<KeyIconRegistry>) {
    let _ = registry.icon_path(UiKey::ArrowUp);
    let _ = registry.icon_path(UiKey::Enter);
    let _ = registry.icon_path_with_outline(UiKey::Enter, true);
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    CollectionMenu,
    Loading,
    Playing,
    LevelComplete,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MainMenuAction {
    Play,
    Exit,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
struct MainMenuSelection(pub usize);

#[derive(Resource, Debug, Clone, Copy, Default)]
struct CollectionMenuSelection(pub usize);

#[derive(Debug, Clone)]
struct CollectionEntry {
    display_name: String,
    path: String,
}

#[derive(Resource, Debug, Clone, Default)]
struct CollectionsResource {
    entries: Vec<CollectionEntry>,
}

#[derive(Resource, Debug, Clone, Default)]
struct CollectionMenuStatus(pub Option<String>);

#[derive(Resource, Debug, Clone, Default)]
struct RenameState {
    active: bool,
    source_path: Option<String>,
    source_display_name: Option<String>,
    buffer: String,
}

#[derive(Resource, Debug, Clone, Default)]
struct DeleteConfirmState {
    active: bool,
    source_path: Option<String>,
    source_display_name: Option<String>,
}

#[derive(Resource, Debug, Clone)]
struct SelectedPackPath(pub String);

impl Default for SelectedPackPath {
    fn default() -> Self {
        Self(String::new())
    }
}

#[derive(Resource, Debug, Clone)]
pub struct StartupConfig {
    pub pack_path: String,
    pub start_level: usize,
}

#[derive(Resource, Debug, Clone)]
pub struct LoadedLevels {
    pub levels: Vec<Level>,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct ActiveLevelIndex(pub usize);

#[derive(Resource, Debug, Clone)]
pub struct CurrentGameState(pub GameState);

#[derive(Resource, Debug, Clone, Default)]
pub struct UndoHistory(pub VecDeque<GameState>);

#[derive(Resource, Debug)]
struct MoveRepeatState {
    held_direction: Option<Direction>,
    initial_delay: Timer,
    repeat_interval: Timer,
}

impl Default for MoveRepeatState {
    fn default() -> Self {
        Self {
            held_direction: None,
            initial_delay: Timer::from_seconds(MOVE_REPEAT_INITIAL_DELAY_SECS, TimerMode::Once),
            repeat_interval: Timer::from_seconds(MOVE_REPEAT_INTERVAL_SECS, TimerMode::Repeating),
        }
    }
}

impl MoveRepeatState {
    fn reset(&mut self) {
        self.held_direction = None;
        self.initial_delay.reset();
        self.repeat_interval.reset();
    }

    fn start_direction(&mut self, direction: Direction) {
        self.held_direction = Some(direction);
        self.initial_delay.reset();
        self.repeat_interval.reset();
    }
}

#[derive(Resource, Debug)]
struct UndoRepeatState {
    initial_delay: Timer,
    repeat_interval: Timer,
}

impl Default for UndoRepeatState {
    fn default() -> Self {
        Self {
            initial_delay: Timer::from_seconds(MOVE_REPEAT_INITIAL_DELAY_SECS, TimerMode::Once),
            repeat_interval: Timer::from_seconds(MOVE_REPEAT_INTERVAL_SECS, TimerMode::Repeating),
        }
    }
}

impl UndoRepeatState {
    fn reset(&mut self) {
        self.initial_delay.reset();
        self.repeat_interval.reset();
    }
}

#[derive(Resource, Debug, Clone)]
struct SpriteAssets {
    wall: Handle<Image>,
    floor: Handle<Image>,
    goal: Handle<Image>,
    box_tile: Handle<Image>,
    box_on_goal: Handle<Image>,
    player: Handle<Image>,
    player_on_goal: Handle<Image>,
}

#[derive(Component)]
struct BoardEntity;

#[derive(Component)]
struct Tile;

#[derive(Component)]
struct BoxEntity;

#[derive(Component)]
struct PlayerEntity;

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct HudLevelText;

#[derive(Component)]
struct HudMovesText;

#[derive(Component)]
struct HudPushesText;

#[derive(Component)]
struct WinOverlay;

#[derive(Component)]
struct BuildVersionText;

#[derive(Component)]
struct MainMenuRoot;

#[derive(Component)]
struct MainMenuItem {
    index: usize,
    action: MainMenuAction,
}

#[derive(Component)]
struct CollectionMenuRoot;

#[derive(Component)]
struct CollectionMenuItem {
    index: usize,
    display_name: String,
}

#[derive(Component)]
struct CollectionMenuStatusText;

#[derive(Component)]
struct CollectionMenuDefaultHints;

#[derive(Component)]
struct CollectionMenuRenameHints;

#[derive(Component)]
struct CollectionMenuDeleteHints;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .insert_resource(ClearColor(GAME_BACKGROUND_COLOR))
            .init_resource::<MainMenuSelection>()
            .init_resource::<CollectionMenuSelection>()
            .init_resource::<KeyIconRegistry>()
            .init_resource::<CollectionsResource>()
            .init_resource::<CollectionMenuStatus>()
            .init_resource::<RenameState>()
            .init_resource::<DeleteConfirmState>()
            .init_resource::<SelectedPackPath>()
            .init_resource::<MoveRepeatState>()
            .init_resource::<UndoRepeatState>()
            .add_systems(Startup, (setup_camera, load_sprite_assets))
            .add_systems(OnEnter(AppState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(AppState::MainMenu), despawn_main_menu)
            .add_systems(
                OnEnter(AppState::CollectionMenu),
                (discover_collections, spawn_collection_menu).chain(),
            )
            .add_systems(OnExit(AppState::CollectionMenu), despawn_collection_menu)
            .add_systems(OnEnter(AppState::Loading), load_levels)
            .add_systems(
                OnEnter(AppState::Playing),
                (setup_level_state, spawn_board).chain(),
            )
            .add_systems(
                Update,
                (handle_main_menu_input, update_main_menu_visuals)
                    .run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(
                Update,
                (
                    handle_collection_menu_input,
                    refresh_collection_menu,
                    update_collection_menu_visuals,
                    update_collection_menu_status_text,
                    update_collection_menu_hint_visibility,
                )
                    .chain()
                    .run_if(in_state(AppState::CollectionMenu)),
            )
            .add_systems(
                Update,
                (
                    handle_back_to_collection_input,
                    handle_lifecycle_input,
                    handle_move_input,
                    update_hud_text,
                    sync_completion_state,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (sync_static_board, sync_dynamic_entities)
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    handle_back_to_collection_input,
                    handle_lifecycle_input,
                    handle_move_input,
                    update_hud_text,
                    sync_completion_state,
                )
                    .run_if(in_state(AppState::LevelComplete)),
            )
            .add_systems(
                Update,
                (sync_static_board, sync_dynamic_entities)
                    .chain()
                    .run_if(in_state(AppState::LevelComplete)),
            )
            .add_systems(
                Startup,
                (
                    spawn_hud,
                    spawn_win_overlay,
                    spawn_build_version_text,
                    update_win_overlay_visibility,
                    validate_key_icon_registry,
                ),
            )
            .add_systems(
                Update,
                (
                    set_window_icon_once,
                    update_hud_visibility,
                    update_win_overlay_visibility,
                ),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn load_sprite_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player = asset_server.load("player.png");
    commands.insert_resource(SpriteAssets {
        wall: asset_server.load("wall.png"),
        floor: asset_server.load("floor.png"),
        goal: asset_server.load("goal.png"),
        box_tile: asset_server.load("box.png"),
        box_on_goal: asset_server.load("box_on_goal.png"),
        player: player.clone(),
        // M3 request: use player.png for player-on-goal visual.
        player_on_goal: player,
    });
}

fn spawn_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icon_registry: Res<KeyIconRegistry>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mut bundle = NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        ..default()
    };
    bundle.visibility = Visibility::Hidden;

    commands.spawn((bundle, HudText)).with_children(|parent| {
        spawn_hud_value_row(parent, &font, "Level", "1/1 - (unnamed)", HudLevelText);
        spawn_hud_value_row(parent, &font, "Moves", "0", HudMovesText);
        spawn_hud_value_row(parent, &font, "Pushes", "0", HudPushesText);

        spawn_hud_action_row(
            parent,
            &asset_server,
            &icon_registry,
            &font,
            UiKey::Arrows,
            "Move player",
        );
        spawn_hud_action_row(
            parent,
            &asset_server,
            &icon_registry,
            &font,
            UiKey::KeyU,
            "Undo",
        );
        spawn_hud_action_row(
            parent,
            &asset_server,
            &icon_registry,
            &font,
            UiKey::KeyR,
            "Restart",
        );
        spawn_hud_action_row(
            parent,
            &asset_server,
            &icon_registry,
            &font,
            UiKey::KeyN,
            "Next level",
        );
        spawn_hud_action_row(
            parent,
            &asset_server,
            &icon_registry,
            &font,
            UiKey::KeyP,
            "Previous level",
        );
        spawn_hud_action_row(
            parent,
            &asset_server,
            &icon_registry,
            &font,
            UiKey::Escape,
            "Back to menu",
        );
    });
}

fn spawn_hud_value_row<T: Component>(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    label: &str,
    value: &str,
    marker: T,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            row.spawn(TextBundle::from_section(
                format!("{label}:"),
                TextStyle {
                    font: font.clone(),
                    font_size: 20.0,
                    color: Color::srgb(0.82, 0.88, 0.93),
                },
            ));
            row.spawn((
                TextBundle::from_section(
                    value,
                    TextStyle {
                        font: font.clone(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ),
                marker,
            ));
        });
}

fn spawn_hud_action_row(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    icon_registry: &KeyIconRegistry,
    font: &Handle<Font>,
    key: UiKey,
    text: &str,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            spawn_key_icon(row, asset_server, icon_registry, key);
            row.spawn(TextBundle::from_section(
                format!(": {text}"),
                TextStyle {
                    font: font.clone(),
                    font_size: 18.0,
                    color: Color::srgb(0.78, 0.84, 0.9),
                },
            ));
        });
}

fn spawn_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icon_registry: Res<KeyIconRegistry>,
    mut selection: ResMut<MainMenuSelection>,
) {
    selection.0 = 0;
    let title_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let item_font = title_font.clone();

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    ..default()
                },
                background_color: BackgroundColor(GAME_BACKGROUND_COLOR),
                ..default()
            },
            MainMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Sokoban",
                TextStyle {
                    font: title_font,
                    font_size: 56.0,
                    color: Color::WHITE,
                },
            ));

            parent.spawn((
                TextBundle::from_section(
                    "Play",
                    TextStyle {
                        font: item_font.clone(),
                        font_size: 34.0,
                        color: Color::WHITE,
                    },
                ),
                MainMenuItem {
                    index: 0,
                    action: MainMenuAction::Play,
                },
            ));

            parent.spawn((
                TextBundle::from_section(
                    "Exit",
                    TextStyle {
                        font: item_font,
                        font_size: 34.0,
                        color: Color::WHITE,
                    },
                ),
                MainMenuItem {
                    index: 1,
                    action: MainMenuAction::Exit,
                },
            ));

            spawn_main_menu_icon_hint(
                parent,
                &asset_server,
                &icon_registry,
                asset_server.load("fonts/FiraSans-Bold.ttf"),
            );
        });
}

fn despawn_main_menu(mut commands: Commands, roots: Query<Entity, With<MainMenuRoot>>) {
    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
}

fn handle_main_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<MainMenuSelection>,
    mut next_state: ResMut<NextState<AppState>>,
    items: Query<&MainMenuItem>,
    mut exit: EventWriter<AppExit>,
) {
    let item_count = items.iter().count();
    if item_count == 0 {
        return;
    }

    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        selection.0 = if selection.0 == 0 {
            item_count - 1
        } else {
            selection.0 - 1
        };
    }

    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        selection.0 = (selection.0 + 1) % item_count;
    }

    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        let action = items
            .iter()
            .find(|item| item.index == selection.0)
            .map(|item| item.action);

        match action {
            Some(MainMenuAction::Play) => {
                next_state.set(AppState::CollectionMenu);
            }
            Some(MainMenuAction::Exit) => {
                exit.send(AppExit::Success);
            }
            None => {}
        }
    }
}

fn update_main_menu_visuals(
    selection: Res<MainMenuSelection>,
    mut items: Query<(&MainMenuItem, &mut Text)>,
) {
    for (item, mut text) in &mut items {
        let (label, color) = if item.index == selection.0 {
            (
                format!("> {}", action_label(item.action)),
                Color::srgb(1.0, 0.9, 0.35),
            )
        } else {
            (format!("  {}", action_label(item.action)), Color::WHITE)
        };
        text.sections[0].value = label;
        text.sections[0].style.color = color;
    }
}

fn action_label(action: MainMenuAction) -> &'static str {
    match action {
        MainMenuAction::Play => "Play",
        MainMenuAction::Exit => "Exit",
    }
}

fn spawn_main_menu_icon_hint(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    icon_registry: &KeyIconRegistry,
    font: Handle<Font>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                left: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(8.0),
                ..default()
            },
            ..default()
        })
        .with_children(|column| {
            column
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    spawn_key_icon_with_outline(
                        row,
                        asset_server,
                        icon_registry,
                        UiKey::ArrowsVertical,
                        true,
                    );
                    row.spawn(TextBundle::from_section(
                        ": Selection",
                        TextStyle {
                            font: font.clone(),
                            font_size: 18.0,
                            color: Color::srgb(0.75, 0.8, 0.85),
                        },
                    ));
                });

            column
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    spawn_key_icon(row, asset_server, icon_registry, UiKey::Enter);
                    row.spawn(TextBundle::from_section(
                        ": Confirm",
                        TextStyle {
                            font: font.clone(),
                            font_size: 18.0,
                            color: Color::srgb(0.75, 0.8, 0.85),
                        },
                    ));
                });
        });
}

fn spawn_key_icon(
    row: &mut ChildBuilder,
    asset_server: &AssetServer,
    icon_registry: &KeyIconRegistry,
    key: UiKey,
) {
    spawn_key_icon_with_outline(row, asset_server, icon_registry, key, false);
}

fn spawn_key_icon_with_outline(
    row: &mut ChildBuilder,
    asset_server: &AssetServer,
    icon_registry: &KeyIconRegistry,
    key: UiKey,
    use_outline: bool,
) {
    row.spawn(ImageBundle {
        style: Style {
            width: Val::Px(32.0),
            height: Val::Px(32.0),
            ..default()
        },
        image: UiImage::new(
            asset_server.load(icon_registry.icon_path_with_outline(key, use_outline)),
        ),
        ..default()
    });
}

fn discover_collections(
    mut collections: ResMut<CollectionsResource>,
    mut status: ResMut<CollectionMenuStatus>,
    path_config: Res<PathConfig>,
) {
    match discover_collections_from_disk(path_config.as_ref()) {
        Ok(entries) => {
            collections.entries = entries;
        }
        Err(err) => {
            status.0 = Some(err);
            collections.entries = vec![default_collection_entry(path_config.as_ref())];
        }
    }
}

fn spawn_collection_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icon_registry: Res<KeyIconRegistry>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut selection: ResMut<CollectionMenuSelection>,
    selected_pack: Res<SelectedPackPath>,
    collections: Res<CollectionsResource>,
    status: Res<CollectionMenuStatus>,
    rename_state: Res<RenameState>,
    delete_confirm: Res<DeleteConfirmState>,
) {
    let selected_index = collections
        .entries
        .iter()
        .position(|entry| entry.path == selected_pack.0)
        .unwrap_or(0)
        .min(collections.entries.len().saturating_sub(1));
    selection.0 = selected_index;
    let window_height = primary_window
        .get_single()
        .map(|window| window.height())
        .unwrap_or(768.0);

    spawn_collection_menu_entities(
        &mut commands,
        &asset_server,
        &icon_registry,
        selection.0,
        &collections.entries,
        status.0.as_deref(),
        rename_state.active,
        delete_confirm.active,
        window_height,
    );
}

fn despawn_collection_menu(mut commands: Commands, roots: Query<Entity, With<CollectionMenuRoot>>) {
    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
}

fn handle_collection_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    path_config: Res<PathConfig>,
    mut selection: ResMut<CollectionMenuSelection>,
    mut collections: ResMut<CollectionsResource>,
    mut selected_pack: ResMut<SelectedPackPath>,
    mut status: ResMut<CollectionMenuStatus>,
    mut rename_state: ResMut<RenameState>,
    mut delete_confirm: ResMut<DeleteConfirmState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if rename_state.active {
        handle_rename_mode_input(
            &keys,
            path_config.as_ref(),
            &mut rename_state,
            &mut collections,
            &mut selection,
            &mut selected_pack,
            &mut status,
        );
        return;
    }

    if delete_confirm.active {
        handle_delete_confirm_input(
            &keys,
            path_config.as_ref(),
            &mut delete_confirm,
            &mut collections,
            &mut selection,
            &mut selected_pack,
            &mut status,
        );
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
        return;
    }

    if keys.just_pressed(KeyCode::KeyI) {
        match import_collection(path_config.as_ref()) {
            Ok(imported_path) => match discover_collections_from_disk(path_config.as_ref()) {
                Ok(entries) => {
                    collections.entries = entries;
                    if let Some(index) = collections
                        .entries
                        .iter()
                        .position(|entry| entry.path == imported_path)
                    {
                        selection.0 = index;
                    }
                    selected_pack.0 = imported_path;
                    status.0 = None;
                }
                Err(err) => {
                    status.0 = Some(err);
                }
            },
            Err(err) => {
                status.0 = Some(err);
            }
        }
        return;
    }

    let item_count = collections.entries.len();
    if item_count == 0 {
        return;
    }

    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        selection.0 = if selection.0 == 0 {
            item_count - 1
        } else {
            selection.0 - 1
        };
    }

    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        selection.0 = (selection.0 + 1) % item_count;
    }

    if (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space))
        && let Some(item) = collections.entries.get(selection.0)
    {
        selected_pack.0 = item.path.clone();
        status.0 = None;
        next_state.set(AppState::Loading);
    }

    if (keys.just_pressed(KeyCode::KeyR) || keys.just_pressed(KeyCode::F2))
        && let Some(item) = collections.entries.get(selection.0)
    {
        if item.path == default_collection_path(path_config.as_ref()) {
            status.0 = Some("Built-in default collection cannot be renamed".to_string());
            return;
        }

        rename_state.active = true;
        rename_state.source_path = Some(item.path.clone());
        rename_state.source_display_name = Some(item.display_name.clone());
        rename_state.buffer = item.display_name.clone();
        status.0 = Some("Renaming".to_string());
    }

    if keys.just_pressed(KeyCode::Delete)
        && let Some(item) = collections.entries.get(selection.0)
    {
        if item.path == default_collection_path(path_config.as_ref()) {
            status.0 = Some("Built-in default collection cannot be deleted".to_string());
            return;
        }

        delete_confirm.active = true;
        delete_confirm.source_path = Some(item.path.clone());
        delete_confirm.source_display_name = Some(item.display_name.clone());
        status.0 = Some(format!("Delete '{}' ? (Y/N)", item.display_name));
    }
}

fn update_collection_menu_visuals(
    selection: Res<CollectionMenuSelection>,
    rename_state: Res<RenameState>,
    mut items: Query<(&CollectionMenuItem, &mut Text)>,
) {
    for (item, mut text) in &mut items {
        let is_selected = item.index == selection.0;
        let is_renaming_selected = is_selected && rename_state.active;
        let selected_label = if is_renaming_selected {
            format!("> {}_", rename_state.buffer)
        } else {
            format!("> {}", item.display_name)
        };

        let (label, color) = if is_selected {
            (selected_label, Color::srgb(1.0, 0.9, 0.35))
        } else {
            (format!("  {}", item.display_name), Color::WHITE)
        };
        text.sections[0].value = label;
        text.sections[0].style.color = color;
    }
}

fn update_collection_menu_status_text(
    status: Res<CollectionMenuStatus>,
    rename_state: Res<RenameState>,
    mut query: Query<&mut Text, With<CollectionMenuStatusText>>,
) {
    if !(status.is_changed() || rename_state.is_changed()) {
        return;
    }

    let status_value = if rename_state.active {
        match status.0.as_deref() {
            Some(msg) if msg != "Renaming" => msg.to_string(),
            _ => "Renaming".to_string(),
        }
    } else {
        status.0.clone().unwrap_or_default()
    };

    for mut text in &mut query {
        text.sections[0].value = status_value.clone();
    }
}

fn refresh_collection_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icon_registry: Res<KeyIconRegistry>,
    primary_window: Query<Ref<Window>, With<PrimaryWindow>>,
    selection: Res<CollectionMenuSelection>,
    collections: Res<CollectionsResource>,
    status: Res<CollectionMenuStatus>,
    rename_state: Res<RenameState>,
    delete_confirm: Res<DeleteConfirmState>,
    roots: Query<Entity, With<CollectionMenuRoot>>,
) {
    let (window_height, window_changed) = match primary_window.get_single() {
        Ok(window) => (window.height(), window.is_changed()),
        Err(_) => (768.0, false),
    };

    if !(collections.is_changed()
        || selection.is_changed()
        || rename_state.is_changed()
        || delete_confirm.is_changed()
        || window_changed)
    {
        return;
    }

    rebuild_collection_menu(
        &mut commands,
        &asset_server,
        &icon_registry,
        &selection,
        &collections,
        &status,
        &rename_state,
        &delete_confirm,
        window_height,
        &roots,
    );
}

fn rebuild_collection_menu(
    commands: &mut Commands,
    asset_server: &AssetServer,
    icon_registry: &KeyIconRegistry,
    selection: &CollectionMenuSelection,
    collections: &CollectionsResource,
    status: &CollectionMenuStatus,
    rename_state: &RenameState,
    delete_confirm: &DeleteConfirmState,
    window_height: f32,
    roots: &Query<Entity, With<CollectionMenuRoot>>,
) {
    for root in roots {
        commands.entity(root).despawn_recursive();
    }

    let selected_index = selection.0.min(collections.entries.len().saturating_sub(1));

    spawn_collection_menu_entities(
        commands,
        asset_server,
        icon_registry,
        selected_index,
        &collections.entries,
        status.0.as_deref(),
        rename_state.active,
        delete_confirm.active,
        window_height,
    );
}

fn spawn_collection_menu_entities(
    commands: &mut Commands,
    asset_server: &AssetServer,
    icon_registry: &KeyIconRegistry,
    selected_index: usize,
    entries: &[CollectionEntry],
    status: Option<&str>,
    rename_active: bool,
    delete_active: bool,
    window_height: f32,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let visible_rows = collection_menu_visible_rows(window_height);
    let (start_index, end_index) =
        collection_menu_visible_range(selected_index, entries.len(), visible_rows);
    let has_more_above = start_index > 0;
    let has_more_below = end_index < entries.len();

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                background_color: BackgroundColor(GAME_BACKGROUND_COLOR),
                ..default()
            },
            CollectionMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Select Collection",
                TextStyle {
                    font: font.clone(),
                    font_size: 44.0,
                    color: Color::WHITE,
                },
            ));

            let mut top_marker = NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            };
            top_marker.visibility = if has_more_above {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            parent.spawn(top_marker).with_children(|row| {
                spawn_key_icon(row, asset_server, icon_registry, UiKey::FlairArrowsUp);
            });

            for (index, entry) in entries
                .iter()
                .enumerate()
                .skip(start_index)
                .take(end_index - start_index)
            {
                let label = if index == selected_index {
                    format!("> {}", entry.display_name)
                } else {
                    format!("  {}", entry.display_name)
                };
                let color = if index == selected_index {
                    Color::srgb(1.0, 0.9, 0.35)
                } else {
                    Color::WHITE
                };

                parent.spawn((
                    TextBundle::from_section(
                        label,
                        TextStyle {
                            font: font.clone(),
                            font_size: 30.0,
                            color,
                        },
                    ),
                    CollectionMenuItem {
                        index,
                        display_name: entry.display_name.clone(),
                    },
                ));
            }

            let mut bottom_marker = NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            };
            bottom_marker.visibility = if has_more_below {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            parent.spawn(bottom_marker).with_children(|row| {
                spawn_key_icon(row, asset_server, icon_registry, UiKey::FlairArrowsDown);
            });

            spawn_collection_menu_icon_hints(
                parent,
                asset_server,
                icon_registry,
                font.clone(),
                rename_active,
                delete_active,
            );

            parent.spawn((
                TextBundle::from_section(
                    status.unwrap_or_default(),
                    TextStyle {
                        font,
                        font_size: 16.0,
                        color: Color::srgb(0.95, 0.75, 0.4),
                    },
                ),
                CollectionMenuStatusText,
            ));
        });
}

fn collection_menu_visible_rows(window_height: f32) -> usize {
    // Reserve space for title, status, markers, and control hints.
    const RESERVED_HEIGHT: f32 = 420.0;
    const ENTRY_ROW_HEIGHT: f32 = 40.0;
    let rows = ((window_height - RESERVED_HEIGHT).max(0.0) / ENTRY_ROW_HEIGHT).floor() as usize;
    rows.max(COLLECTION_MENU_MIN_VISIBLE_ROWS)
}

fn collection_menu_visible_range(
    selected_index: usize,
    total_entries: usize,
    visible_rows: usize,
) -> (usize, usize) {
    if total_entries <= visible_rows {
        return (0, total_entries);
    }

    let half = visible_rows / 2;
    let mut start = selected_index.saturating_sub(half);
    let max_start = total_entries - visible_rows;
    if start > max_start {
        start = max_start;
    }
    let end = (start + visible_rows).min(total_entries);
    (start, end)
}

fn spawn_collection_menu_icon_hints(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    icon_registry: &KeyIconRegistry,
    font: Handle<Font>,
    rename_active: bool,
    delete_active: bool,
) {
    let show_default = !rename_active && !delete_active;
    let mut default_bundle = NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            row_gap: Val::Px(8.0),
            ..default()
        },
        ..default()
    };
    default_bundle.visibility = if show_default {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    parent
        .spawn((default_bundle, CollectionMenuDefaultHints))
        .with_children(|row| {
            spawn_collection_action_hint(
                row,
                asset_server,
                icon_registry,
                &font,
                UiKey::Enter,
                "Play",
            );
            spawn_collection_action_hint(
                row,
                asset_server,
                icon_registry,
                &font,
                UiKey::KeyI,
                "Import",
            );
            spawn_collection_action_hint(
                row,
                asset_server,
                icon_registry,
                &font,
                UiKey::KeyR,
                "Rename",
            );
            spawn_collection_action_hint(
                row,
                asset_server,
                icon_registry,
                &font,
                UiKey::Delete,
                "Delete",
            );
            spawn_collection_action_hint(
                row,
                asset_server,
                icon_registry,
                &font,
                UiKey::Escape,
                "Back",
            );
        });

    let mut rename_bundle = NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            row_gap: Val::Px(8.0),
            ..default()
        },
        ..default()
    };
    rename_bundle.visibility = if rename_active {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    parent
        .spawn((rename_bundle, CollectionMenuRenameHints))
        .with_children(|row| {
            spawn_collection_action_hint(
                row,
                asset_server,
                icon_registry,
                &font,
                UiKey::Enter,
                "Confirm Rename",
            );
            spawn_collection_action_hint(
                row,
                asset_server,
                icon_registry,
                &font,
                UiKey::Escape,
                "Cancel",
            );
        });

    let mut delete_bundle = NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            row_gap: Val::Px(8.0),
            ..default()
        },
        ..default()
    };
    delete_bundle.visibility = if delete_active {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    parent
        .spawn((delete_bundle, CollectionMenuDeleteHints))
        .with_children(|row| {
            spawn_collection_action_hint(
                row,
                asset_server,
                icon_registry,
                &font,
                UiKey::KeyY,
                "Confirm Delete",
            );
            spawn_collection_action_hint(
                row,
                asset_server,
                icon_registry,
                &font,
                UiKey::KeyN,
                "Cancel",
            );
            spawn_collection_action_hint(
                row,
                asset_server,
                icon_registry,
                &font,
                UiKey::Escape,
                "Cancel",
            );
        });
}

fn update_collection_menu_hint_visibility(
    rename_state: Res<RenameState>,
    delete_confirm: Res<DeleteConfirmState>,
    mut hints: Query<
        (
            &mut Visibility,
            Option<&CollectionMenuDefaultHints>,
            Option<&CollectionMenuRenameHints>,
            Option<&CollectionMenuDeleteHints>,
        ),
        Or<(
            With<CollectionMenuDefaultHints>,
            With<CollectionMenuRenameHints>,
            With<CollectionMenuDeleteHints>,
        )>,
    >,
) {
    if !(rename_state.is_changed() || delete_confirm.is_changed()) {
        return;
    }

    let default_visibility = if rename_state.active || delete_confirm.active {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    let rename_visibility = if rename_state.active {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let delete_visibility = if delete_confirm.active {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    for (mut visibility, default_marker, rename_marker, delete_marker) in &mut hints {
        if default_marker.is_some() {
            *visibility = default_visibility;
        } else if rename_marker.is_some() {
            *visibility = rename_visibility;
        } else if delete_marker.is_some() {
            *visibility = delete_visibility;
        }
    }
}

fn spawn_collection_action_hint(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    icon_registry: &KeyIconRegistry,
    font: &Handle<Font>,
    key: UiKey,
    label: &str,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(5.0),
                ..default()
            },
            ..default()
        })
        .with_children(|hint| {
            spawn_key_icon(hint, asset_server, icon_registry, key);
            hint.spawn(TextBundle::from_section(
                format!(": {label}"),
                TextStyle {
                    font: font.clone(),
                    font_size: 18.0,
                    color: Color::srgb(0.75, 0.8, 0.85),
                },
            ));
        });
}

fn update_hud_visibility(
    state: Res<State<AppState>>,
    mut hud_query: Query<&mut Visibility, With<HudText>>,
) {
    let visible = matches!(state.get(), AppState::Playing | AppState::LevelComplete);
    for mut visibility in &mut hud_query {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_win_overlay_visibility(
    levels: Option<Res<LoadedLevels>>,
    active_index: Option<Res<ActiveLevelIndex>>,
    current_state: Option<Res<CurrentGameState>>,
    state: Res<State<AppState>>,
    mut overlay_query: Query<&mut Visibility, With<WinOverlay>>,
) {
    let is_in_game = matches!(state.get(), AppState::Playing | AppState::LevelComplete);
    let is_won = match (levels, active_index, current_state) {
        (Some(levels), Some(active_index), Some(current_state)) => {
            current_state.0.is_won(&levels.levels[active_index.0])
        }
        _ => false,
    };
    let visibility = if is_in_game && is_won {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for mut overlay in &mut overlay_query {
        *overlay = visibility;
    }
}

fn spawn_win_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut bundle = TextBundle::from_sections([TextSection::new(
        "Level Complete!\nPress N for next level or R to restart",
        TextStyle {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 36.0,
            color: Color::srgb(1.0, 0.95, 0.5),
        },
    )])
    .with_text_justify(JustifyText::Center)
    .with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(24.0),
        left: Val::Px(0.0),
        width: Val::Percent(100.0),
        ..default()
    });
    bundle.visibility = Visibility::Hidden;

    commands.spawn((bundle, WinOverlay));
}

fn spawn_build_version_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut bundle = TextBundle::from_section(
        format!("v{BUILD_VERSION}"),
        TextStyle {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 18.0,
            color: Color::srgb(0.62, 0.68, 0.74),
        },
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(8.0),
        right: Val::Px(10.0),
        ..default()
    });
    bundle.z_index = ZIndex::Global(100);

    commands.spawn((bundle, BuildVersionText));
}

fn set_window_icon_once(
    mut applied: Local<bool>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    winit_windows: NonSend<WinitWindows>,
) {
    if *applied {
        return;
    }

    let Ok(primary_entity) = primary_window.get_single() else {
        return;
    };
    let Some(window) = winit_windows.get_window(primary_entity) else {
        return;
    };

    let Ok(image) = image::load_from_memory(include_bytes!("../assets/box.png")) else {
        *applied = true;
        return;
    };
    let image = image.into_rgba8();
    let (width, height) = image.dimensions();

    if let Ok(icon) = Icon::from_rgba(image.into_raw(), width, height) {
        window.set_window_icon(Some(icon));
    }

    *applied = true;
}

fn load_levels(
    mut commands: Commands,
    config: Res<StartupConfig>,
    selected_pack: Res<SelectedPackPath>,
    mut status: ResMut<CollectionMenuStatus>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let pack_path = if selected_pack.0.trim().is_empty() {
        &config.pack_path
    } else {
        &selected_pack.0
    };
    let pack_display = display_absolute_path_str(pack_path);

    let pack = match LevelPack::load(pack_path) {
        Ok(pack) => pack,
        Err(err) => {
            status.0 = Some(format!("Failed to load '{}': {err:#}", pack_display));
            next_state.set(AppState::CollectionMenu);
            return;
        }
    };

    let levels = match pack.parse_levels() {
        Ok(levels) => levels,
        Err(err) => {
            status.0 = Some(format!("Failed to parse '{}': {err:#}", pack_display));
            next_state.set(AppState::CollectionMenu);
            return;
        }
    };

    if levels.is_empty() {
        status.0 = Some(format!("Collection '{}' has no levels", pack_display));
        next_state.set(AppState::CollectionMenu);
        return;
    }

    let index = config.start_level.clamp(1, levels.len()) - 1;
    commands.insert_resource(ActiveLevelIndex(index));
    commands.insert_resource(LoadedLevels { levels });
    commands.insert_resource(UndoHistory::default());
    status.0 = None;
    next_state.set(AppState::Playing);
}

fn setup_level_state(
    mut commands: Commands,
    levels: Res<LoadedLevels>,
    active_index: Res<ActiveLevelIndex>,
    mut undo_history: ResMut<UndoHistory>,
    mut move_repeat: ResMut<MoveRepeatState>,
    mut undo_repeat: ResMut<UndoRepeatState>,
) {
    commands.insert_resource(CurrentGameState(GameState::from_level(
        &levels.levels[active_index.0],
    )));
    undo_history.0.clear();
    move_repeat.reset();
    undo_repeat.reset();
}

fn spawn_board(
    mut commands: Commands,
    levels: Res<LoadedLevels>,
    active_index: Res<ActiveLevelIndex>,
    current_state: Res<CurrentGameState>,
    sprites: Res<SpriteAssets>,
    existing: Query<Entity, With<BoardEntity>>,
) {
    despawn_board(&mut commands, &existing);
    spawn_tiles(&mut commands, &levels.levels[active_index.0], &sprites);
    spawn_dynamic(
        &mut commands,
        &levels.levels[active_index.0],
        &current_state.0,
        &sprites,
    );
}

fn handle_lifecycle_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    levels: Res<LoadedLevels>,
    mut active_index: ResMut<ActiveLevelIndex>,
    mut current_state: ResMut<CurrentGameState>,
    mut undo_history: ResMut<UndoHistory>,
    mut move_repeat: ResMut<MoveRepeatState>,
    mut undo_repeat: ResMut<UndoRepeatState>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        reset_active_level(
            &levels,
            active_index.0,
            &mut current_state,
            &mut undo_history,
            &mut move_repeat,
            &mut undo_repeat,
        );
        return;
    }

    if keys.just_pressed(KeyCode::KeyN) {
        active_index.0 = (active_index.0 + 1) % levels.levels.len();
        reset_active_level(
            &levels,
            active_index.0,
            &mut current_state,
            &mut undo_history,
            &mut move_repeat,
            &mut undo_repeat,
        );
        return;
    }

    if keys.just_pressed(KeyCode::KeyP) {
        active_index.0 = if active_index.0 == 0 {
            levels.levels.len() - 1
        } else {
            active_index.0 - 1
        };
        reset_active_level(
            &levels,
            active_index.0,
            &mut current_state,
            &mut undo_history,
            &mut move_repeat,
            &mut undo_repeat,
        );
        return;
    }

    if keys.just_pressed(KeyCode::KeyU) {
        undo_repeat.reset();
        if let Some(previous) = undo_history.0.pop_back() {
            current_state.0 = previous;
        }
        return;
    }

    if !keys.pressed(KeyCode::KeyU) {
        undo_repeat.reset();
        return;
    }

    undo_repeat.initial_delay.tick(time.delta());
    if !undo_repeat.initial_delay.finished() {
        return;
    }

    undo_repeat.repeat_interval.tick(time.delta());
    if undo_repeat.repeat_interval.just_finished()
        && let Some(previous) = undo_history.0.pop_back()
    {
        current_state.0 = previous;
    }
}

fn handle_back_to_collection_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyQ) {
        next_state.set(AppState::CollectionMenu);
    }
}

fn handle_move_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    levels: Res<LoadedLevels>,
    active_index: Res<ActiveLevelIndex>,
    mut current_state: ResMut<CurrentGameState>,
    mut undo_history: ResMut<UndoHistory>,
    mut next_state: ResMut<NextState<AppState>>,
    mut move_repeat: ResMut<MoveRepeatState>,
) {
    let pressed_dir = if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        Some(Direction::Up)
    } else if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        Some(Direction::Down)
    } else if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        Some(Direction::Left)
    } else if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        Some(Direction::Right)
    } else {
        None
    };

    let Some(dir) = pressed_dir else {
        move_repeat.reset();
        return;
    };

    let mut move_once = |direction: Direction| {
        let level = &levels.levels[active_index.0];
        let before = current_state.0.clone();
        let result = try_step(&mut current_state.0, level, direction);

        if !result.moved {
            return;
        }

        if undo_history.0.len() == UNDO_LIMIT {
            undo_history.0.pop_front();
        }
        undo_history.0.push_back(before);

        if result.won {
            next_state.set(AppState::LevelComplete);
        }
    };

    let dir_changed = move_repeat.held_direction != Some(dir);
    let just_pressed_dir =
        if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
            Some(Direction::Up)
        } else if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
            Some(Direction::Down)
        } else if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
            Some(Direction::Left)
        } else if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
            Some(Direction::Right)
        } else {
            None
        };

    if dir_changed || just_pressed_dir == Some(dir) {
        move_repeat.start_direction(dir);
        move_once(dir);
        return;
    }

    move_repeat.initial_delay.tick(time.delta());
    if !move_repeat.initial_delay.finished() {
        return;
    }

    move_repeat.repeat_interval.tick(time.delta());
    if move_repeat.repeat_interval.just_finished() {
        move_once(dir);
    }
}

fn sync_completion_state(
    levels: Res<LoadedLevels>,
    active_index: Res<ActiveLevelIndex>,
    current_state: Res<CurrentGameState>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !(levels.is_changed() || active_index.is_changed() || current_state.is_changed()) {
        return;
    }

    let level = &levels.levels[active_index.0];
    let is_won = current_state.0.is_won(level);

    if is_won && state.get() != &AppState::LevelComplete {
        next_state.set(AppState::LevelComplete);
    }
}

fn update_hud_text(
    levels: Res<LoadedLevels>,
    active_index: Res<ActiveLevelIndex>,
    current_state: Res<CurrentGameState>,
    mut hud_query: Query<
        (
            &mut Text,
            Option<&HudLevelText>,
            Option<&HudMovesText>,
            Option<&HudPushesText>,
        ),
        Or<(With<HudLevelText>, With<HudMovesText>, With<HudPushesText>)>,
    >,
) {
    if !(levels.is_changed() || active_index.is_changed() || current_state.is_changed()) {
        return;
    }

    let level = &levels.levels[active_index.0];
    let level_name = level.name.as_deref().unwrap_or("(unnamed)");
    let level_line = format!(
        "{}/{} - {}",
        active_index.0 + 1,
        levels.levels.len(),
        level_name
    );
    let moves_line = current_state.0.moves.to_string();
    let pushes_line = current_state.0.pushes.to_string();

    for (mut text, is_level, is_moves, is_pushes) in &mut hud_query {
        if is_level.is_some() {
            text.sections[0].value = level_line.clone();
        } else if is_moves.is_some() {
            text.sections[0].value = moves_line.clone();
        } else if is_pushes.is_some() {
            text.sections[0].value = pushes_line.clone();
        }
    }
}

fn sync_static_board(
    mut commands: Commands,
    levels: Res<LoadedLevels>,
    active_index: Res<ActiveLevelIndex>,
    current_state: Res<CurrentGameState>,
    sprites: Res<SpriteAssets>,
    existing: Query<Entity, With<BoardEntity>>,
) {
    if !active_index.is_changed() {
        return;
    }

    despawn_board(&mut commands, &existing);
    spawn_tiles(&mut commands, &levels.levels[active_index.0], &sprites);
    spawn_dynamic(
        &mut commands,
        &levels.levels[active_index.0],
        &current_state.0,
        &sprites,
    );
}

fn sync_dynamic_entities(
    mut commands: Commands,
    levels: Res<LoadedLevels>,
    active_index: Res<ActiveLevelIndex>,
    current_state: Res<CurrentGameState>,
    sprites: Res<SpriteAssets>,
    dynamic_entities: Query<Entity, Or<(With<BoxEntity>, With<PlayerEntity>)>>,
) {
    if !current_state.is_changed() {
        return;
    }

    for entity in &dynamic_entities {
        commands.entity(entity).despawn();
    }

    spawn_dynamic(
        &mut commands,
        &levels.levels[active_index.0],
        &current_state.0,
        &sprites,
    );
}

fn reset_active_level(
    levels: &LoadedLevels,
    active_index: usize,
    current_state: &mut ResMut<CurrentGameState>,
    undo_history: &mut ResMut<UndoHistory>,
    move_repeat: &mut ResMut<MoveRepeatState>,
    undo_repeat: &mut ResMut<UndoRepeatState>,
) {
    current_state.0 = GameState::from_level(&levels.levels[active_index]);
    undo_history.0.clear();
    move_repeat.reset();
    undo_repeat.reset();
}

fn despawn_board(commands: &mut Commands, existing: &Query<Entity, With<BoardEntity>>) {
    for entity in existing {
        commands.entity(entity).despawn();
    }
}

fn spawn_tiles(commands: &mut Commands, level: &Level, sprites: &SpriteAssets) {
    for y in 0..level.height {
        for x in 0..level.width {
            let pos = crate::coord::Pos::new(x, y);
            let texture = if level.is_wall(pos) {
                sprites.wall.clone()
            } else if level.is_goal(pos) {
                sprites.goal.clone()
            } else {
                sprites.floor.clone()
            };

            commands.spawn((
                SpriteBundle {
                    texture,
                    transform: Transform::from_translation(grid_to_world(
                        x,
                        y,
                        level.width,
                        level.height,
                        0.0,
                    )),
                    ..default()
                },
                BoardEntity,
                Tile,
            ));
        }
    }
}

fn spawn_dynamic(
    commands: &mut Commands,
    level: &Level,
    state: &GameState,
    sprites: &SpriteAssets,
) {
    for pos in &state.boxes {
        let texture = if level.is_goal(*pos) {
            sprites.box_on_goal.clone()
        } else {
            sprites.box_tile.clone()
        };

        commands.spawn((
            SpriteBundle {
                texture,
                transform: Transform::from_translation(grid_to_world(
                    pos.x,
                    pos.y,
                    level.width,
                    level.height,
                    1.0,
                )),
                ..default()
            },
            BoardEntity,
            BoxEntity,
        ));
    }

    let player_texture = if level.is_goal(state.player) {
        sprites.player_on_goal.clone()
    } else {
        sprites.player.clone()
    };

    commands.spawn((
        SpriteBundle {
            texture: player_texture,
            transform: Transform::from_translation(grid_to_world(
                state.player.x,
                state.player.y,
                level.width,
                level.height,
                2.0,
            )),
            ..default()
        },
        BoardEntity,
        PlayerEntity,
    ));
}

fn grid_to_world(x: i32, y: i32, width: i32, height: i32, z: f32) -> Vec3 {
    let world_x = (x as f32 - (width as f32 - 1.0) / 2.0) * TILE_SIZE;
    let world_y = ((height as f32 - 1.0) / 2.0 - y as f32) * TILE_SIZE;
    Vec3::new(world_x, world_y, z)
}

fn default_collection_path(path_config: &PathConfig) -> String {
    path_to_string(&path_config.builtin_default_pack)
}

fn default_collection_entry(path_config: &PathConfig) -> CollectionEntry {
    CollectionEntry {
        display_name: capped_collection_display_name(&path_config.builtin_default_pack),
        path: default_collection_path(path_config),
    }
}

fn discover_collections_from_disk(
    path_config: &PathConfig,
) -> Result<Vec<CollectionEntry>, String> {
    let mut entries = vec![default_collection_entry(path_config)];

    fs::create_dir_all(&path_config.imported_dir).map_err(|err| {
        format!(
            "Could not create '{}': {}",
            display_absolute_path(&path_config.imported_dir),
            err
        )
    })?;

    let read_dir = fs::read_dir(&path_config.imported_dir).map_err(|err| {
        format!(
            "Could not read '{}': {}",
            display_absolute_path(&path_config.imported_dir),
            err
        )
    })?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        if !is_txt_file(&path) {
            continue;
        }

        entries.push(CollectionEntry {
            display_name: capped_collection_display_name(&path),
            path: path_to_string(&path),
        });
    }

    entries.sort_by_key(|entry| entry.display_name.to_lowercase());
    Ok(entries)
}

fn import_collection(path_config: &PathConfig) -> Result<String, String> {
    let Some(source_path) = rfd::FileDialog::new()
        .add_filter("Text collections", &["txt"])
        .pick_file()
    else {
        return Err("Import canceled".to_string());
    };

    let source_path_display = display_absolute_path(&source_path);
    let pack = LevelPack::load(&source_path)
        .map_err(|err| format!("Import failed (load): '{}': {err:#}", source_path_display))?;
    let levels = pack
        .parse_levels()
        .map_err(|err| format!("Import failed (parse): '{}': {err:#}", source_path_display))?;
    if levels.is_empty() {
        return Err(format!(
            "Import failed: '{}' contains no levels",
            source_path_display
        ));
    }

    let destination = next_available_import_path(path_config, &source_path)?;
    fs::copy(&source_path, &destination).map_err(|err| {
        format!(
            "Import failed (copy to '{}'): {}",
            display_absolute_path(&destination),
            err
        )
    })?;

    Ok(path_to_string(&destination))
}

fn handle_rename_mode_input(
    keys: &ButtonInput<KeyCode>,
    path_config: &PathConfig,
    rename_state: &mut RenameState,
    collections: &mut CollectionsResource,
    selection: &mut CollectionMenuSelection,
    selected_pack: &mut SelectedPackPath,
    status: &mut CollectionMenuStatus,
) {
    if keys.just_pressed(KeyCode::Escape) {
        let original = rename_state
            .source_display_name
            .as_deref()
            .unwrap_or("collection")
            .to_string();
        rename_state.active = false;
        rename_state.source_path = None;
        rename_state.source_display_name = None;
        rename_state.buffer.clear();
        status.0 = Some(format!("Rename canceled for '{}'", original));
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        rename_state.buffer.pop();
        status.0 = Some("Renaming".to_string());
        return;
    }

    if keys.just_pressed(KeyCode::Enter) {
        let Some(source_path) = rename_state.source_path.clone() else {
            rename_state.active = false;
            status.0 = Some("Rename failed: missing selected collection".to_string());
            return;
        };

        match rename_collection(path_config, &source_path, &rename_state.buffer) {
            Ok(renamed_path) => match discover_collections_from_disk(path_config) {
                Ok(entries) => {
                    collections.entries = entries;
                    selected_pack.0 = renamed_path.clone();
                    if let Some(index) = collections
                        .entries
                        .iter()
                        .position(|entry| entry.path == renamed_path)
                    {
                        selection.0 = index;
                    }
                    status.0 = Some("Collection renamed".to_string());
                }
                Err(err) => {
                    status.0 = Some(err);
                }
            },
            Err(err) => {
                status.0 = Some(err);
                return;
            }
        }

        rename_state.active = false;
        rename_state.source_path = None;
        rename_state.source_display_name = None;
        rename_state.buffer.clear();
        return;
    }

    let typed = typed_characters(keys);
    if !typed.is_empty() {
        rename_state.buffer.push_str(&typed);
        rename_state.buffer = truncate_chars(&rename_state.buffer, COLLECTION_NAME_MAX_CHARS);
        status.0 = Some("Renaming".to_string());
    }
}

fn handle_delete_confirm_input(
    keys: &ButtonInput<KeyCode>,
    path_config: &PathConfig,
    delete_confirm: &mut DeleteConfirmState,
    collections: &mut CollectionsResource,
    selection: &mut CollectionMenuSelection,
    selected_pack: &mut SelectedPackPath,
    status: &mut CollectionMenuStatus,
) {
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyN) {
        delete_confirm.active = false;
        delete_confirm.source_path = None;
        delete_confirm.source_display_name = None;
        status.0 = None;
        return;
    }

    if !keys.just_pressed(KeyCode::KeyY) {
        return;
    }

    let Some(source_path) = delete_confirm.source_path.clone() else {
        delete_confirm.active = false;
        status.0 = Some("Delete failed: missing selected collection".to_string());
        return;
    };

    match delete_collection(path_config, &source_path) {
        Ok(()) => match discover_collections_from_disk(path_config) {
            Ok(entries) => {
                collections.entries = entries;
                if collections.entries.is_empty() {
                    selection.0 = 0;
                    selected_pack.0 = default_collection_path(path_config);
                } else {
                    selection.0 = selection.0.min(collections.entries.len() - 1);
                    if source_path == selected_pack.0 {
                        selected_pack.0 = collections.entries[selection.0].path.clone();
                    }
                }
                status.0 = None;
            }
            Err(err) => {
                status.0 = Some(err);
            }
        },
        Err(err) => {
            status.0 = Some(err);
        }
    }

    delete_confirm.active = false;
    delete_confirm.source_path = None;
    delete_confirm.source_display_name = None;
}

fn rename_collection(
    path_config: &PathConfig,
    source_path: &str,
    raw_name: &str,
) -> Result<String, String> {
    if source_path == default_collection_path(path_config) {
        return Err("Built-in default collection cannot be renamed".to_string());
    }

    let source =
        canonicalize_imported_collection_source(path_config, source_path, "Rename failed")?;
    let imported_dir = canonicalize_imported_dir(path_config, "Rename failed")?;

    let target_file_name = normalize_collection_name(raw_name)?;
    let target_path = imported_dir.join(&target_file_name);

    if source != target_path && target_path.exists() {
        return Err("Name already exists".to_string());
    }

    fs::rename(&source, &target_path).map_err(|err| {
        format!(
            "Rename failed: '{}' -> '{}': {}",
            source.to_string_lossy(),
            target_file_name,
            err
        )
    })?;

    Ok(path_to_string(&target_path))
}

fn delete_collection(path_config: &PathConfig, source_path: &str) -> Result<(), String> {
    if source_path == default_collection_path(path_config) {
        return Err("Built-in default collection cannot be deleted".to_string());
    }

    let source =
        canonicalize_imported_collection_source(path_config, source_path, "Delete failed")?;

    fs::remove_file(&source).map_err(|err| {
        format!(
            "Delete failed: could not remove '{}': {}",
            display_absolute_path(&source),
            err
        )
    })?;

    Ok(())
}

fn normalize_collection_name(raw_name: &str) -> Result<String, String> {
    let trimmed = raw_name.trim();
    if trimmed.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if trimmed.chars().any(is_invalid_filename_char) {
        return Err("Name contains invalid filename characters".to_string());
    }

    let without_extension = trimmed
        .strip_suffix(".txt")
        .or_else(|| trimmed.strip_suffix(".TXT"))
        .unwrap_or(trimmed);
    if without_extension.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if without_extension.chars().count() > COLLECTION_NAME_MAX_CHARS {
        return Err(format!(
            "Name cannot exceed {} characters",
            COLLECTION_NAME_MAX_CHARS
        ));
    }

    let file_name = format!("{without_extension}.txt");

    if file_name.eq_ignore_ascii_case("default.txt") {
        return Err("Name 'default' is reserved".to_string());
    }

    Ok(file_name)
}

fn is_invalid_filename_char(ch: char) -> bool {
    matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
}

fn typed_characters(keys: &ButtonInput<KeyCode>) -> String {
    let mut out = String::new();
    for key in keys.get_just_pressed() {
        if let Some(ch) = keycode_to_char(*key) {
            out.push(ch);
        }
    }
    out
}

fn keycode_to_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        KeyCode::Space => Some(' '),
        KeyCode::Period => Some('.'),
        KeyCode::Minus => Some('-'),
        KeyCode::Comma => Some(','),
        KeyCode::Semicolon => Some(';'),
        _ => None,
    }
}

fn next_available_import_path(
    path_config: &PathConfig,
    source_path: &Path,
) -> Result<PathBuf, String> {
    fs::create_dir_all(&path_config.imported_dir).map_err(|err| {
        format!(
            "Could not create '{}': {}",
            display_absolute_path(&path_config.imported_dir),
            err
        )
    })?;

    let stem = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.trim().is_empty())
        .unwrap_or("collection")
        .trim();
    let sanitized_stem: String = stem
        .chars()
        .map(|ch| {
            if is_invalid_filename_char(ch) {
                '_'
            } else {
                ch
            }
        })
        .collect();
    let truncated_stem = truncate_chars(
        if sanitized_stem.trim().is_empty() {
            "collection"
        } else {
            sanitized_stem.trim()
        },
        COLLECTION_NAME_MAX_CHARS,
    );

    let mut candidate = path_config
        .imported_dir
        .join(format!("{truncated_stem}.txt"));
    if !candidate.exists() {
        return Ok(candidate);
    }

    for n in 1..10_000 {
        let suffix = format!(" ({n})");
        let max_base_chars = COLLECTION_NAME_MAX_CHARS.saturating_sub(suffix.chars().count());
        let base_for_suffix = truncate_chars(&truncated_stem, max_base_chars);
        candidate = path_config
            .imported_dir
            .join(format!("{base_for_suffix}{suffix}.txt"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("Import failed: could not find an available destination filename".to_string())
}

fn is_txt_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
}

fn file_display_name(path: &Path) -> String {
    let raw = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    raw.strip_suffix(".txt")
        .or_else(|| raw.strip_suffix(".TXT"))
        .unwrap_or(&raw)
        .to_string()
}

fn capped_collection_display_name(path: &Path) -> String {
    truncate_chars(&file_display_name(path), COLLECTION_NAME_MAX_CHARS)
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    input.chars().take(max_chars).collect()
}

fn canonicalize_imported_dir(path_config: &PathConfig, op: &str) -> Result<PathBuf, String> {
    fs::create_dir_all(&path_config.imported_dir).map_err(|err| {
        format!(
            "{op}: could not create '{}': {}",
            display_absolute_path(&path_config.imported_dir),
            err
        )
    })?;
    fs::canonicalize(&path_config.imported_dir).map_err(|err| {
        format!(
            "{op}: could not resolve '{}': {}",
            display_absolute_path(&path_config.imported_dir),
            err
        )
    })
}

fn canonicalize_imported_collection_source(
    path_config: &PathConfig,
    source_path: &str,
    op: &str,
) -> Result<PathBuf, String> {
    let source = Path::new(source_path);
    if !source.exists() {
        return Err(format!(
            "{op}: source '{}' does not exist",
            display_absolute_path(source)
        ));
    }

    let source_canonical = fs::canonicalize(source).map_err(|err| {
        format!(
            "{op}: could not resolve source '{}': {}",
            display_absolute_path(source),
            err
        )
    })?;
    let imported_dir = canonicalize_imported_dir(path_config, op)?;

    if source_canonical.parent() != Some(imported_dir.as_path()) || !is_txt_file(&source_canonical)
    {
        return Err(format!(
            "{op}: only imported collections in '{}' can be modified",
            display_absolute_path(&imported_dir)
        ));
    }

    Ok(source_canonical)
}

fn display_absolute_path_str(path: &str) -> String {
    display_absolute_path(Path::new(path))
}

fn display_absolute_path(path: &Path) -> String {
    if path.is_absolute() {
        return path_to_string(path);
    }

    match std::env::current_dir() {
        Ok(cwd) => path_to_string(&cwd.join(path)),
        Err(_) => path_to_string(path),
    }
}
