//! Game UI shell: top bar (status, speed, map mode, world), bottom god dock
//! with category tabs and pixel icons, tool options, a selection panel and a
//! shortcut help overlay. Mouse first; every action also has a key.
use bevy::prelude::*;
use sim_core::terrain::{MapSize, WorldTemplate};

use crate::{
    sprites::{Icon, SpriteBank, SpriteKey},
    GodTool, InspectorText, ToolCategory, ToolContextText, ViewerState, WorldDynamic,
};

const PANEL: Color = Color::srgba(0.07, 0.08, 0.10, 0.93);
const GOLD: Color = Color::srgb(0.80, 0.66, 0.36);
const BUTTON: Color = Color::srgb(0.14, 0.15, 0.18);
const HOVER: Color = Color::srgb(0.22, 0.23, 0.28);
const ACTIVE: Color = Color::srgb(0.46, 0.35, 0.14);
const TEXT: Color = Color::srgb(0.93, 0.91, 0.84);
const DIM: Color = Color::srgb(0.66, 0.66, 0.62);

#[derive(Component)] pub(crate) struct StatusText;
#[derive(Component)] pub(crate) struct CursorText;
#[derive(Component)] pub(crate) struct InspectorPanel;
#[derive(Component)] pub(crate) struct HelpPanel;
#[derive(Component)] pub(crate) struct WorldLabel;
#[derive(Component)] pub(crate) struct MapLabel;
#[derive(Component, Clone, Copy)] pub(crate) struct SpeedButton(f32);
#[derive(Component, Clone, Copy)] pub(crate) struct CategoryTab(ToolCategory);
#[derive(Component, Clone, Copy)] pub(crate) struct ToolSlot(GodTool);
#[derive(Component, Clone, Copy, PartialEq, Eq)] pub(crate) enum Action { NewWorld, NextTemplate, NextSize, NextMapMode, Help }

#[derive(Resource)]
pub struct UiState { category: ToolCategory, help: bool, last_tool: GodTool }
impl Default for UiState {
    fn default() -> Self { Self { category: ToolCategory::Observe, help: false, last_tool: GodTool::Inspect } }
}

const TOOLS: [(GodTool, Icon, &str); 13] = [
    (GodTool::Inspect, Icon::Inspect, "관찰"),
    (GodTool::Resident, Icon::Person, "사람"),
    (GodTool::Animal, Icon::Animal, "동물"),
    (GodTool::Monster, Icon::Monster, "괴물"),
    (GodTool::Vegetation, Icon::Tree, "숲"),
    (GodTool::Mineral, Icon::Ore, "광석"),
    (GodTool::Rock, Icon::Rock, "산"),
    (GodTool::Water, Icon::Water, "물"),
    (GodTool::Rain, Icon::Rain, "비"),
    (GodTool::Drought, Icon::Sun, "가뭄"),
    (GodTool::Fire, Icon::Fire, "불"),
    (GodTool::Flood, Icon::Flood, "홍수"),
    (GodTool::Earthquake, Icon::Quake, "지진"),
];
const CATEGORIES: [(ToolCategory, &str); 4] = [
    (ToolCategory::Observe, "관찰"), (ToolCategory::Life, "생명"),
    (ToolCategory::Nature, "자연"), (ToolCategory::Disaster, "재해"),
];

/// Galmuri14 is drawn for 15 px; other sizes blur the pixel glyphs.
pub const FONT_PX: f32 = 15.0;

pub fn template_name(t: WorldTemplate) -> &'static str {
    match t {
        WorldTemplate::Continents => "대륙", WorldTemplate::Archipelago => "군도", WorldTemplate::Pangaea => "판게아",
        WorldTemplate::Islands => "섬", WorldTemplate::Lakes => "호수",
    }
}

pub fn map_mode_name(m: sim_core::terrain::palette::MapMode) -> &'static str {
    use sim_core::terrain::palette::MapMode::*;
    match m { Terrain => "지형", Height => "고도", Temperature => "기온", Moisture => "습도", Biome => "생물군계" }
}

/// Replace Bevy's default font with the embedded Galmuri so Korean renders everywhere.
pub fn install_font(mut fonts: ResMut<Assets<Font>>) {
    let font = Font::from_bytes(include_bytes!("../assets/fonts/Galmuri14.ttf").to_vec());
    let _ = fonts.insert(&Handle::<Font>::default(), font);
}

fn label(text: &str, size: f32, colour: Color) -> impl Bundle {
    (Text::new(text), TextFont::from_font_size(size), TextColor(colour), TextLayout::new(Justify::Left, LineBreak::NoWrap))
}

fn text_button(text: &str) -> impl Bundle {
    (
        Button,
        Node { padding: UiRect::axes(px(9), px(5)), margin: UiRect::horizontal(px(2)), border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(4)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(BUTTON), BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)),
        children![label(text, FONT_PX, TEXT)],
    )
}

pub fn spawn_ui(mut commands: Commands, mut bank: ResMut<SpriteBank>, mut images: ResMut<Assets<Image>>) {
    let mut icon = |i: Icon| bank.get(SpriteKey::Icon(i), &mut images).0;
    let pause_icon = icon(Icon::Pause);

    // Top bar.
    commands.spawn((
        Node { position_type: PositionType::Absolute, top: px(0), left: px(0), right: px(0), height: px(44),
            flex_direction: FlexDirection::Row, align_items: AlignItems::Center, justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::horizontal(px(12)), border: UiRect::bottom(px(2)), ..default() },
        BackgroundColor(PANEL), BorderColor::all(GOLD), Interaction::default(),
    )).with_children(|bar| {
        bar.spawn((label("", FONT_PX, TEXT), StatusText));
        bar.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }).with_children(|g| {
            g.spawn((Button, Node { width: px(30), height: px(28), margin: UiRect::horizontal(px(2)), border_radius: BorderRadius::all(px(4)),
                justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                BackgroundColor(BUTTON), SpeedButton(0.0)))
                .with_child((ImageNode { image: pause_icon.clone(), ..default() }, Node { width: px(18), height: px(18), ..default() }));
            for (v, t) in [(1.0, "1x"), (5.0, "5x"), (20.0, "20x"), (100.0, "100x"), (1000.0, "1000x")] {
                g.spawn((text_button(t), SpeedButton(v)));
            }
        });
        bar.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, ..default() }).with_children(|g| {
            g.spawn((
                Button,
                Node { padding: UiRect::axes(px(9), px(5)), margin: UiRect::horizontal(px(2)), border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)), ..default() },
                BackgroundColor(BUTTON), BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)), Action::NextMapMode,
            )).with_child((label("", FONT_PX, TEXT), MapLabel));
            g.spawn((
                Button,
                Node { padding: UiRect::axes(px(9), px(5)), margin: UiRect::horizontal(px(2)), border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)), ..default() },
                BackgroundColor(BUTTON), BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)), Action::NextTemplate,
            )).with_child((label("", FONT_PX, TEXT), WorldLabel));
            g.spawn((text_button("크기"), Action::NextSize));
            g.spawn((text_button("새 월드"), Action::NewWorld));
            g.spawn((text_button("?"), Action::Help));
        });
    });

    // God dock: category tabs over a row of tool slots.
    commands.spawn(Node { position_type: PositionType::Absolute, bottom: px(8), left: px(0), right: px(0),
        justify_content: JustifyContent::Center, ..default() })
    .with_children(|row| {
        row.spawn((
            Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::all(px(6)),
                border: UiRect::all(px(2)), border_radius: BorderRadius::all(px(8)), ..default() },
            BackgroundColor(PANEL), BorderColor::all(GOLD), Interaction::default(),
        )).with_children(|dock| {
            dock.spawn(Node { flex_direction: FlexDirection::Row, margin: UiRect::bottom(px(5)), ..default() }).with_children(|tabs| {
                for (c, t) in CATEGORIES { tabs.spawn((text_button(t), CategoryTab(c))); }
            });
            dock.spawn(Node { flex_direction: FlexDirection::Row, ..default() }).with_children(|slots| {
                for (tool, i, t) in TOOLS {
                    slots.spawn((
                        Button,
                        Node { width: px(60), height: px(62), margin: UiRect::horizontal(px(3)), flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(px(1)),
                            border_radius: BorderRadius::all(px(6)), ..default() },
                        BackgroundColor(BUTTON), BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.08)), ToolSlot(tool),
                    )).with_children(|b| {
                        b.spawn((ImageNode { image: icon(i), ..default() }, Node { width: px(32), height: px(32), ..default() }));
                        b.spawn(label(t, FONT_PX, TEXT));
                    });
                }
            });
        });
    });

    // Tool options, above the dock on the right.
    commands.spawn((
        Node { position_type: PositionType::Absolute, bottom: px(118), right: px(12), padding: UiRect::all(px(10)),
            border: UiRect::all(px(1)), border_radius: BorderRadius::all(px(6)), ..default() },
        BackgroundColor(PANEL), BorderColor::all(Color::srgba(0.8, 0.66, 0.36, 0.5)), Interaction::default(),
    )).with_child((label("", FONT_PX, TEXT), ToolContextText));

    // Selection panel.
    commands.spawn((
        Node { position_type: PositionType::Absolute, top: px(54), right: px(12), width: px(340), padding: UiRect::all(px(12)),
            border: UiRect::all(px(2)), border_radius: BorderRadius::all(px(8)), ..default() },
        BackgroundColor(PANEL), BorderColor::all(GOLD), Visibility::Hidden, InspectorPanel, Interaction::default(),
    )).with_child((label("", FONT_PX, TEXT), InspectorText));

    // Cursor tile readout.
    commands.spawn((label("", FONT_PX, DIM), Node { position_type: PositionType::Absolute, bottom: px(12), left: px(12), ..default() }, CursorText));

    // Shortcut help.
    commands.spawn((
        Node { position_type: PositionType::Absolute, top: px(70), left: px(24), padding: UiRect::all(px(14)),
            border: UiRect::all(px(2)), border_radius: BorderRadius::all(px(8)), ..default() },
        BackgroundColor(PANEL), BorderColor::all(GOLD), Visibility::Hidden, HelpPanel,
    )).with_child(label(
        "카메라\n  휠: 커서 쪽으로 확대/축소    우클릭·휠클릭 드래그: 이동\n  좌클릭 드래그(관찰 도구): 이동    화면 가장자리: 스크롤    WASD/방향키: 이동\n\n\
         시간\n  Space: 일시정지    1-5: x1 / x5 / x20 / x100 / x1000\n\n\
         월드\n  R: 새 월드    `: 다음 지형 형태    \\: 다음 크기    Tab: 지도 모드\n\n\
         신의 도구\n  I 관찰  H 사람  Z 동물  M 괴물  T 숲  O 광석\n  N 비  X 가뭄  F 불  G 홍수  Q 지진\n  [ ]: 반경    , .: 세기    L: 괴물 연구소    V: 정착지 현황\n\n\
         F1: 도움말 닫기", FONT_PX, TEXT));
}

pub fn ui_buttons(
    mut commands: Commands,
    mut state: ResMut<ViewerState>,
    mut ui: ResMut<UiState>,
    dynamic: Query<Entity, With<WorldDynamic>>,
    mut buttons: Query<(Ref<Interaction>, &mut BackgroundColor, Option<&SpeedButton>, Option<&CategoryTab>, Option<&ToolSlot>, Option<&Action>), With<Button>>,
) {
    let mut rebuild = false;
    for (interaction, mut bg, speed, tab, slot, action) in &mut buttons {
        if interaction.is_changed() && *interaction == Interaction::Pressed {
            if let Some(SpeedButton(v)) = speed {
                if *v == 0.0 { state.paused = !state.paused; } else { state.speed = *v; state.paused = false; }
            }
            if let Some(CategoryTab(c)) = tab { ui.category = *c; }
            if let Some(ToolSlot(t)) = slot { state.tool = *t; }
            match action {
                Some(Action::NewWorld) => { state.seed = state.seed.wrapping_add(1); rebuild = true; }
                Some(Action::NextTemplate) => {
                    let i = WorldTemplate::ALL.iter().position(|t| *t == state.template).unwrap_or(0);
                    state.template = WorldTemplate::ALL[(i + 1) % WorldTemplate::ALL.len()];
                    rebuild = true;
                }
                Some(Action::NextSize) => {
                    let i = MapSize::ALL.iter().position(|t| *t == state.map_size).unwrap_or(0);
                    state.map_size = MapSize::ALL[(i + 1) % MapSize::ALL.len()];
                    rebuild = true;
                }
                Some(Action::NextMapMode) => state.map_mode = state.map_mode.next(),
                Some(Action::Help) => ui.help = !ui.help,
                None => {}
            }
        }
        let selected = match (speed, tab, slot) {
            (Some(SpeedButton(v)), ..) => if *v == 0.0 { state.paused } else { !state.paused && state.speed == *v },
            (_, Some(CategoryTab(c)), _) => ui.category == *c,
            (_, _, Some(ToolSlot(t))) => state.tool == *t,
            _ => false,
        };
        let target = if selected { ACTIVE } else if *interaction == Interaction::Hovered { HOVER } else { BUTTON };
        if bg.0 != target { bg.0 = target; }
    }
    if rebuild {
        state.rebuild_world();
        for e in &dynamic { commands.entity(e).despawn(); }
    }
}

pub fn ui_layout(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<ViewerState>,
    mut ui: ResMut<UiState>,
    mut slots: Query<(&ToolSlot, &mut Node)>,
    mut inspector: Query<&mut Visibility, (With<InspectorPanel>, Without<HelpPanel>)>,
    mut help: Query<&mut Visibility, (With<HelpPanel>, Without<InspectorPanel>)>,
) {
    // Keyboard tool changes pull the dock to that tool's category.
    if state.tool != ui.last_tool {
        ui.last_tool = state.tool;
        ui.category = state.tool.category();
    }
    if keys.just_pressed(KeyCode::F1) && !state.monster_lab { ui.help = !ui.help; }
    for (slot, mut node) in &mut slots {
        let show = if slot.0.category() == ui.category { Display::Flex } else { Display::None };
        if node.display != show { node.display = show; }
    }
    if let Ok(mut v) = inspector.single_mut() {
        *v = if state.selected.is_some() && !state.monster_lab { Visibility::Visible } else { Visibility::Hidden };
    }
    if let Ok(mut v) = help.single_mut() {
        *v = if ui.help { Visibility::Visible } else { Visibility::Hidden };
    }
}

pub fn ui_status(
    state: Res<ViewerState>,
    mut status: Query<&mut Text, (With<StatusText>, Without<CursorText>, Without<WorldLabel>, Without<MapLabel>)>,
    mut cursor: Query<&mut Text, (With<CursorText>, Without<StatusText>, Without<WorldLabel>, Without<MapLabel>)>,
    mut world: Query<&mut Text, (With<WorldLabel>, Without<StatusText>, Without<CursorText>, Without<MapLabel>)>,
    mut map: Query<&mut Text, (With<MapLabel>, Without<StatusText>, Without<CursorText>, Without<WorldLabel>)>,
) {
    let sim = &state.sim;
    if let Ok(mut t) = status.single_mut() {
        let alive = sim.residents.iter().filter(|r| r.health > 0.0).count();
        let villages = sim.settlements.iter().filter(|s| !s.members.is_empty()).count();
        t.0 = format!("{:.1}년   인구 {}   가구 {}   마을 {}   건물 {}{}",
            sim.year, alive, sim.households.len(), villages, sim.structures.len(),
            if state.paused { "   일시정지" } else { "" });
    }
    if let Ok(mut t) = cursor.single_mut() {
        if t.0 != state.cursor_tile { t.0 = state.cursor_tile.clone(); }
    }
    if let Ok(mut t) = world.single_mut() {
        let size = sim.terrain.as_deref().map(|m| m.width).unwrap_or(0);
        let s = format!("월드: {} {}", template_name(state.template), size);
        if t.0 != s { t.0 = s; }
    }
    if let Ok(mut t) = map.single_mut() {
        let s = format!("지도: {}", map_mode_name(state.map_mode));
        if t.0 != s { t.0 = s; }
    }
}

#[derive(Component)] pub(crate) struct VillageBanner(u64);
#[derive(Component)] pub(crate) struct BannerText;

/// WorldBox-style name plates: one per living village, pinned above its centre
/// at a constant screen size. Clicking a plate opens that village's card.
pub fn sync_banners(
    mut commands: Commands,
    mut state: ResMut<ViewerState>,
    camera: Query<(&Camera, &GlobalTransform), With<crate::WorldCamera>>,
    mut banners: Query<(Entity, &VillageBanner, &mut Node, &Interaction, &Children), Without<BannerText>>,
    mut texts: Query<&mut Text, With<BannerText>>,
) {
    let Ok((cam, global)) = camera.single() else { return; };
    let terrain = state.sim.terrain.clone();
    let t = terrain.as_deref();
    let mut live: std::collections::HashMap<u64, (Vec2, String)> = state.sim.settlements.iter()
        .filter(|v| !v.members.is_empty())
        .map(|v| (v.id, (crate::terrain_view::to_view(t, v.center), format!("{} · {}명", crate::names::village(&v.lexicon), v.members.len()))))
        .collect();
    let mut clicked = None;
    for (e, banner, mut node, interaction, children) in &mut banners {
        let Some((view, label)) = live.remove(&banner.0) else { commands.entity(e).despawn(); continue; };
        match cam.world_to_viewport(global, view.extend(0.0)) {
            Ok(p) => { node.left = px(p.x - 60.0); node.top = px(p.y - 64.0); node.display = Display::Flex; }
            Err(_) => node.display = Display::None,
        }
        if let Some(&c) = children.first() {
            if let Ok(mut text) = texts.get_mut(c) { if text.0 != label { text.0 = label; } }
        }
        if *interaction == Interaction::Pressed { clicked = Some(banner.0); }
    }
    for (id, (_, label)) in live {
        commands.spawn((
            Node { position_type: PositionType::Absolute, width: px(120), padding: UiRect::axes(px(6), px(3)),
                justify_content: JustifyContent::Center, border: UiRect::all(px(1)), border_radius: BorderRadius::all(px(4)),
                display: Display::None, ..default() },
            BackgroundColor(Color::srgba(0.07, 0.08, 0.10, 0.82)), BorderColor::all(GOLD), Interaction::default(), VillageBanner(id),
        )).with_child((label_text(&label), BannerText));
    }
    if let Some(id) = clicked { state.selected = Some(crate::Selected::Village(id)); }
}

fn label_text(text: &str) -> impl Bundle { label(text, FONT_PX, TEXT) }
