use std::{
    collections::{HashMap, VecDeque},
    env, fs,
    path::{Path, PathBuf},
};

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use winit::window::Icon;

use crate::{
    coord::Direction,
    editor::{EditorMap, EditorTile, EditorUndoSnapshot, SimpleWarning},
    editor_model::{EditableCollection, EditableMap, MAP_NAME_MAX_CHARS},
    level::{Level, LevelPack, RawLevel},
    paths::PathConfig,
    rules::try_step,
    state::GameState,
};

const UNDO_LIMIT: usize = 10_000;
const TILE_SIZE: f32 = 36.0;
const COLLECTION_MENU_MIN_VISIBLE_ROWS: usize = 3;
const COLLECTION_NAME_MAX_CHARS: usize = 64;
const MAP_LIST_COLUMNS_DEFAULT: usize = 5;
const MAP_LIST_CARD_PREVIEW_MAX_LINES: usize = 15;
const MAP_LIST_CARD_PREVIEW_MAX_WIDTH: usize = 15;
const MAP_LIST_MIN_VISIBLE_ROWS: usize = 1;
const MAP_LIST_PREVIEW_TILE_SIZE: f32 = 12.0;
const MAP_EDITOR_TILE_SIZE: f32 = 28.0;
const MAP_EDITOR_PHANTOM_ALPHA: f32 = 0.45;
const MAP_LIST_CARD_PREVIEW_HEIGHT_PX: f32 =
    MAP_LIST_CARD_PREVIEW_MAX_LINES as f32 * MAP_LIST_PREVIEW_TILE_SIZE;
const MAP_LIST_CARD_NAME_ROW_PX: f32 = 20.0;
const MAP_LIST_CARD_INTERNAL_GAP_PX: f32 = 8.0;
const MAP_LIST_GRID_WIDTH_PERCENT: f32 = 94.0;
const MAP_LIST_GRID_PADDING_LEFT_PX: f32 = 180.0;
const MAP_LIST_GRID_PADDING_RIGHT_PX: f32 = 24.0;
const MAP_LIST_GRID_COLUMN_GAP_PX: f32 = 10.0;
const MAP_LIST_CARD_MIN_WIDTH_PX: f32 = 200.0;
const MAP_LIST_CARD_MIN_HEIGHT_PX: f32 =
    MAP_LIST_CARD_PREVIEW_HEIGHT_PX + MAP_LIST_CARD_NAME_ROW_PX + MAP_LIST_CARD_INTERNAL_GAP_PX;
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
    Ctrl,
    Enter,
    Space,
    Escape,
    Delete,
    Backspace,
    KeyA,
    KeyD,
    KeyE,
    KeyF2,
    KeyI,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyW,
    KeyY,
    KeyZ,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
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
            UiKey::Ctrl,
            IconVariant {
                default_path: "icons/keyboard_ctrl.png",
                outline_path: "icons/keyboard_ctrl_outline.png",
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
            UiKey::KeyE,
            IconVariant {
                default_path: "icons/keyboard_e.png",
                outline_path: "icons/keyboard_e_outline.png",
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
            UiKey::KeyM,
            IconVariant {
                default_path: "icons/keyboard_m.png",
                outline_path: "icons/keyboard_m_outline.png",
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
            UiKey::KeyO,
            IconVariant {
                default_path: "icons/keyboard_o.png",
                outline_path: "icons/keyboard_o_outline.png",
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
            UiKey::KeyT,
            IconVariant {
                default_path: "icons/keyboard_t.png",
                outline_path: "icons/keyboard_t_outline.png",
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
        variants.insert(
            UiKey::KeyZ,
            IconVariant {
                default_path: "icons/keyboard_z.png",
                outline_path: "icons/keyboard_z_outline.png",
            },
        );
        variants.insert(
            UiKey::Digit1,
            IconVariant {
                default_path: "icons/keyboard_1.png",
                outline_path: "icons/keyboard_1_outline.png",
            },
        );
        variants.insert(
            UiKey::Digit2,
            IconVariant {
                default_path: "icons/keyboard_2.png",
                outline_path: "icons/keyboard_2_outline.png",
            },
        );
        variants.insert(
            UiKey::Digit3,
            IconVariant {
                default_path: "icons/keyboard_3.png",
                outline_path: "icons/keyboard_3_outline.png",
            },
        );
        variants.insert(
            UiKey::Digit4,
            IconVariant {
                default_path: "icons/keyboard_4.png",
                outline_path: "icons/keyboard_4_outline.png",
            },
        );
        variants.insert(
            UiKey::Digit5,
            IconVariant {
                default_path: "icons/keyboard_5.png",
                outline_path: "icons/keyboard_5_outline.png",
            },
        );
        variants.insert(
            UiKey::Digit6,
            IconVariant {
                default_path: "icons/keyboard_6.png",
                outline_path: "icons/keyboard_6_outline.png",
            },
        );
        variants.insert(
            UiKey::Digit7,
            IconVariant {
                default_path: "icons/keyboard_7.png",
                outline_path: "icons/keyboard_7_outline.png",
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
    CreateCollectionPrompt,
    CollectionMapList,
    MapEditor,
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

#[derive(Debug, Clone)]
struct CollectionMapEntry {
    name: String,
    raw_lines: Vec<String>,
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

#[derive(Resource, Debug, Clone, Default)]
struct CreateCollectionPromptState {
    buffer: String,
    status: Option<String>,
}

#[derive(Resource, Debug, Clone, Default)]
struct CollectionMapListState {
    collection_name: String,
    collection_path: String,
    maps: Vec<CollectionMapEntry>,
    selected_index: usize,
    status: Option<String>,
    revision: u64,
    move_mode_active: bool,
    move_mode_original_maps: Option<Vec<CollectionMapEntry>>,
    move_mode_original_selected_index: usize,
}

#[derive(Resource, Debug, Clone, Default)]
struct CollectionMapListLayoutState {
    start_row: usize,
    end_row: usize,
    columns: usize,
    rendered_revision: u64,
}

#[derive(Resource, Debug, Clone)]
struct EditorSession {
    collection_path: String,
    map_index: usize,
    selected_brush: EditorTile,
    dirty: bool,
    undo_stack: Vec<EditorUndoSnapshot>,
    working_map: Option<EditorMap>,
    status: Option<String>,
    last_painted_cell: Option<(usize, usize)>,
    baseline_map: Option<EditorUndoSnapshot>,
}

impl Default for EditorSession {
    fn default() -> Self {
        Self {
            collection_path: String::new(),
            map_index: 0,
            selected_brush: EditorTile::Floor,
            dirty: false,
            undo_stack: Vec::new(),
            working_map: None,
            status: None,
            last_painted_cell: None,
            baseline_map: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MapNamePromptMode {
    New,
    Rename,
}

#[derive(Resource, Debug, Clone, Default)]
struct MapNamePromptState {
    active: bool,
    mode: Option<MapNamePromptMode>,
    buffer: String,
    target_index: Option<usize>,
}

#[derive(Resource, Debug, Clone, Default)]
struct MapDeleteConfirmState {
    active: bool,
    target_index: Option<usize>,
}

#[derive(Resource, Debug, Clone, Default)]
struct MapResizePromptState {
    active: bool,
    buffer: String,
}

#[derive(Resource, Debug, Clone, Default)]
struct MapRenamePromptState {
    active: bool,
    buffer: String,
}

#[derive(Resource, Debug, Clone, Default)]
struct MapExitConfirmState {
    active: bool,
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

#[derive(Component)]
struct CreateCollectionPromptRoot;

#[derive(Component)]
struct CreateCollectionPromptInputText;

#[derive(Component)]
struct CreateCollectionPromptStatusText;

#[derive(Component)]
struct CollectionMapListRoot;

#[derive(Component)]
struct CollectionMapListStatusText;

#[derive(Component)]
struct MapEditorRoot;

#[derive(Component)]
struct MapEditorInfoText;

#[derive(Component)]
struct MapEditorStatusText;

#[derive(Component)]
struct MapEditorCanvasNode;

#[derive(Component)]
struct MapEditorBrushRow {
    brush: EditorTile,
}

#[derive(Component)]
struct MapEditorCanvasTile {
    x: usize,
    y: usize,
}

#[derive(Component)]
struct MapEditorWarningText;

#[derive(Resource, Debug, Clone, Default)]
struct MapEditorLayoutState {
    rendered_size: Option<(usize, usize)>,
}

#[derive(Resource, Debug, Clone, Copy, Eq, PartialEq, Default)]
enum PlaySessionMode {
    #[default]
    NormalCollection,
    EditorPlaytest,
}

#[derive(Resource, Debug, Clone, Default)]
struct MapEditorResumeState {
    resume_existing: bool,
}

#[derive(Component)]
struct CollectionMapCard {
    index: usize,
}

#[derive(Component)]
struct CollectionMapCardNameText {
    index: usize,
}

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
            .init_resource::<CreateCollectionPromptState>()
            .init_resource::<CollectionMapListState>()
            .init_resource::<CollectionMapListLayoutState>()
            .init_resource::<MapNamePromptState>()
            .init_resource::<MapDeleteConfirmState>()
            .init_resource::<MapResizePromptState>()
            .init_resource::<MapRenamePromptState>()
            .init_resource::<MapExitConfirmState>()
            .init_resource::<EditorSession>()
            .init_resource::<MapEditorLayoutState>()
            .init_resource::<PlaySessionMode>()
            .init_resource::<MapEditorResumeState>()
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
            .add_systems(
                OnEnter(AppState::CreateCollectionPrompt),
                spawn_create_collection_prompt,
            )
            .add_systems(
                OnExit(AppState::CreateCollectionPrompt),
                despawn_create_collection_prompt,
            )
            .add_systems(
                OnEnter(AppState::CollectionMapList),
                (setup_collection_map_list, spawn_collection_map_list).chain(),
            )
            .add_systems(
                OnExit(AppState::CollectionMapList),
                despawn_collection_map_list,
            )
            .add_systems(
                OnEnter(AppState::MapEditor),
                (setup_map_editor_session, spawn_map_editor).chain(),
            )
            .add_systems(OnExit(AppState::MapEditor), despawn_map_editor)
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
                    handle_create_collection_prompt_input,
                    update_create_collection_prompt_text,
                )
                    .chain()
                    .run_if(in_state(AppState::CreateCollectionPrompt)),
            )
            .add_systems(
                Update,
                (
                    handle_collection_map_list_input,
                    update_collection_map_list_status_text,
                    update_collection_map_list_selection_visuals,
                    refresh_collection_map_list,
                )
                    .chain()
                    .run_if(in_state(AppState::CollectionMapList)),
            )
            .add_systems(
                Update,
                (
                    handle_map_editor_input,
                    refresh_map_editor,
                    update_map_editor_hover_preview,
                    update_map_editor_visuals,
                    update_map_editor_text,
                    update_map_editor_warning_text,
                )
                    .chain()
                    .run_if(in_state(AppState::MapEditor)),
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
            UiKey::KeyZ,
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

    if keys.just_pressed(KeyCode::KeyN) {
        next_state.set(AppState::CreateCollectionPrompt);
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

    selection.0 = selection.0.min(item_count - 1);
    let move_up = keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW);
    let move_down = keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS);

    if move_up {
        selection.0 = if selection.0 == 0 {
            item_count - 1
        } else {
            selection.0 - 1
        };
    }

    if move_down {
        selection.0 = (selection.0 + 1) % item_count;
    }

    // If navigation happened this frame, skip actions to avoid acting on a transient selection.
    if move_up || move_down {
        return;
    }

    let current_index = selection.0;

    if keys.just_pressed(KeyCode::KeyE)
        && let Some(item) = collections.entries.get(current_index)
    {
        selected_pack.0 = item.path.clone();
        status.0 = None;
        next_state.set(AppState::CollectionMapList);
        return;
    }

    if (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space))
        && let Some(item) = collections.entries.get(current_index)
    {
        if let Ok(collection) = EditableCollection::load(&item.path)
            && collection.maps.is_empty()
        {
            status.0 = Some("Collection is empty. Press E to add maps before playing.".to_string());
            return;
        }
        selected_pack.0 = item.path.clone();
        status.0 = None;
        next_state.set(AppState::Loading);
        return;
    }

    if (keys.just_pressed(KeyCode::KeyR) || keys.just_pressed(KeyCode::F2))
        && let Some(item) = collections.entries.get(current_index)
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
        return;
    }

    if keys.just_pressed(KeyCode::Delete)
        && let Some(item) = collections.entries.get(current_index)
    {
        if item.path == default_collection_path(path_config.as_ref()) {
            status.0 = Some("Built-in default collection cannot be deleted".to_string());
            return;
        }

        delete_confirm.active = true;
        delete_confirm.source_path = Some(item.path.clone());
        delete_confirm.source_display_name = Some(item.display_name.clone());
        status.0 = Some(format!("Delete '{}' ? (Y/N)", item.display_name));
        return;
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
                UiKey::KeyN,
                "New",
            );
            spawn_collection_action_hint(
                row,
                asset_server,
                icon_registry,
                &font,
                UiKey::KeyE,
                "Edit",
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

fn spawn_create_collection_prompt(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icon_registry: Res<KeyIconRegistry>,
    mut prompt_state: ResMut<CreateCollectionPromptState>,
) {
    prompt_state.buffer.clear();
    prompt_state.status = None;
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(14.0),
                    ..default()
                },
                background_color: BackgroundColor(GAME_BACKGROUND_COLOR),
                ..default()
            },
            CreateCollectionPromptRoot,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Create Collection",
                TextStyle {
                    font: font.clone(),
                    font_size: 44.0,
                    color: Color::WHITE,
                },
            ));

            parent.spawn((
                TextBundle::from_section(
                    "_",
                    TextStyle {
                        font: font.clone(),
                        font_size: 30.0,
                        color: Color::srgb(1.0, 0.9, 0.35),
                    },
                ),
                CreateCollectionPromptInputText,
            ));

            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font.clone(),
                        font_size: 16.0,
                        color: Color::srgb(0.95, 0.75, 0.4),
                    },
                ),
                CreateCollectionPromptStatusText,
            ));

            parent
                .spawn(NodeBundle {
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
                })
                .with_children(|row| {
                    spawn_collection_action_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        &font,
                        UiKey::Enter,
                        "Create",
                    );
                    spawn_collection_action_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        &font,
                        UiKey::Backspace,
                        "Delete Char",
                    );
                    spawn_collection_action_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        &font,
                        UiKey::Escape,
                        "Back",
                    );
                });
        });
}

fn despawn_create_collection_prompt(
    mut commands: Commands,
    roots: Query<Entity, With<CreateCollectionPromptRoot>>,
) {
    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
}

fn handle_create_collection_prompt_input(
    keys: Res<ButtonInput<KeyCode>>,
    path_config: Res<PathConfig>,
    mut prompt_state: ResMut<CreateCollectionPromptState>,
    mut selected_pack: ResMut<SelectedPackPath>,
    mut map_list_state: ResMut<CollectionMapListState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::CollectionMenu);
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        prompt_state.buffer.pop();
        return;
    }

    if keys.just_pressed(KeyCode::Enter) {
        match EditableCollection::create_in_imported_dir(
            &path_config.imported_dir,
            &prompt_state.buffer,
        ) {
            Ok(collection) => {
                selected_pack.0 = path_to_string(&collection.source);
                map_list_state.collection_name = collection.collection_name;
                map_list_state.collection_path = selected_pack.0.clone();
                map_list_state.maps = collection
                    .maps
                    .iter()
                    .map(|map| CollectionMapEntry {
                        name: map.name.clone(),
                        raw_lines: map.raw_lines.clone(),
                    })
                    .collect();
                map_list_state.selected_index = 0;
                map_list_state.status = None;
                map_list_state.move_mode_active = false;
                map_list_state.move_mode_original_maps = None;
                map_list_state.move_mode_original_selected_index = 0;
                bump_map_list_revision(&mut map_list_state);
                next_state.set(AppState::CollectionMapList);
            }
            Err(err) => {
                prompt_state.status = Some(err.to_string());
            }
        }
        return;
    }

    let typed = typed_characters(&keys);
    if !typed.is_empty() {
        prompt_state.buffer.push_str(&typed);
        prompt_state.buffer = truncate_chars(&prompt_state.buffer, COLLECTION_NAME_MAX_CHARS);
    }
}

fn update_create_collection_prompt_text(
    prompt_state: Res<CreateCollectionPromptState>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<CreateCollectionPromptInputText>>,
        Query<&mut Text, With<CreateCollectionPromptStatusText>>,
    )>,
) {
    if !prompt_state.is_changed() {
        return;
    }

    let current = format!("{}_", prompt_state.buffer);
    for mut text in &mut text_sets.p0() {
        text.sections[0].value = current.clone();
    }

    let status = prompt_state.status.clone().unwrap_or_default();
    for mut text in &mut text_sets.p1() {
        text.sections[0].value = status.clone();
    }
}

fn setup_collection_map_list(
    selected_pack: Res<SelectedPackPath>,
    mut map_list_state: ResMut<CollectionMapListState>,
    mut map_name_prompt: ResMut<MapNamePromptState>,
    mut map_delete_confirm: ResMut<MapDeleteConfirmState>,
) {
    map_name_prompt.active = false;
    map_name_prompt.mode = None;
    map_name_prompt.buffer.clear();
    map_name_prompt.target_index = None;
    map_delete_confirm.active = false;
    map_delete_confirm.target_index = None;

    if selected_pack.0.is_empty() {
        map_list_state.collection_name = "Unknown".to_string();
        map_list_state.collection_path.clear();
        map_list_state.maps.clear();
        map_list_state.selected_index = 0;
        map_list_state.status = Some("No selected collection".to_string());
        map_list_state.move_mode_active = false;
        map_list_state.move_mode_original_maps = None;
        map_list_state.move_mode_original_selected_index = 0;
        bump_map_list_revision(&mut map_list_state);
        return;
    }

    match EditableCollection::load(&selected_pack.0) {
        Ok(collection) => {
            map_list_state.collection_name = collection.collection_name;
            map_list_state.collection_path = path_to_string(&collection.source);
            map_list_state.maps = collection
                .maps
                .iter()
                .map(|map| CollectionMapEntry {
                    name: map.name.clone(),
                    raw_lines: map.raw_lines.clone(),
                })
                .collect();
            map_list_state.selected_index = map_list_state
                .selected_index
                .min(map_list_state.maps.len().saturating_sub(1));
            map_list_state.status = None;
            map_list_state.move_mode_active = false;
            map_list_state.move_mode_original_maps = None;
            map_list_state.move_mode_original_selected_index = map_list_state.selected_index;
            bump_map_list_revision(&mut map_list_state);
        }
        Err(err) => {
            map_list_state.collection_name = file_display_name(Path::new(&selected_pack.0));
            map_list_state.collection_path = selected_pack.0.clone();
            map_list_state.maps.clear();
            map_list_state.selected_index = 0;
            map_list_state.status = Some(format!("Failed to load collection: {err:#}"));
            map_list_state.move_mode_active = false;
            map_list_state.move_mode_original_maps = None;
            map_list_state.move_mode_original_selected_index = 0;
            bump_map_list_revision(&mut map_list_state);
        }
    }
}

fn spawn_collection_map_list(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icon_registry: Res<KeyIconRegistry>,
    sprite_assets: Res<SpriteAssets>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    map_list_state: Res<CollectionMapListState>,
    mut layout_state: ResMut<CollectionMapListLayoutState>,
) {
    let (window_height, window_width) = primary_window
        .get_single()
        .map(|window| (window.height(), window.width()))
        .unwrap_or((768.0, 1024.0));
    spawn_collection_map_list_entities(
        &mut commands,
        &asset_server,
        &icon_registry,
        &sprite_assets,
        &map_list_state,
        &mut layout_state,
        window_height,
        window_width,
    );
}

fn despawn_collection_map_list(
    mut commands: Commands,
    roots: Query<Entity, With<CollectionMapListRoot>>,
) {
    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
}

fn handle_collection_map_list_input(
    keys: Res<ButtonInput<KeyCode>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut map_list_state: ResMut<CollectionMapListState>,
    mut map_name_prompt: ResMut<MapNamePromptState>,
    mut map_delete_confirm: ResMut<MapDeleteConfirmState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if map_name_prompt.active {
        handle_map_name_prompt_input(&keys, &mut map_list_state, &mut map_name_prompt);
        return;
    }

    if map_delete_confirm.active {
        handle_map_delete_confirm_input(&keys, &mut map_list_state, &mut map_delete_confirm);
        return;
    }

    let columns = primary_window
        .get_single()
        .map(|window| map_list_columns(window.width()))
        .unwrap_or(MAP_LIST_COLUMNS_DEFAULT);
    let map_count = map_list_state.maps.len();

    if map_list_state.move_mode_active {
        handle_map_move_mode_input(&keys, &mut map_list_state, columns);
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::CollectionMenu);
        return;
    }

    if keys.just_pressed(KeyCode::KeyE) {
        if map_count == 0 {
            map_list_state.status = Some("Collection is empty. Create a map first.".to_string());
            return;
        }
        next_state.set(AppState::MapEditor);
        return;
    }

    if keys.just_pressed(KeyCode::KeyN) {
        map_name_prompt.active = true;
        map_name_prompt.mode = Some(MapNamePromptMode::New);
        map_name_prompt.buffer.clear();
        map_name_prompt.target_index = None;
        map_list_state.status = Some("New map name: _".to_string());
        return;
    }

    if keys.just_pressed(KeyCode::KeyO) {
        map_list_state.status = match export_collection_with_save_dialog(&map_list_state) {
            Ok(message) => Some(message),
            Err(err) => Some(err),
        };
        return;
    }

    if map_count == 0 {
        if keys.just_pressed(KeyCode::KeyM) {
            map_list_state.status = Some("Collection is empty. Create a map first.".to_string());
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyM) {
        begin_map_move_mode(&mut map_list_state);
        return;
    }

    if keys.just_pressed(KeyCode::KeyR) || keys.just_pressed(KeyCode::F2) {
        map_name_prompt.active = true;
        map_name_prompt.mode = Some(MapNamePromptMode::Rename);
        map_name_prompt.buffer = map_list_state.maps[map_list_state.selected_index]
            .name
            .clone();
        map_name_prompt.target_index = Some(map_list_state.selected_index);
        map_list_state.status = Some(format!("Rename map: {}_", map_name_prompt.buffer));
        return;
    }

    if keys.just_pressed(KeyCode::Delete) {
        map_delete_confirm.active = true;
        map_delete_confirm.target_index = Some(map_list_state.selected_index);
        let selected_name = map_list_state.maps[map_list_state.selected_index]
            .name
            .clone();
        map_list_state.status = Some(format!("Delete '{}' ? (Y/N)", selected_name));
        return;
    }

    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
        if let Some(target) = map_list_move_target(
            map_list_state.selected_index,
            columns,
            map_count,
            Direction::Left,
        ) {
            map_list_state.selected_index = target;
        }
        return;
    }

    if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
        if let Some(target) = map_list_move_target(
            map_list_state.selected_index,
            columns,
            map_count,
            Direction::Right,
        ) {
            map_list_state.selected_index = target;
        }
        return;
    }

    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        if let Some(target) = map_list_move_target(
            map_list_state.selected_index,
            columns,
            map_count,
            Direction::Up,
        ) {
            map_list_state.selected_index = target;
        }
        return;
    }

    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        if let Some(target) = map_list_move_target(
            map_list_state.selected_index,
            columns,
            map_count,
            Direction::Down,
        ) {
            map_list_state.selected_index = target;
        }
    }
}

fn handle_map_move_mode_input(
    keys: &ButtonInput<KeyCode>,
    map_list_state: &mut CollectionMapListState,
    columns: usize,
) {
    let map_count = map_list_state.maps.len();
    if map_count == 0 {
        cancel_map_move_mode(map_list_state);
        map_list_state.status = Some("Collection is empty. Create a map first.".to_string());
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        cancel_map_move_mode(map_list_state);
        return;
    }

    if keys.just_pressed(KeyCode::Enter) {
        if let Err(err) = commit_map_move_mode(map_list_state) {
            map_list_state.status = Some(err);
        }
        return;
    }

    let direction = if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
        Some(Direction::Left)
    } else if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
        Some(Direction::Right)
    } else if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        Some(Direction::Up)
    } else if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        Some(Direction::Down)
    } else {
        None
    };

    let Some(direction) = direction else {
        return;
    };

    let Some(target) =
        map_list_move_target(map_list_state.selected_index, columns, map_count, direction)
    else {
        return;
    };
    if target == map_list_state.selected_index {
        return;
    }

    let moved = map_list_state.maps.remove(map_list_state.selected_index);
    map_list_state.maps.insert(target, moved);
    map_list_state.selected_index = target;
    map_list_state.status = Some("Move mode: arrows move, Enter saves, Esc cancels".to_string());
    bump_map_list_revision(map_list_state);
}

fn begin_map_move_mode(map_list_state: &mut CollectionMapListState) {
    map_list_state.move_mode_active = true;
    map_list_state.move_mode_original_maps = Some(map_list_state.maps.clone());
    map_list_state.move_mode_original_selected_index = map_list_state.selected_index;
    map_list_state.status = Some("Move mode: arrows move, Enter saves, Esc cancels".to_string());
    bump_map_list_revision(map_list_state);
}

fn commit_map_move_mode(map_list_state: &mut CollectionMapListState) -> Result<(), String> {
    save_map_list_order(map_list_state)?;
    map_list_state.move_mode_active = false;
    map_list_state.move_mode_original_maps = None;
    map_list_state.move_mode_original_selected_index = map_list_state.selected_index;
    map_list_state.status = None;
    bump_map_list_revision(map_list_state);
    Ok(())
}

fn cancel_map_move_mode(map_list_state: &mut CollectionMapListState) {
    if let Some(original_maps) = map_list_state.move_mode_original_maps.take() {
        map_list_state.maps = original_maps;
    }
    map_list_state.selected_index = map_list_state
        .move_mode_original_selected_index
        .min(map_list_state.maps.len().saturating_sub(1));
    map_list_state.move_mode_active = false;
    map_list_state.status = None;
    bump_map_list_revision(map_list_state);
}

fn save_map_list_order(map_list_state: &CollectionMapListState) -> Result<(), String> {
    let mut collection = EditableCollection::load(&map_list_state.collection_path)
        .map_err(|err| format!("Failed to load collection: {err:#}"))?;
    collection.maps = map_list_state
        .maps
        .iter()
        .map(|map| EditableMap {
            name: map.name.clone(),
            raw_lines: map.raw_lines.clone(),
        })
        .collect();
    collection
        .save()
        .map_err(|err| format!("Failed to save collection: {err:#}"))
}

fn map_list_move_target(
    current: usize,
    columns: usize,
    map_count: usize,
    direction: Direction,
) -> Option<usize> {
    if map_count == 0 {
        return None;
    }
    match direction {
        Direction::Left => current.checked_sub(1),
        Direction::Right => {
            let next = current + 1;
            (next < map_count).then_some(next)
        }
        Direction::Up => current.checked_sub(columns),
        Direction::Down => {
            let next = current + columns;
            if next < map_count {
                return Some(next);
            }
            let current_row = current / columns;
            let last_row = (map_count - 1) / columns;
            if current_row + 1 != last_row {
                return None;
            }
            let last_index = map_count - 1;
            (current < last_index).then_some(last_index)
        }
    }
}

fn handle_map_name_prompt_input(
    keys: &ButtonInput<KeyCode>,
    map_list_state: &mut CollectionMapListState,
    map_name_prompt: &mut MapNamePromptState,
) {
    if keys.just_pressed(KeyCode::Escape) {
        map_name_prompt.active = false;
        map_name_prompt.mode = None;
        map_name_prompt.buffer.clear();
        map_name_prompt.target_index = None;
        map_list_state.status = None;
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        map_name_prompt.buffer.pop();
        update_map_name_prompt_status(map_list_state, map_name_prompt);
        return;
    }

    if keys.just_pressed(KeyCode::Enter) {
        let outcome = match map_name_prompt.mode {
            Some(MapNamePromptMode::New) => add_new_map(map_list_state, &map_name_prompt.buffer),
            Some(MapNamePromptMode::Rename) => {
                let index = map_name_prompt
                    .target_index
                    .unwrap_or(map_list_state.selected_index);
                rename_existing_map(map_list_state, index, &map_name_prompt.buffer)
            }
            None => Err("Missing map prompt mode".to_string()),
        };

        match outcome {
            Ok(()) => {
                map_name_prompt.active = false;
                map_name_prompt.mode = None;
                map_name_prompt.buffer.clear();
                map_name_prompt.target_index = None;
                map_list_state.status = None;
            }
            Err(err) => {
                map_list_state.status = Some(err);
            }
        }
        return;
    }

    let typed = typed_characters(keys);
    if !typed.is_empty() {
        map_name_prompt.buffer.push_str(&typed);
        map_name_prompt.buffer = truncate_chars(&map_name_prompt.buffer, MAP_NAME_MAX_CHARS);
        update_map_name_prompt_status(map_list_state, map_name_prompt);
    }
}

fn handle_map_delete_confirm_input(
    keys: &ButtonInput<KeyCode>,
    map_list_state: &mut CollectionMapListState,
    map_delete_confirm: &mut MapDeleteConfirmState,
) {
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyN) {
        map_delete_confirm.active = false;
        map_delete_confirm.target_index = None;
        map_list_state.status = None;
        return;
    }

    if !keys.just_pressed(KeyCode::KeyY) {
        return;
    }

    let Some(index) = map_delete_confirm.target_index else {
        map_delete_confirm.active = false;
        map_list_state.status = Some("Delete failed: missing selected map".to_string());
        return;
    };

    if let Err(err) = delete_existing_map(map_list_state, index) {
        map_list_state.status = Some(err);
        return;
    }

    map_delete_confirm.active = false;
    map_delete_confirm.target_index = None;
    map_list_state.status = None;
}

fn add_new_map(map_list_state: &mut CollectionMapListState, raw_name: &str) -> Result<(), String> {
    let mut collection = EditableCollection::load(&map_list_state.collection_path)
        .map_err(|err| format!("Failed to load collection: {err:#}"))?;
    collection
        .add_map(raw_name, default_new_map_lines())
        .map_err(|err| err.to_string())?;
    collection
        .save()
        .map_err(|err| format!("Failed to save collection: {err:#}"))?;
    let new_name = raw_name.trim().to_string();
    reload_collection_map_list_state(map_list_state, Some(&new_name))
}

fn rename_existing_map(
    map_list_state: &mut CollectionMapListState,
    index: usize,
    raw_name: &str,
) -> Result<(), String> {
    let mut collection = EditableCollection::load(&map_list_state.collection_path)
        .map_err(|err| format!("Failed to load collection: {err:#}"))?;
    collection
        .rename_map(index, raw_name)
        .map_err(|err| err.to_string())?;
    collection
        .save()
        .map_err(|err| format!("Failed to save collection: {err:#}"))?;
    let renamed = raw_name.trim().to_string();
    reload_collection_map_list_state(map_list_state, Some(&renamed))
}

fn delete_existing_map(
    map_list_state: &mut CollectionMapListState,
    index: usize,
) -> Result<(), String> {
    let mut collection = EditableCollection::load(&map_list_state.collection_path)
        .map_err(|err| format!("Failed to load collection: {err:#}"))?;
    collection
        .delete_map(index)
        .map_err(|err| err.to_string())?;
    collection
        .save()
        .map_err(|err| format!("Failed to save collection: {err:#}"))?;
    reload_collection_map_list_state(map_list_state, None)?;
    if map_list_state.maps.is_empty() {
        map_list_state.selected_index = 0;
    } else {
        map_list_state.selected_index = map_list_state
            .selected_index
            .min(map_list_state.maps.len().saturating_sub(1));
    }
    Ok(())
}

fn reload_collection_map_list_state(
    map_list_state: &mut CollectionMapListState,
    preferred_map_name: Option<&str>,
) -> Result<(), String> {
    let collection = EditableCollection::load(&map_list_state.collection_path)
        .map_err(|err| format!("Failed to reload collection: {err:#}"))?;
    map_list_state.collection_name = collection.collection_name;
    map_list_state.maps = collection
        .maps
        .iter()
        .map(|map| CollectionMapEntry {
            name: map.name.clone(),
            raw_lines: map.raw_lines.clone(),
        })
        .collect();

    if map_list_state.maps.is_empty() {
        map_list_state.selected_index = 0;
        bump_map_list_revision(map_list_state);
        return Ok(());
    }

    if let Some(target_name) = preferred_map_name
        && let Some(idx) = map_list_state
            .maps
            .iter()
            .position(|map| map.name.eq_ignore_ascii_case(target_name))
    {
        map_list_state.selected_index = idx;
        bump_map_list_revision(map_list_state);
        return Ok(());
    }

    map_list_state.selected_index = map_list_state
        .selected_index
        .min(map_list_state.maps.len().saturating_sub(1));
    bump_map_list_revision(map_list_state);
    Ok(())
}

fn update_map_name_prompt_status(
    map_list_state: &mut CollectionMapListState,
    map_name_prompt: &MapNamePromptState,
) {
    let label = match map_name_prompt.mode {
        Some(MapNamePromptMode::New) => "New map name",
        Some(MapNamePromptMode::Rename) => "Rename map",
        None => "Map name",
    };
    map_list_state.status = Some(format!("{label}: {}_", map_name_prompt.buffer));
}

fn default_new_map_lines() -> Vec<String> {
    vec![
        "#####".to_string(),
        "#@$.#".to_string(),
        "#####".to_string(),
    ]
}

fn bump_map_list_revision(map_list_state: &mut CollectionMapListState) {
    map_list_state.revision = map_list_state.revision.wrapping_add(1);
}

fn update_collection_map_list_selection_visuals(
    map_list_state: Res<CollectionMapListState>,
    mut cards: Query<(&CollectionMapCard, &mut BorderColor)>,
    mut names: Query<(&CollectionMapCardNameText, &mut Text)>,
) {
    if !map_list_state.is_changed() {
        return;
    }

    let selected_color = if map_list_state.move_mode_active {
        Color::srgb(0.35, 0.65, 1.0)
    } else {
        Color::srgb(1.0, 0.9, 0.35)
    };
    for (card, mut border_color) in &mut cards {
        border_color.0 = if card.index == map_list_state.selected_index {
            selected_color
        } else {
            Color::NONE
        };
    }

    for (name, mut text) in &mut names {
        text.sections[0].style.color = if name.index == map_list_state.selected_index {
            selected_color
        } else {
            Color::WHITE
        };
    }
}

fn update_collection_map_list_status_text(
    map_list_state: Res<CollectionMapListState>,
    mut status_text: Query<&mut Text, With<CollectionMapListStatusText>>,
) {
    if !map_list_state.is_changed() {
        return;
    }

    let value = map_list_state.status.clone().unwrap_or_default();
    for mut text in &mut status_text {
        text.sections[0].value = value.clone();
    }
}

fn refresh_collection_map_list(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icon_registry: Res<KeyIconRegistry>,
    sprite_assets: Res<SpriteAssets>,
    primary_window: Query<Ref<Window>, With<PrimaryWindow>>,
    map_list_state: Res<CollectionMapListState>,
    mut layout_state: ResMut<CollectionMapListLayoutState>,
    roots: Query<Entity, With<CollectionMapListRoot>>,
) {
    let (window_height, window_width, window_changed) = match primary_window.get_single() {
        Ok(window) => (window.height(), window.width(), window.is_changed()),
        Err(_) => (768.0, 1024.0, false),
    };

    let columns = map_list_columns(window_width);
    let selected_row = if map_list_state.maps.is_empty() {
        0
    } else {
        map_list_state.selected_index / columns
    };
    let selection_outside_view = selected_row < layout_state.start_row
        || selected_row >= layout_state.end_row
        || columns != layout_state.columns;
    let data_changed = map_list_state.revision != layout_state.rendered_revision;

    if !(window_changed || data_changed || (map_list_state.is_changed() && selection_outside_view))
    {
        return;
    }

    for root in &roots {
        commands.entity(root).despawn_recursive();
    }

    spawn_collection_map_list_entities(
        &mut commands,
        &asset_server,
        &icon_registry,
        &sprite_assets,
        &map_list_state,
        &mut layout_state,
        window_height,
        window_width,
    );
}

fn spawn_collection_map_list_entities(
    commands: &mut Commands,
    asset_server: &AssetServer,
    icon_registry: &KeyIconRegistry,
    sprite_assets: &SpriteAssets,
    map_list_state: &CollectionMapListState,
    layout_state: &mut CollectionMapListLayoutState,
    window_height: f32,
    window_width: f32,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let card_font = font.clone();
    let columns = map_list_columns(window_width);
    let total_maps = map_list_state.maps.len();
    let total_rows = if total_maps == 0 {
        0
    } else {
        total_maps.div_ceil(columns)
    };
    let selected_row = if total_maps == 0 {
        0
    } else {
        map_list_state.selected_index / columns
    };
    let visible_rows = map_list_visible_rows(window_height);
    let (start_row, end_row) = map_list_visible_range(selected_row, total_rows, visible_rows);
    layout_state.start_row = start_row;
    layout_state.end_row = end_row;
    layout_state.columns = columns;
    layout_state.rendered_revision = map_list_state.revision;
    let start_index = start_row * columns;
    let end_index = (end_row * columns).min(total_maps);
    let has_more_above = start_row > 0;
    let has_more_below = end_row < total_rows;

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                background_color: BackgroundColor(GAME_BACKGROUND_COLOR),
                ..default()
            },
            CollectionMapListRoot,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style {
                    margin: UiRect::top(Val::Px(16.0)),
                    ..default()
                },
                text: Text::from_section(
                    map_list_state.collection_name.clone(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 42.0,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            });

            let mut top_marker = NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(26.0),
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

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(MAP_LIST_GRID_WIDTH_PERCENT),
                        padding: UiRect {
                            left: Val::Px(MAP_LIST_GRID_PADDING_LEFT_PX),
                            right: Val::Px(MAP_LIST_GRID_PADDING_RIGHT_PX),
                            top: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                        },
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::FlexStart,
                        align_content: AlignContent::FlexStart,
                        column_gap: Val::Px(MAP_LIST_GRID_COLUMN_GAP_PX),
                        row_gap: Val::Px(MAP_LIST_GRID_COLUMN_GAP_PX),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|grid| {
                    if total_maps == 0 {
                        grid.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(260.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|empty_state| {
                            empty_state.spawn(TextBundle::from_section(
                                "No maps in this collection. Press N to add one.",
                                TextStyle {
                                    font: card_font.clone(),
                                    font_size: 22.0,
                                    color: Color::srgb(0.82, 0.86, 0.92),
                                },
                            ));
                        });
                    } else {
                        for idx in start_index..end_index {
                            let map = &map_list_state.maps[idx];
                            let selected = idx == map_list_state.selected_index;
                            let selected_border = if map_list_state.move_mode_active {
                                Color::srgb(0.35, 0.65, 1.0)
                            } else {
                                Color::srgb(1.0, 0.9, 0.35)
                            };

                            grid.spawn(NodeBundle {
                                style: Style {
                                    min_width: Val::Px(MAP_LIST_CARD_MIN_WIDTH_PX),
                                    min_height: Val::Px(MAP_LIST_CARD_MIN_HEIGHT_PX),
                                    border: UiRect::all(Val::Px(2.0)),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::FlexStart,
                                    align_items: AlignItems::Stretch,
                                    row_gap: Val::Px(MAP_LIST_CARD_INTERNAL_GAP_PX),
                                    ..default()
                                },
                                border_color: BorderColor(if selected {
                                    selected_border
                                } else {
                                    Color::NONE
                                }),
                                ..default()
                            })
                            .insert(CollectionMapCard { index: idx })
                            .with_children(|card| {
                                let name_color = if selected {
                                    selected_border
                                } else {
                                    Color::WHITE
                                };
                                card.spawn((
                                    TextBundle::from_section(
                                        map.name.clone(),
                                        TextStyle {
                                            font: card_font.clone(),
                                            font_size: 20.0,
                                            color: name_color,
                                        },
                                    ),
                                    CollectionMapCardNameText { index: idx },
                                ));
                                spawn_map_preview_tiles(card, sprite_assets, &map.raw_lines);
                            });
                        }
                    }
                });

            let mut bottom_marker = NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(26.0),
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

            parent.spawn((
                TextBundle::from_section(
                    map_list_state.status.clone().unwrap_or_default(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 16.0,
                        color: Color::srgb(0.95, 0.75, 0.4),
                    },
                ),
                CollectionMapListStatusText,
            ));

            parent
                .spawn(NodeBundle {
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
                })
                .with_children(|row| {
                    if map_list_state.move_mode_active {
                        spawn_collection_action_hint(
                            row,
                            asset_server,
                            icon_registry,
                            &font,
                            UiKey::Arrows,
                            "Move Map",
                        );
                        spawn_collection_action_hint(
                            row,
                            asset_server,
                            icon_registry,
                            &font,
                            UiKey::Enter,
                            "Save Move",
                        );
                        spawn_collection_action_hint(
                            row,
                            asset_server,
                            icon_registry,
                            &font,
                            UiKey::Escape,
                            "Cancel Move",
                        );
                    } else {
                        spawn_collection_action_hint(
                            row,
                            asset_server,
                            icon_registry,
                            &font,
                            UiKey::Arrows,
                            "Select Map",
                        );
                        spawn_collection_action_hint(
                            row,
                            asset_server,
                            icon_registry,
                            &font,
                            UiKey::KeyM,
                            "Move Map",
                        );
                        spawn_collection_action_hint(
                            row,
                            asset_server,
                            icon_registry,
                            &font,
                            UiKey::KeyN,
                            "New Map",
                        );
                        spawn_collection_action_hint(
                            row,
                            asset_server,
                            icon_registry,
                            &font,
                            UiKey::KeyO,
                            "Export Collection",
                        );
                        spawn_collection_action_hint(
                            row,
                            asset_server,
                            icon_registry,
                            &font,
                            UiKey::KeyR,
                            "Rename Map",
                        );
                        spawn_collection_action_hint(
                            row,
                            asset_server,
                            icon_registry,
                            &font,
                            UiKey::Delete,
                            "Delete Map",
                        );
                        spawn_collection_action_hint(
                            row,
                            asset_server,
                            icon_registry,
                            &font,
                            UiKey::KeyE,
                            "Edit Map",
                        );
                        spawn_collection_action_hint(
                            row,
                            asset_server,
                            icon_registry,
                            &font,
                            UiKey::Escape,
                            "Back",
                        );
                    }
                });
        });
}

fn map_list_visible_rows(window_height: f32) -> usize {
    // Reserve space for title, markers, hints and status.
    const RESERVED_HEIGHT: f32 = 280.0;
    // Card row height is card min height plus the grid row gap.
    const CARD_ROW_HEIGHT: f32 = MAP_LIST_CARD_MIN_HEIGHT_PX + MAP_LIST_GRID_COLUMN_GAP_PX;
    let rows = ((window_height - RESERVED_HEIGHT).max(0.0) / CARD_ROW_HEIGHT).floor() as usize;
    rows.max(MAP_LIST_MIN_VISIBLE_ROWS)
}

fn map_list_columns(window_width: f32) -> usize {
    let container_width = window_width * (MAP_LIST_GRID_WIDTH_PERCENT / 100.0);
    let usable_width =
        (container_width - MAP_LIST_GRID_PADDING_LEFT_PX - MAP_LIST_GRID_PADDING_RIGHT_PX)
            .max(MAP_LIST_CARD_MIN_WIDTH_PX);
    let card_and_gap = MAP_LIST_CARD_MIN_WIDTH_PX + MAP_LIST_GRID_COLUMN_GAP_PX;
    let columns = ((usable_width + MAP_LIST_GRID_COLUMN_GAP_PX) / card_and_gap).floor() as usize;
    columns.max(1)
}

fn map_list_visible_range(
    selected_row: usize,
    total_rows: usize,
    visible_rows: usize,
) -> (usize, usize) {
    if total_rows <= visible_rows {
        return (0, total_rows);
    }
    let half = visible_rows / 2;
    let mut start = selected_row.saturating_sub(half);
    let max_start = total_rows - visible_rows;
    if start > max_start {
        start = max_start;
    }
    let end = (start + visible_rows).min(total_rows);
    (start, end)
}

fn spawn_map_preview_tiles(
    parent: &mut ChildBuilder,
    sprite_assets: &SpriteAssets,
    lines: &[String],
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(MAP_LIST_CARD_PREVIEW_MAX_WIDTH as f32 * MAP_LIST_PREVIEW_TILE_SIZE),
                height: Val::Px(
                    MAP_LIST_CARD_PREVIEW_MAX_LINES as f32 * MAP_LIST_PREVIEW_TILE_SIZE,
                ),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(0.0),
                ..default()
            },
            ..default()
        })
        .with_children(|grid| {
            for y in 0..MAP_LIST_CARD_PREVIEW_MAX_LINES {
                grid.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::FlexStart,
                        column_gap: Val::Px(0.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    let source_line = lines.get(y).map(String::as_str).unwrap_or("");
                    for x in 0..MAP_LIST_CARD_PREVIEW_MAX_WIDTH {
                        let ch = source_line.chars().nth(x).unwrap_or(' ');
                        row.spawn(ImageBundle {
                            style: Style {
                                width: Val::Px(MAP_LIST_PREVIEW_TILE_SIZE),
                                height: Val::Px(MAP_LIST_PREVIEW_TILE_SIZE),
                                ..default()
                            },
                            image: UiImage::new(preview_tile_handle(sprite_assets, ch)),
                            ..default()
                        });
                    }
                });
            }
        });
}

fn preview_tile_handle(sprite_assets: &SpriteAssets, ch: char) -> Handle<Image> {
    match ch {
        '#' => sprite_assets.wall.clone(),
        '.' => sprite_assets.goal.clone(),
        '$' => sprite_assets.box_tile.clone(),
        '*' => sprite_assets.box_on_goal.clone(),
        '@' => sprite_assets.player.clone(),
        '+' => sprite_assets.player_on_goal.clone(),
        '-' | ' ' => sprite_assets.floor.clone(),
        _ => sprite_assets.floor.clone(),
    }
}

fn setup_map_editor_session(
    map_list_state: Res<CollectionMapListState>,
    mut editor_session: ResMut<EditorSession>,
    mut resize_prompt: ResMut<MapResizePromptState>,
    mut rename_prompt: ResMut<MapRenamePromptState>,
    mut exit_confirm: ResMut<MapExitConfirmState>,
    mut editor_layout: ResMut<MapEditorLayoutState>,
    mut resume_state: ResMut<MapEditorResumeState>,
) {
    editor_layout.rendered_size = None;
    resize_prompt.active = false;
    resize_prompt.buffer.clear();
    rename_prompt.active = false;
    rename_prompt.buffer.clear();
    exit_confirm.active = false;

    if resume_state.resume_existing && editor_session.working_map.is_some() {
        resume_state.resume_existing = false;
        editor_session.status = None;
        return;
    }
    resume_state.resume_existing = false;

    editor_session.collection_path = map_list_state.collection_path.clone();
    editor_session.map_index = map_list_state.selected_index;
    editor_session.selected_brush = EditorTile::Floor;
    editor_session.dirty = false;
    editor_session.undo_stack.clear();
    editor_session.last_painted_cell = None;
    editor_session.baseline_map = None;

    match EditableCollection::load(&map_list_state.collection_path) {
        Ok(collection) => {
            if let Some(raw_map) = collection.maps.get(map_list_state.selected_index) {
                match EditorMap::from_raw_lines(Some(&raw_map.name), &raw_map.raw_lines) {
                    Ok(working_map) => {
                        editor_session.baseline_map = Some(working_map.clone());
                        editor_session.working_map = Some(working_map);
                        editor_session.status = None;
                    }
                    Err(err) => {
                        editor_session.working_map = None;
                        editor_session.status =
                            Some(format!("Failed to initialize editor map: {err:#}"));
                    }
                }
            } else {
                editor_session.working_map = None;
                editor_session.status = Some("Selected map index is out of bounds".to_string());
            }
        }
        Err(err) => {
            editor_session.working_map = None;
            editor_session.status = Some(format!("Failed to load collection: {err:#}"));
        }
    }
}

fn spawn_map_editor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icon_registry: Res<KeyIconRegistry>,
    sprite_assets: Res<SpriteAssets>,
    editor_session: Res<EditorSession>,
    mut editor_layout: ResMut<MapEditorLayoutState>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let info_text = map_editor_subtitle(&editor_session);
    editor_layout.rendered_size = editor_session
        .working_map
        .as_ref()
        .map(|map| (map.width, map.height));
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(16.0),
                    ..default()
                },
                background_color: BackgroundColor(GAME_BACKGROUND_COLOR),
                ..default()
            },
            MapEditorRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        top: Val::Px(16.0),
                        row_gap: Val::Px(6.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|header| {
                    header.spawn(TextBundle::from_section(
                        "Map Editor",
                        TextStyle {
                            font: font.clone(),
                            font_size: 34.0,
                            color: Color::WHITE,
                        },
                    ));
                    header.spawn((
                        TextBundle::from_section(
                            info_text,
                            TextStyle {
                                font: font.clone(),
                                font_size: 24.0,
                                color: Color::srgb(0.86, 0.89, 0.93),
                            },
                        ),
                        MapEditorInfoText,
                    ));
                    header.spawn((
                        TextBundle::from_section(
                            editor_session.status.clone().unwrap_or_default(),
                            TextStyle {
                                font: font.clone(),
                                font_size: 16.0,
                                color: Color::srgb(0.95, 0.75, 0.4),
                            },
                        ),
                        MapEditorStatusText,
                    ));
                });

            parent.spawn((
                TextBundle::from_section(
                    map_editor_warning_message(&editor_session),
                    TextStyle {
                        font: font.clone(),
                        font_size: 16.0,
                        color: Color::srgb(1.0, 0.78, 0.32),
                    },
                )
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    ..default()
                })
                .with_text_justify(JustifyText::Center),
                MapEditorWarningText,
            ));

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|canvas_parent| {
                    if let Some(map) = &editor_session.working_map {
                        spawn_map_editor_canvas(canvas_parent, sprite_assets.as_ref(), map);
                    }
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::FlexStart,
                        position_type: PositionType::Absolute,
                        top: Val::Px(16.0),
                        left: Val::Px(16.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    spawn_collection_action_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        &font,
                        UiKey::Escape,
                        "Back",
                    );
                    spawn_collection_action_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        &font,
                        UiKey::KeyM,
                        "Resize Canvas",
                    );
                    spawn_collection_action_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        &font,
                        UiKey::KeyT,
                        "Play-test",
                    );
                    spawn_collection_multi_key_action_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        &font,
                        UiKey::KeyR,
                        "/",
                        UiKey::KeyF2,
                        "Rename Map",
                    );
                    spawn_mouse_action_hint(
                        row,
                        asset_server.as_ref(),
                        &font,
                        "icons/mouse_left.png",
                        "Paint",
                    );
                    spawn_collection_combo_action_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        &font,
                        UiKey::Ctrl,
                        UiKey::KeyS,
                        "Save",
                    );
                    spawn_collection_combo_action_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        &font,
                        UiKey::Ctrl,
                        UiKey::KeyZ,
                        "Undo",
                    );
                    spawn_map_editor_brush_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        sprite_assets.as_ref(),
                        &font,
                        UiKey::Digit1,
                        EditorTile::Box,
                        "Box",
                    );
                    spawn_map_editor_brush_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        sprite_assets.as_ref(),
                        &font,
                        UiKey::Digit2,
                        EditorTile::BoxOnGoal,
                        "Box on Goal",
                    );
                    spawn_map_editor_brush_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        sprite_assets.as_ref(),
                        &font,
                        UiKey::Digit3,
                        EditorTile::Floor,
                        "Floor",
                    );
                    spawn_map_editor_brush_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        sprite_assets.as_ref(),
                        &font,
                        UiKey::Digit4,
                        EditorTile::Goal,
                        "Goal",
                    );
                    spawn_map_editor_brush_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        sprite_assets.as_ref(),
                        &font,
                        UiKey::Digit5,
                        EditorTile::Wall,
                        "Wall",
                    );
                    spawn_map_editor_brush_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        sprite_assets.as_ref(),
                        &font,
                        UiKey::Digit6,
                        EditorTile::Player,
                        "Player",
                    );
                    spawn_map_editor_brush_hint(
                        row,
                        asset_server.as_ref(),
                        icon_registry.as_ref(),
                        sprite_assets.as_ref(),
                        &font,
                        UiKey::Digit7,
                        EditorTile::PlayerOnGoal,
                        "Player on Goal",
                    );
                });
        });
}

fn despawn_map_editor(mut commands: Commands, roots: Query<Entity, With<MapEditorRoot>>) {
    for root in &roots {
        commands.entity(root).despawn_recursive();
    }
}

fn handle_map_editor_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut editor_session: ResMut<EditorSession>,
    mut resize_prompt: ResMut<MapResizePromptState>,
    mut rename_prompt: ResMut<MapRenamePromptState>,
    mut exit_confirm: ResMut<MapExitConfirmState>,
    canvas_query: Query<&RelativeCursorPosition, With<MapEditorCanvasNode>>,
    mut map_list_state: ResMut<CollectionMapListState>,
    mut play_mode: ResMut<PlaySessionMode>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if exit_confirm.active {
        handle_map_exit_confirm_input(
            &keys,
            &mut editor_session,
            &mut exit_confirm,
            &mut map_list_state,
            &mut next_state,
        );
        return;
    }

    if rename_prompt.active {
        handle_map_rename_prompt_input(&keys, &mut editor_session, &mut rename_prompt);
        return;
    }

    if resize_prompt.active {
        handle_map_resize_prompt_input(&keys, &mut editor_session, &mut resize_prompt);
        return;
    }

    if (keys.just_pressed(KeyCode::KeyR) || keys.just_pressed(KeyCode::F2))
        && let Some(map) = editor_session.working_map.as_ref()
    {
        rename_prompt.active = true;
        rename_prompt.buffer = map.map_name.clone();
        editor_session.status = Some(format!("Rename map: {}_", rename_prompt.buffer));
        return;
    }

    if keys.just_pressed(KeyCode::KeyS) && is_ctrl_pressed(&keys) {
        match save_editor_map(&mut editor_session) {
            Ok(()) => {
                editor_session.status = Some("Saved".to_string());
            }
            Err(err) => {
                editor_session.status = Some(err);
            }
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyZ) && is_ctrl_pressed(&keys) {
        if let Some(previous) = editor_session.undo_stack.pop() {
            editor_session.working_map = Some(previous);
            editor_session.dirty = editor_session.working_map != editor_session.baseline_map;
            editor_session.status = None;
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyT) {
        match start_editor_playtest(&editor_session) {
            Ok(level) => {
                commands.insert_resource(ActiveLevelIndex(0));
                commands.insert_resource(LoadedLevels {
                    levels: vec![level],
                });
                commands.insert_resource(UndoHistory::default());
                *play_mode = PlaySessionMode::EditorPlaytest;
                next_state.set(AppState::Playing);
            }
            Err(err) => {
                editor_session.status = Some(err);
            }
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyM) {
        resize_prompt.active = true;
        resize_prompt.buffer = match editor_session.working_map.as_ref() {
            Some(map) => format!("{}x{}", map.width, map.height),
            None => String::new(),
        };
        editor_session.status = Some(format!("Canvas size: {}_", resize_prompt.buffer));
        return;
    }

    if keys.just_pressed(KeyCode::Digit1) {
        editor_session.selected_brush = EditorTile::Box;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        editor_session.selected_brush = EditorTile::BoxOnGoal;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        editor_session.selected_brush = EditorTile::Floor;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        editor_session.selected_brush = EditorTile::Goal;
    }
    if keys.just_pressed(KeyCode::Digit5) {
        editor_session.selected_brush = EditorTile::Wall;
    }
    if keys.just_pressed(KeyCode::Digit6) {
        editor_session.selected_brush = EditorTile::Player;
    }
    if keys.just_pressed(KeyCode::Digit7) {
        editor_session.selected_brush = EditorTile::PlayerOnGoal;
    }

    let selected_brush = editor_session.selected_brush;
    if mouse_buttons.just_pressed(MouseButton::Left) || mouse_buttons.pressed(MouseButton::Left) {
        let mut painted_coord: Option<(usize, usize)> = None;
        let last_painted = editor_session.last_painted_cell;
        if let Some(map) = editor_session.working_map.as_mut() {
            for cursor in &canvas_query {
                if let Some(position) = cursor.normalized {
                    if !(0.0..1.0).contains(&position.x) || !(0.0..1.0).contains(&position.y) {
                        break;
                    }
                    let x = ((position.x * map.width as f32).floor() as usize).min(map.width - 1);
                    let y = ((position.y * map.height as f32).floor() as usize).min(map.height - 1);
                    let coord = (x, y);
                    if last_painted == Some(coord) {
                        break;
                    }
                    let before = map.clone();
                    if map.paint(x, y, selected_brush).is_ok() {
                        editor_session.undo_stack.push(before);
                        painted_coord = Some(coord);
                    }
                    break;
                }
            }
        }
        if let Some(coord) = painted_coord {
            editor_session.last_painted_cell = Some(coord);
            editor_session.dirty = true;
            editor_session.status = None;
        }
    } else {
        editor_session.last_painted_cell = None;
    }

    if keys.just_pressed(KeyCode::Escape) {
        if editor_session.dirty {
            exit_confirm.active = true;
            editor_session.status =
                Some("Unsaved changes. Save (Y), Discard (N), Cancel (Esc)".to_string());
        } else {
            map_list_state.selected_index = editor_session.map_index;
            next_state.set(AppState::CollectionMapList);
        }
    }
}

fn handle_map_resize_prompt_input(
    keys: &ButtonInput<KeyCode>,
    editor_session: &mut EditorSession,
    resize_prompt: &mut MapResizePromptState,
) {
    if keys.just_pressed(KeyCode::Escape) {
        resize_prompt.active = false;
        resize_prompt.buffer.clear();
        editor_session.status = None;
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        resize_prompt.buffer.pop();
        editor_session.status = Some(format!("Canvas size: {}_", resize_prompt.buffer));
        return;
    }

    if keys.just_pressed(KeyCode::Enter) {
        let Some((width, height)) = parse_canvas_size_input(&resize_prompt.buffer) else {
            editor_session.status = Some("Invalid size. Use format {Width}x{Height}".to_string());
            return;
        };

        let Some(map) = editor_session.working_map.as_mut() else {
            editor_session.status = Some("No map loaded".to_string());
            return;
        };

        let before = map.clone();
        match map.resize(width, height) {
            Ok(()) => {
                editor_session.undo_stack.push(before);
                resize_prompt.active = false;
                resize_prompt.buffer.clear();
                editor_session.dirty = true;
                editor_session.status = None;
            }
            Err(err) => {
                editor_session.status = Some(format!("Resize failed: {err}"));
            }
        }
        return;
    }

    let typed = typed_characters(keys);
    if !typed.is_empty() {
        resize_prompt.buffer.push_str(&typed);
        resize_prompt.buffer = truncate_chars(&resize_prompt.buffer, 16);
        editor_session.status = Some(format!("Canvas size: {}_", resize_prompt.buffer));
    }
}

fn handle_map_rename_prompt_input(
    keys: &ButtonInput<KeyCode>,
    editor_session: &mut EditorSession,
    rename_prompt: &mut MapRenamePromptState,
) {
    if keys.just_pressed(KeyCode::Escape) {
        rename_prompt.active = false;
        rename_prompt.buffer.clear();
        editor_session.status = None;
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) {
        rename_prompt.buffer.pop();
        editor_session.status = Some(format!("Rename map: {}_", rename_prompt.buffer));
        return;
    }

    if keys.just_pressed(KeyCode::Enter) {
        let Some(map) = editor_session.working_map.as_mut() else {
            editor_session.status = Some("No map loaded".to_string());
            return;
        };
        let before = map.clone();
        match map.rename_map(&rename_prompt.buffer) {
            Ok(()) => {
                editor_session.undo_stack.push(before);
                rename_prompt.active = false;
                rename_prompt.buffer.clear();
                editor_session.dirty = true;
                editor_session.status = None;
            }
            Err(err) => {
                editor_session.status = Some(format!("Rename failed: {err}"));
            }
        }
        return;
    }

    let typed = typed_characters(keys);
    if !typed.is_empty() {
        rename_prompt.buffer.push_str(&typed);
        rename_prompt.buffer = truncate_chars(&rename_prompt.buffer, MAP_NAME_MAX_CHARS);
        editor_session.status = Some(format!("Rename map: {}_", rename_prompt.buffer));
    }
}

fn handle_map_exit_confirm_input(
    keys: &ButtonInput<KeyCode>,
    editor_session: &mut EditorSession,
    exit_confirm: &mut MapExitConfirmState,
    map_list_state: &mut CollectionMapListState,
    next_state: &mut NextState<AppState>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit_confirm.active = false;
        editor_session.status = None;
        return;
    }

    if keys.just_pressed(KeyCode::KeyN) {
        exit_confirm.active = false;
        editor_session.status = None;
        map_list_state.selected_index = editor_session.map_index;
        next_state.set(AppState::CollectionMapList);
        return;
    }

    if keys.just_pressed(KeyCode::KeyY) {
        match save_editor_map(editor_session) {
            Ok(()) => {
                exit_confirm.active = false;
                map_list_state.selected_index = editor_session.map_index;
                next_state.set(AppState::CollectionMapList);
            }
            Err(err) => {
                editor_session.status = Some(err);
            }
        }
    }
}

fn save_editor_map(editor_session: &mut EditorSession) -> Result<(), String> {
    let Some(map) = editor_session.working_map.as_ref() else {
        return Err("No map loaded".to_string());
    };

    let mut collection = EditableCollection::load(&editor_session.collection_path)
        .map_err(|err| format!("Failed to load collection: {err:#}"))?;
    if editor_session.map_index >= collection.maps.len() {
        return Err("Map index is out of bounds".to_string());
    }

    collection.maps[editor_session.map_index].name = map.map_name.clone();
    collection.maps[editor_session.map_index].raw_lines = map.to_raw_lines();
    collection
        .save()
        .map_err(|err| format!("Failed to save collection: {err:#}"))?;

    editor_session.baseline_map = Some(map.clone());
    editor_session.dirty = false;
    Ok(())
}

fn start_editor_playtest(editor_session: &EditorSession) -> Result<Level, String> {
    let Some(map) = editor_session.working_map.as_ref() else {
        return Err("No map loaded".to_string());
    };
    let raw = RawLevel {
        name: Some(map.map_name.clone()),
        lines: map.to_raw_lines(),
    };
    raw.to_level()
        .map_err(|err| format!("Play-test blocked: {err:#}"))
}

fn update_map_editor_text(
    editor_session: Res<EditorSession>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<MapEditorInfoText>>,
        Query<&mut Text, With<MapEditorStatusText>>,
    )>,
) {
    if !editor_session.is_changed() {
        return;
    }

    let info = map_editor_subtitle(&editor_session);
    for mut text in &mut text_sets.p0() {
        text.sections[0].value = info.clone();
    }

    let status = editor_session.status.clone().unwrap_or_default();
    for mut text in &mut text_sets.p1() {
        text.sections[0].value = status.clone();
    }
}

fn update_map_editor_warning_text(
    editor_session: Res<EditorSession>,
    mut warning_text: Query<&mut Text, With<MapEditorWarningText>>,
) {
    if !editor_session.is_changed() {
        return;
    }
    let warning = map_editor_warning_message(&editor_session);
    for mut text in &mut warning_text {
        text.sections[0].value = warning.clone();
    }
}

fn map_editor_subtitle(editor_session: &EditorSession) -> String {
    match &editor_session.working_map {
        Some(map) => {
            let dirty_suffix = if editor_session.dirty { "*" } else { "" };
            format!(
                "{} ({}x{}){}",
                map.map_name, map.width, map.height, dirty_suffix
            )
        }
        None => "Map: <failed to load>".to_string(),
    }
}

fn map_editor_warning_message(editor_session: &EditorSession) -> String {
    let Some(map) = editor_session.working_map.as_ref() else {
        return String::new();
    };
    match map.simple_warning() {
        Some(SimpleWarning::MissingBoxes) => "Warning: map has no boxes".to_string(),
        Some(SimpleWarning::BoxesMoreThanGoals { boxes, goals }) => {
            format!("Warning: boxes ({boxes}) exceed goals ({goals})")
        }
        Some(SimpleWarning::AlreadySolved) => {
            "Warning: map is already solved (all boxes are on goals)".to_string()
        }
        Some(SimpleWarning::MissingPlayer) => "Warning: map has no player".to_string(),
        None => String::new(),
    }
}

fn refresh_map_editor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icon_registry: Res<KeyIconRegistry>,
    sprite_assets: Res<SpriteAssets>,
    editor_session: Res<EditorSession>,
    editor_layout: ResMut<MapEditorLayoutState>,
    roots: Query<Entity, With<MapEditorRoot>>,
) {
    let desired_size = editor_session
        .working_map
        .as_ref()
        .map(|map| (map.width, map.height));
    if desired_size == editor_layout.rendered_size {
        return;
    }

    for root in &roots {
        commands.entity(root).despawn_recursive();
    }

    spawn_map_editor(
        commands,
        asset_server,
        icon_registry,
        sprite_assets,
        editor_session,
        editor_layout,
    );
}

fn update_map_editor_visuals(
    editor_session: Res<EditorSession>,
    mut brush_rows: Query<(&MapEditorBrushRow, &mut BorderColor, &mut BackgroundColor)>,
) {
    if !editor_session.is_changed() {
        return;
    }

    for (row, mut border, mut bg) in &mut brush_rows {
        let selected = row.brush == editor_session.selected_brush;
        border.0 = if selected {
            Color::srgb(1.0, 0.9, 0.35)
        } else {
            Color::NONE
        };
        bg.0 = if selected {
            Color::srgb(0.22, 0.22, 0.24)
        } else {
            Color::NONE
        };
    }
}

fn spawn_map_editor_canvas(
    parent: &mut ChildBuilder,
    sprite_assets: &SpriteAssets,
    map: &EditorMap,
) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(map.width as f32 * MAP_EDITOR_TILE_SIZE),
                    height: Val::Px(map.height as f32 * MAP_EDITOR_TILE_SIZE),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(0.0),
                    ..default()
                },
                border_color: BorderColor(Color::srgb(0.4, 0.4, 0.45)),
                ..default()
            },
            RelativeCursorPosition::default(),
            MapEditorCanvasNode,
        ))
        .with_children(|grid| {
            for y in 0..map.height {
                grid.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(0.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    for x in 0..map.width {
                        let cell = map.get(x, y).unwrap_or(crate::editor::Cell::Floor);
                        row.spawn((
                            ImageBundle {
                                style: Style {
                                    width: Val::Px(MAP_EDITOR_TILE_SIZE),
                                    height: Val::Px(MAP_EDITOR_TILE_SIZE),
                                    ..default()
                                },
                                image: UiImage::new(editor_cell_image_handle(sprite_assets, cell)),
                                ..default()
                            },
                            MapEditorCanvasTile { x, y },
                        ));
                    }
                });
            }
        });
}

fn spawn_map_editor_brush_hint(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    icon_registry: &KeyIconRegistry,
    sprite_assets: &SpriteAssets,
    font: &Handle<Font>,
    key: UiKey,
    brush: EditorTile,
    label: &str,
) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(5.0),
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                    ..default()
                },
                border_color: BorderColor(Color::NONE),
                ..default()
            },
            MapEditorBrushRow { brush },
        ))
        .with_children(|hint| {
            spawn_key_icon(hint, asset_server, icon_registry, key);
            hint.spawn(TextBundle::from_section(
                ":",
                TextStyle {
                    font: font.clone(),
                    font_size: 18.0,
                    color: Color::srgb(0.75, 0.8, 0.85),
                },
            ));
            hint.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(22.0),
                    height: Val::Px(22.0),
                    ..default()
                },
                image: UiImage::new(editor_brush_image_handle(sprite_assets, brush)),
                ..default()
            });
            hint.spawn(TextBundle::from_section(
                format!(" {label}"),
                TextStyle {
                    font: font.clone(),
                    font_size: 18.0,
                    color: Color::srgb(0.75, 0.8, 0.85),
                },
            ));
        });
}

fn editor_brush_image_handle(sprite_assets: &SpriteAssets, brush: EditorTile) -> Handle<Image> {
    match brush {
        EditorTile::Box => sprite_assets.box_tile.clone(),
        EditorTile::BoxOnGoal => sprite_assets.box_on_goal.clone(),
        EditorTile::Floor => sprite_assets.floor.clone(),
        EditorTile::Goal => sprite_assets.goal.clone(),
        EditorTile::Wall => sprite_assets.wall.clone(),
        EditorTile::Player => sprite_assets.player.clone(),
        EditorTile::PlayerOnGoal => sprite_assets.player_on_goal.clone(),
    }
}

fn editor_cell_image_handle(
    sprite_assets: &SpriteAssets,
    cell: crate::editor::Cell,
) -> Handle<Image> {
    match cell {
        crate::editor::Cell::Box => sprite_assets.box_tile.clone(),
        crate::editor::Cell::BoxOnGoal => sprite_assets.box_on_goal.clone(),
        crate::editor::Cell::Floor => sprite_assets.floor.clone(),
        crate::editor::Cell::Goal => sprite_assets.goal.clone(),
        crate::editor::Cell::Wall => sprite_assets.wall.clone(),
        crate::editor::Cell::Player => sprite_assets.player.clone(),
        crate::editor::Cell::PlayerOnGoal => sprite_assets.player_on_goal.clone(),
    }
}

fn update_map_editor_hover_preview(
    editor_session: Res<EditorSession>,
    sprite_assets: Res<SpriteAssets>,
    canvas_query: Query<&RelativeCursorPosition, With<MapEditorCanvasNode>>,
    mut tiles: Query<(&MapEditorCanvasTile, &mut UiImage)>,
) {
    let Some(map) = editor_session.working_map.as_ref() else {
        return;
    };

    let hover_coord = canvas_query
        .iter()
        .find_map(|cursor| cursor.normalized)
        .map(|position| {
            if !(0.0..1.0).contains(&position.x) || !(0.0..1.0).contains(&position.y) {
                return None;
            }
            let x = ((position.x * map.width as f32).floor() as usize).min(map.width - 1);
            let y = ((position.y * map.height as f32).floor() as usize).min(map.height - 1);
            Some((x, y))
        })
        .flatten();

    for (tile, mut image) in &mut tiles {
        let base_cell = map
            .get(tile.x, tile.y)
            .unwrap_or(crate::editor::Cell::Floor);
        image.texture = editor_cell_image_handle(sprite_assets.as_ref(), base_cell);
        image.color = Color::WHITE;

        let selected_cell = editor_brush_to_cell(editor_session.selected_brush);
        if let Some((hx, hy)) = hover_coord
            && tile.x == hx
            && tile.y == hy
            && base_cell != selected_cell
        {
            image.texture =
                editor_brush_image_handle(sprite_assets.as_ref(), editor_session.selected_brush);
            image.color = Color::srgba(1.0, 1.0, 1.0, MAP_EDITOR_PHANTOM_ALPHA);
        }
    }
}

fn editor_brush_to_cell(brush: EditorTile) -> crate::editor::Cell {
    match brush {
        EditorTile::Box => crate::editor::Cell::Box,
        EditorTile::BoxOnGoal => crate::editor::Cell::BoxOnGoal,
        EditorTile::Floor => crate::editor::Cell::Floor,
        EditorTile::Goal => crate::editor::Cell::Goal,
        EditorTile::Wall => crate::editor::Cell::Wall,
        EditorTile::Player => crate::editor::Cell::Player,
        EditorTile::PlayerOnGoal => crate::editor::Cell::PlayerOnGoal,
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

fn spawn_collection_combo_action_hint(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    icon_registry: &KeyIconRegistry,
    font: &Handle<Font>,
    first: UiKey,
    second: UiKey,
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
            spawn_key_icon(hint, asset_server, icon_registry, first);
            hint.spawn(TextBundle::from_section(
                "+",
                TextStyle {
                    font: font.clone(),
                    font_size: 18.0,
                    color: Color::srgb(0.75, 0.8, 0.85),
                },
            ));
            spawn_key_icon(hint, asset_server, icon_registry, second);
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

fn spawn_collection_multi_key_action_hint(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    icon_registry: &KeyIconRegistry,
    font: &Handle<Font>,
    first: UiKey,
    separator: &str,
    second: UiKey,
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
            spawn_key_icon(hint, asset_server, icon_registry, first);
            hint.spawn(TextBundle::from_section(
                separator,
                TextStyle {
                    font: font.clone(),
                    font_size: 18.0,
                    color: Color::srgb(0.75, 0.8, 0.85),
                },
            ));
            spawn_key_icon(hint, asset_server, icon_registry, second);
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

fn spawn_mouse_action_hint(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    font: &Handle<Font>,
    mouse_icon_path: &'static str,
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
            hint.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                image: UiImage::new(asset_server.load(mouse_icon_path)),
                ..default()
            });
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
    mut play_mode: ResMut<PlaySessionMode>,
    mut resume_state: ResMut<MapEditorResumeState>,
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
    *play_mode = PlaySessionMode::NormalCollection;
    resume_state.resume_existing = false;
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

    if keys.just_pressed(KeyCode::KeyZ) {
        undo_repeat.reset();
        if let Some(previous) = undo_history.0.pop_back() {
            current_state.0 = previous;
        }
        return;
    }

    if !keys.pressed(KeyCode::KeyZ) {
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
    mut play_mode: ResMut<PlaySessionMode>,
    mut resume_state: ResMut<MapEditorResumeState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyQ) {
        if *play_mode == PlaySessionMode::EditorPlaytest {
            resume_state.resume_existing = true;
            *play_mode = PlaySessionMode::NormalCollection;
            next_state.set(AppState::MapEditor);
        } else {
            next_state.set(AppState::CollectionMenu);
        }
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

fn export_collection_with_save_dialog(
    map_list_state: &CollectionMapListState,
) -> Result<String, String> {
    if map_list_state.collection_path.trim().is_empty() {
        return Err("Export failed: collection path is empty".to_string());
    }

    let source_path = PathBuf::from(&map_list_state.collection_path);
    if !source_path.exists() {
        return Err(format!(
            "Export failed: source does not exist '{}'",
            display_absolute_path(&source_path)
        ));
    }

    let mut dialog = rfd::FileDialog::new()
        .add_filter("Text collections", &["txt"])
        .set_file_name(&suggested_export_file_name(map_list_state));
    if let Some(downloads_dir) = default_downloads_dir() {
        dialog = dialog.set_directory(downloads_dir);
    }

    let Some(mut destination) = dialog.save_file() else {
        return Err("Export canceled".to_string());
    };
    if destination.extension().is_none() {
        destination.set_extension("txt");
    }

    fs::copy(&source_path, &destination).map_err(|err| {
        format!(
            "Export failed (copy to '{}'): {}",
            display_absolute_path(&destination),
            err
        )
    })?;

    Ok(format!(
        "Exported to '{}'",
        display_absolute_path(&destination)
    ))
}

fn suggested_export_file_name(map_list_state: &CollectionMapListState) -> String {
    let source_path = Path::new(&map_list_state.collection_path);
    if let Some(name) = source_path.file_name().and_then(|name| name.to_str())
        && !name.trim().is_empty()
    {
        return name.to_string();
    }

    let base: String = map_list_state
        .collection_name
        .chars()
        .map(|ch| {
            if is_invalid_filename_char(ch) || ch.is_control() {
                '_'
            } else {
                ch
            }
        })
        .collect();
    let base = base.trim();
    let base = if base.is_empty() { "collection" } else { base };
    format!("{}.txt", truncate_chars(base, COLLECTION_NAME_MAX_CHARS))
}

fn default_downloads_dir() -> Option<PathBuf> {
    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        let downloads = home.join("Downloads");
        if downloads.is_dir() {
            return Some(downloads);
        }
        if home.is_dir() {
            return Some(home);
        }
    }

    if let Some(user_profile) = env::var_os("USERPROFILE").map(PathBuf::from) {
        let downloads = user_profile.join("Downloads");
        if downloads.is_dir() {
            return Some(downloads);
        }
        if user_profile.is_dir() {
            return Some(user_profile);
        }
    }

    None
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

fn parse_canvas_size_input(input: &str) -> Option<(usize, usize)> {
    let trimmed = input.trim();
    let (w, h) = trimmed
        .split_once('x')
        .or_else(|| trimmed.split_once('X'))?;
    let width = w.trim().parse::<usize>().ok()?;
    let height = h.trim().parse::<usize>().ok()?;
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

fn is_ctrl_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)
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
