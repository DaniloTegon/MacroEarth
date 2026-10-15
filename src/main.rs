#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui;
#[cfg(target_os = "windows")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use resvg::usvg::TreeParsing;
use std::sync::{Arc, Mutex};

#[cfg(target_os = "windows")]
use windows_sys::{
    Win32::Foundation::GetLastError,
    Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
        KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC, MOD_NOREPEAT, MapVirtualKeyW,
        RegisterHotKey, SendInput, UnregisterHotKey, VK_A, VK_D, VK_DOWN, VK_LCONTROL, VK_LEFT,
        VK_RIGHT, VK_S, VK_UP, VK_W,
    },
    Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetMessageW, GetWindowLongPtrW, KillTimer, LWA_ALPHA, MSG, PM_NOREMOVE,
        PeekMessageW, SetLayeredWindowAttributes, SetTimer, SetWindowLongPtrW, WM_HOTKEY, WM_TIMER,
        WS_EX_LAYERED,
    },
};

#[derive(Clone, Copy, Debug)]
struct Stratagem {
    category: &'static str,
    department: &'static str,
    name: &'static str,
    short_name: &'static str,
    icon: &'static str,
    asset: &'static [u8],
    sequence: &'static [&'static str],
}

#[cfg(any())]
#[allow(dead_code)]
const LEGACY_STRATAGEMS: &[Stratagem] = &[
    Stratagem {
        category: "Ofensiva",
        name: "Orbital Precision Strike",
        short_name: "OPS",
        icon: "◎",
        asset: include_bytes!("../assets/stratagems/orbital_precision.svg"),
        sequence: &["D", "D", "W"],
    },
    Stratagem {
        category: "Ofensiva",
        name: "Orbital Railcannon Strike",
        short_name: "RAIL",
        icon: "⌖",
        asset: include_bytes!("../assets/stratagems/orbital_railcannon.svg"),
        sequence: &["D", "S", "W", "W", "D"],
    },
    Stratagem {
        category: "Ofensiva",
        name: "Orbital Gas Strike",
        short_name: "GAS",
        icon: "☣",
        asset: include_bytes!("../assets/stratagems/orbital_gas.svg"),
        sequence: &["D", "D", "S", "D"],
    },
    Stratagem {
        category: "Ofensiva",
        name: "Orbital Smoke Strike",
        short_name: "FUMACA",
        icon: "☁",
        asset: include_bytes!("../assets/stratagems/orbital_smoke.svg"),
        sequence: &["D", "D", "S", "W"],
    },
    Stratagem {
        category: "Ofensiva",
        name: "Eagle Strafing Run",
        short_name: "STRAFE",
        icon: "✈",
        asset: include_bytes!("../assets/stratagems/eagle_strafing.svg"),
        sequence: &["W", "D", "D"],
    },
    Stratagem {
        category: "Ofensiva",
        name: "Eagle Cluster Bomb",
        short_name: "CLUSTER",
        icon: "✹",
        asset: include_bytes!("../assets/stratagems/eagle_cluster.svg"),
        sequence: &["W", "D", "S", "S", "D"],
    },
    Stratagem {
        category: "Ofensiva",
        name: "Eagle Napalm Airstrike",
        short_name: "NAPALM",
        icon: "♨",
        asset: include_bytes!("../assets/stratagems/eagle_napalm.svg"),
        sequence: &["W", "D", "S", "W"],
    },
    Stratagem {
        category: "Ofensiva",
        name: "Orbital Airburst Strike",
        short_name: "AIRBURST",
        icon: "⁙",
        asset: include_bytes!("../assets/stratagems/orbital_airburst.svg"),
        sequence: &["D", "D", "D"],
    },
    Stratagem {
        category: "Ofensiva",
        name: "Eagle Smoke Strike",
        short_name: "EAGLE SMOKE",
        icon: "☄",
        asset: include_bytes!("../assets/stratagems/eagle_smoke.svg"),
        sequence: &["W", "D", "W", "S"],
    },
    Stratagem {
        category: "Defensiva",
        name: "Machine Gun Sentry",
        short_name: "MG SENTRY",
        icon: "▦",
        asset: include_bytes!("../assets/stratagems/machine_gun_sentry.svg"),
        sequence: &["W", "D", "D", "W"],
    },
    Stratagem {
        category: "Defensiva",
        name: "Mortar Sentry",
        short_name: "MORTAR",
        icon: "▧",
        asset: include_bytes!("../assets/stratagems/mortar_sentry.svg"),
        sequence: &["W", "D", "D", "S"],
    },
    Stratagem {
        category: "Defensiva",
        name: "Shield Generator Relay",
        short_name: "SHIELD",
        icon: "⬡",
        asset: include_bytes!("../assets/stratagems/shield_relay.svg"),
        sequence: &["S", "A", "D", "A", "D"],
    },
    Stratagem {
        category: "Suprimentos",
        name: "B-1 Supply Pack",
        short_name: "SUPPLY",
        icon: "▣",
        asset: include_bytes!("../assets/stratagems/supply_pack.svg"),
        sequence: &["S", "A", "S", "W", "W", "S"],
    },
    Stratagem {
        category: "Suprimentos",
        name: "LIFT-850 Jump Pack",
        short_name: "JUMP",
        icon: "↟",
        asset: include_bytes!("../assets/stratagems/jump_pack.svg"),
        sequence: &["S", "W", "W", "S", "W"],
    },
    Stratagem {
        category: "Suprimentos",
        name: "EXO-45 Patriot Exosuit",
        short_name: "PATRIOT",
        icon: "▰",
        asset: include_bytes!("../assets/stratagems/patriot_exosuit.svg"),
        sequence: &["S", "D", "W", "A", "S", "S"],
    },
    Stratagem {
        category: "Missão",
        name: "Resupply",
        short_name: "RESUPPLY",
        icon: "＋",
        asset: include_bytes!("../assets/stratagems/resupply.svg"),
        sequence: &["S", "S", "W", "D"],
    },
    Stratagem {
        category: "Missão",
        name: "Reinforce",
        short_name: "REINFORCE",
        icon: "⬆",
        asset: include_bytes!("../assets/stratagems/reinforce.svg"),
        sequence: &["W", "S", "D", "A", "W"],
    },
];

include!("stratagems.rs");

#[derive(Clone, Copy, Default)]
struct MacroCell {
    stratagem_index: Option<usize>,
    shortcut: Option<egui::Key>,
    listening: bool,
}

impl MacroCell {
    fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Snapshot of the bindings read by the background hotkey thread.
#[derive(Clone, Default, PartialEq)]
struct HotkeyConfig {
    bindings: Vec<(u16, &'static [&'static str])>,
    use_arrow_keys: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct SavedMacroCell {
    stratagem_index: Option<usize>,
    shortcut: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct SavedAppConfig {
    rows: usize,
    columns: usize,
    cells: Vec<Vec<SavedMacroCell>>,
    use_arrow_keys: bool,
    enabled_stratagems: Vec<bool>,
    show_disabled_stratagems: bool,
}

impl SavedAppConfig {
    fn path() -> Option<std::path::PathBuf> {
        std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("LOCALAPPDATA"))
            .map(std::path::PathBuf::from)
            .map(|directory| directory.join("MacroEarth").join("settings.json"))
    }

    fn load() -> Option<Self> {
        let contents = std::fs::read(Self::path()?).ok()?;
        serde_json::from_slice(&contents).ok()
    }
}

struct CellClicks {
    tile: bool,
    tile_drag_started: bool,
    shortcut: bool,
    shortcut_secondary: bool,
    middle: bool,
}

struct CellOutput {
    rect: egui::Rect,
    clicked: bool,
    drag_started: bool,
}

struct StratagemMacrosApp {
    hotkeys: Arc<Mutex<HotkeyConfig>>,
    last_saved_settings: Option<SavedAppConfig>,
    drag_source: Option<(usize, usize)>,
    use_arrow_keys: bool,
    rows: usize,
    columns: usize,
    cells: Vec<Vec<MacroCell>>,
    enabled_stratagems: Vec<bool>,
    show_disabled_stratagems: bool,
    show_settings_window: bool,
    show_selector: Option<(usize, usize, egui::Pos2)>,
    show_selector_just_opened: bool,
}

impl Default for StratagemMacrosApp {
    fn default() -> Self {
        let saved = SavedAppConfig::load();
        let rows = saved
            .as_ref()
            .map_or(4, |settings| settings.rows)
            .clamp(1, Self::MAX_GRID_DIMENSION);
        let columns = saved
            .as_ref()
            .map_or(4, |settings| settings.columns)
            .clamp(1, Self::MAX_GRID_DIMENSION);
        let mut cells = vec![vec![MacroCell::default(); columns]; rows];
        if let Some(settings) = &saved {
            for (row_index, row) in settings.cells.iter().take(rows).enumerate() {
                for (column_index, saved_cell) in row.iter().take(columns).enumerate() {
                    let cell = &mut cells[row_index][column_index];
                    cell.stratagem_index = saved_cell
                        .stratagem_index
                        .filter(|index| *index < STRATAGEMS.len());
                    cell.shortcut = saved_cell
                        .shortcut
                        .as_deref()
                        .and_then(egui::Key::from_name);
                }
            }
        }
        let enabled_stratagems = saved
            .as_ref()
            .filter(|settings| settings.enabled_stratagems.len() == STRATAGEMS.len())
            .map(|settings| settings.enabled_stratagems.clone())
            .unwrap_or_else(|| vec![true; STRATAGEMS.len()]);
        Self {
            hotkeys: Default::default(),
            last_saved_settings: None,
            drag_source: None,
            use_arrow_keys: saved
                .as_ref()
                .is_some_and(|settings| settings.use_arrow_keys),
            rows,
            columns,
            cells,
            enabled_stratagems,
            show_disabled_stratagems: saved
                .as_ref()
                .is_some_and(|settings| settings.show_disabled_stratagems),
            show_settings_window: false,
            show_selector: None,
            show_selector_just_opened: false,
        }
    }
}

impl StratagemMacrosApp {
    const MAX_GRID_DIMENSION: usize = 10;
    const GRID_CONTROL_INDENT: f32 = 40.0;
    const STRATAGEM_WINDOW_SIZE: [f32; 2] = [456.0, 450.0];

    fn grid_window_size(rows: usize, columns: usize) -> egui::Vec2 {
        let rows = rows.clamp(1, Self::MAX_GRID_DIMENSION) as f32;
        let columns = columns.clamp(1, Self::MAX_GRID_DIMENSION) as f32;
        let width = 78.0 + columns * 74.0;
        let height = 172.0 + rows * 74.0;
        egui::vec2(width, height)
    }

    fn adjust_grid_dimension(current: usize, delta: isize) -> usize {
        (current as isize + delta).clamp(1, Self::MAX_GRID_DIMENSION as isize) as usize
    }

    fn persist_settings_if_changed(&mut self) {
        let settings = SavedAppConfig {
            rows: self.rows,
            columns: self.columns,
            cells: self
                .cells
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|cell| SavedMacroCell {
                            stratagem_index: cell.stratagem_index,
                            shortcut: cell.shortcut.map(|key| key.name().to_owned()),
                        })
                        .collect()
                })
                .collect(),
            use_arrow_keys: self.use_arrow_keys,
            enabled_stratagems: self.enabled_stratagems.clone(),
            show_disabled_stratagems: self.show_disabled_stratagems,
        };
        if self.last_saved_settings.as_ref() == Some(&settings) {
            return;
        }

        let Some(path) = SavedAppConfig::path() else {
            return;
        };
        let Some(parent) = path.parent() else {
            return;
        };
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
        if let Ok(contents) = serde_json::to_vec_pretty(&settings) {
            if std::fs::write(path, contents).is_ok() {
                self.last_saved_settings = Some(settings);
            }
        }
    }

    fn stratagem_icon(stratagem: &Stratagem, size: f32) -> egui::Image<'static> {
        egui::Image::from_bytes(format!("bytes://{}", stratagem.icon), stratagem.asset)
            .fit_to_exact_size(egui::vec2(size, size))
            .alt_text(stratagem.name)
    }

    fn category_color(category: &str) -> egui::Color32 {
        match category {
            "Offensive" => egui::Color32::from_rgb(188, 53, 43),
            "Defense" => egui::Color32::from_rgb(76, 131, 86),
            "Supply" => egui::Color32::from_rgb(54, 119, 148),
            _ => egui::Color32::from_rgb(211, 167, 37),
        }
    }

    fn resize_grid(&mut self) {
        self.cells.resize(self.rows, Vec::new());
        for row in &mut self.cells {
            row.resize(self.columns, MacroCell::default());
        }
    }

    fn capture_shortcut(&mut self, ctx: &egui::Context) -> bool {
        let pressed_key = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } => Some(*key),
                _ => None,
            })
        });

        if let Some(key) = pressed_key {
            for row in &mut self.cells {
                for cell in row {
                    if cell.listening {
                        if key == egui::Key::Escape {
                            cell.listening = false;
                        } else {
                            cell.shortcut = Some(key);
                            cell.listening = false;
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    fn update_shortcut_focus(
        &mut self,
        any_click: bool,
        click_position: Option<egui::Pos2>,
        focused_shortcut_rect: Option<egui::Rect>,
        clicked_shortcut: Option<(usize, usize)>,
        viewport_focused: Option<bool>,
    ) {
        let clicked_outside_shortcut = any_click
            && clicked_shortcut.is_none()
            && !focused_shortcut_rect
                .is_some_and(|rect| click_position.is_some_and(|position| rect.contains(position)));
        if viewport_focused == Some(false) || clicked_outside_shortcut {
            for cell in self.cells.iter_mut().flatten() {
                cell.listening = false;
            }
        } else if let Some((row_index, column_index)) = clicked_shortcut {
            for cell in self.cells.iter_mut().flatten() {
                cell.listening = false;
            }
            self.cells[row_index][column_index].listening = true;
        }
    }

    fn change_grid_size(&mut self, ctx: &egui::Context, row_delta: isize, column_delta: isize) {
        self.rows = Self::adjust_grid_dimension(self.rows, row_delta);
        self.columns = Self::adjust_grid_dimension(self.columns, column_delta);
        self.resize_grid();
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Self::grid_window_size(
            self.rows,
            self.columns,
        )));
    }

    fn show_selector_window(&mut self, ctx: &egui::Context) {
        let Some((row_index, column_index, position)) = self.show_selector else {
            return;
        };

        let just_opened = self.show_selector_just_opened;
        let mut close_selector = false;
        let mut selected_stratagem = false;
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("stratagem-selector"),
            egui::ViewportBuilder::default()
                .with_title("MacroEarth | Stratagems")
                .with_inner_size(Self::STRATAGEM_WINDOW_SIZE)
                .with_resizable(false)
                .with_position(position)
                .with_always_on_top(),
            |selector_ctx, _class| {
                let viewport = selector_ctx.input(|input| input.viewport().clone());
                close_selector =
                    viewport.close_requested() || (!just_opened && viewport.focused == Some(false));

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::NONE
                            .fill(egui::Color32::from_rgb(9, 11, 12))
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                egui::Color32::from_rgb(222, 185, 54),
                            ))
                            .inner_margin(egui::Margin::symmetric(14, 8)),
                    )
                    .show(selector_ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("STRATAGEMS")
                                    .size(14.0)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("×").clicked() {
                                        close_selector = true;
                                    }
                                },
                            );
                        });
                        ui.checkbox(
                            &mut self.show_disabled_stratagems,
                            egui::RichText::new("Show unavailable stratagems").size(12.0),
                        );
                        ui.separator();

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for category in
                                    ["Common", "Objectives", "Offensive", "Supply", "Defense"]
                                {
                                    let items: Vec<_> = STRATAGEMS
                                        .iter()
                                        .enumerate()
                                        .filter(|(index, stratagem)| {
                                            stratagem.category == category
                                                && (self.enabled_stratagems[*index]
                                                    || self.show_disabled_stratagems)
                                        })
                                        .map(|(index, _)| index)
                                        .collect();

                                    if items.is_empty() {
                                        continue;
                                    }

                                    let enabled_count = items
                                        .iter()
                                        .filter(|index| self.enabled_stratagems[**index])
                                        .count();
                                    egui::CollapsingHeader::new(
                                        egui::RichText::new(format!(
                                            "{} ({enabled_count}/{})",
                                            category.to_uppercase(),
                                            items.len()
                                        ))
                                        .size(12.0)
                                        .strong()
                                        .color(egui::Color32::WHITE),
                                    )
                                    .id_salt(("selector-section", category))
                                    .default_open(true)
                                    .show_unindented(
                                        ui,
                                        |ui| {
                                            egui::Grid::new(("selector-grid", category))
                                                .num_columns(4)
                                                .spacing(egui::vec2(6.0, 6.0))
                                                .show(ui, |ui| {
                                                    for (position, index) in
                                                        items.iter().enumerate()
                                                    {
                                                        let stratagem = &STRATAGEMS[*index];
                                                        let is_enabled =
                                                            self.enabled_stratagems[*index];
                                                        let is_selected = self.cells[row_index]
                                                            [column_index]
                                                            .stratagem_index
                                                            == Some(*index);
                                                        ui.allocate_ui_with_layout(
                                                            egui::vec2(94.0, 94.0),
                                                            egui::Layout::top_down(
                                                                egui::Align::Center,
                                                            ),
                                                            |cell_ui| {
                                                                let button = egui::Button::image(
                                                                    Self::stratagem_icon(
                                                                        stratagem, 48.0,
                                                                    ),
                                                                )
                                                                .fill(if is_selected {
                                                                    Self::category_color(category)
                                                                } else {
                                                                    egui::Color32::from_rgb(
                                                                        27, 29, 30,
                                                                    )
                                                                })
                                                                .stroke(egui::Stroke::new(
                                                                    1.0_f32,
                                                                    egui::Color32::from_rgb(
                                                                        222, 185, 54,
                                                                    ),
                                                                ))
                                                                .min_size(egui::vec2(90.0, 62.0));
                                                                let response = cell_ui
                                                                    .add_enabled(is_enabled, button)
                                                                    .on_hover_text(stratagem.name);
                                                                cell_ui.add(
                                                            egui::Label::new(
                                                                egui::RichText::new(stratagem.short_name)
                                                                    .size(11.0)
                                                                    .color(egui::Color32::WHITE),
                                                            )
                                                            .wrap()
                                                            .halign(egui::Align::Center),
                                                        );
                                                                if response.clicked() {
                                                                    self.cells[row_index]
                                                                        [column_index]
                                                                        .stratagem_index =
                                                                        Some(*index);
                                                                    selected_stratagem = true;
                                                                }
                                                            },
                                                        );
                                                        if (position + 1) % 4 == 0 {
                                                            ui.end_row();
                                                        }
                                                    }
                                                    if items.len() % 4 != 0 {
                                                        ui.end_row();
                                                    }
                                                });
                                        },
                                    );
                                    ui.add_space(7.0);
                                }
                            });
                    });
            },
        );

        if close_selector || selected_stratagem {
            self.show_selector = None;
            self.show_selector_just_opened = false;
        } else {
            self.show_selector_just_opened = false;
        }
    }

    fn show_settings_window(&mut self, ctx: &egui::Context) {
        if !self.show_settings_window {
            return;
        }

        let mut close_settings = false;
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("stratagem-settings"),
            egui::ViewportBuilder::default()
                .with_title("MacroEarth | Settings")
                .with_inner_size([456.0, 450.0])
                .with_clamp_size_to_monitor_size(true)
                .with_position([360.0, 40.0])
                .with_decorations(false),
            |settings_ctx, _class| {
                settings_ctx.set_visuals(egui::Visuals::dark());
                close_settings = settings_ctx.input(|input| input.viewport().close_requested());
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::NONE
                            .fill(egui::Color32::from_rgb(8, 10, 11))
                            .inner_margin(egui::Margin::symmetric(14, 8)),
                    )
                    .show(settings_ctx, |ui| {
                        // Selectable labels would swallow the drag that moves the window.
                        ui.style_mut().interaction.selectable_labels = false;
                        // Added first so every real widget sits above it and wins the hit test.
                        let background = ui.interact(
                            settings_ctx.screen_rect(),
                            egui::Id::new("settings-drag-background"),
                            egui::Sense::drag(),
                        );
                        if background.drag_started() {
                            settings_ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                        ui.label(
                            egui::RichText::new("MacroEarth | Settings")
                                .size(15.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );
                        ui.separator();
                        self.settings_ui(ui);
                    });
            },
        );

        if close_settings {
            self.show_settings_window = false;
        }
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("STRATAGEMS")
                    .size(14.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(egui::RichText::new("Uncheck All").size(12.0))
                    .clicked()
                {
                    self.enabled_stratagems.fill(false);
                }
                if ui
                    .button(egui::RichText::new("Check All").size(12.0))
                    .clicked()
                {
                    self.enabled_stratagems.fill(true);
                }
            });
        });
        ui.add_space(4.0);

        let mut departments = Vec::new();
        for stratagem in STRATAGEMS {
            if !departments.contains(&stratagem.department) {
                departments.push(stratagem.department);
            }
        }

        ui.style_mut().spacing.scroll = egui::style::ScrollStyle::floating();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .drag_to_scroll(false)
            .show(ui, |ui| {
                let origin_x = ui.max_rect().min.x;
                for department in departments {
                    let entries: Vec<_> = STRATAGEMS
                        .iter()
                        .enumerate()
                        .filter(|(_, stratagem)| stratagem.department == department)
                        .collect();
                    let enabled_count = entries
                        .iter()
                        .filter(|(index, _)| self.enabled_stratagems[*index])
                        .count();

                    // A tile border sticks out 1pt on the left and the parent cursor follows it.
                    let mut area = ui.available_rect_before_wrap();
                    area.min.x = origin_x;
                    let mut section_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .id_salt(("settings-section-ui", department))
                            .max_rect(area),
                    );
                    egui::CollapsingHeader::new(
                        egui::RichText::new(format!(
                            "{} ({enabled_count}/{})",
                            department.to_uppercase(),
                            entries.len()
                        ))
                        .size(13.0)
                        .strong()
                        .color(egui::Color32::WHITE),
                    )
                    .id_salt(("settings-section", department))
                    .default_open(true)
                    .show_unindented(&mut section_ui, |ui| {
                        egui::Grid::new(("settings-grid", department))
                            .num_columns(4)
                            .spacing(egui::vec2(6.0, 6.0))
                            .show(ui, |ui| {
                                for (position, (index, stratagem)) in entries.iter().enumerate() {
                                    let enabled = self.enabled_stratagems[*index];
                                    egui::Frame::NONE
                                        .fill(egui::Color32::from_rgb(25, 27, 28))
                                        .stroke(egui::Stroke::new(
                                            1.0_f32,
                                            if enabled {
                                                egui::Color32::from_rgb(222, 185, 54)
                                            } else {
                                                egui::Color32::from_rgb(82, 82, 78)
                                            },
                                        ))
                                        .inner_margin(egui::Margin::same(3))
                                        .show(ui, |ui| {
                                            ui.allocate_ui_with_layout(
                                                egui::vec2(94.0, 94.0),
                                                egui::Layout::top_down(egui::Align::Center),
                                                |tile| {
                                                    let tile_rect =
                                                        tile.available_rect_before_wrap();
                                                    tile.spacing_mut().icon_width = 14.0;
                                                    let icon_rect = egui::Rect::from_min_size(
                                                        egui::pos2(
                                                            tile_rect.center().x - 24.0,
                                                            tile_rect.top() + 3.0,
                                                        ),
                                                        egui::vec2(48.0, 48.0),
                                                    );
                                                    tile.put(
                                                        icon_rect,
                                                        Self::stratagem_icon(stratagem, 48.0),
                                                    );
                                                    let checkbox_rect = egui::Rect::from_min_size(
                                                        tile_rect.left_top(),
                                                        egui::vec2(16.0, 16.0),
                                                    );
                                                    tile.painter().rect_filled(
                                                        checkbox_rect,
                                                        1.0,
                                                        egui::Color32::from_rgb(18, 20, 21),
                                                    );
                                                    tile.put(
                                                checkbox_rect,
                                                egui::Checkbox::new(
                                                    &mut self.enabled_stratagems[*index],
                                                    "",
                                                ),
                                            )
                                            .on_hover_text("Include in stratagem selection");
                                                    let name_rect = egui::Rect::from_min_max(
                                                        egui::pos2(
                                                            tile_rect.left(),
                                                            icon_rect.bottom() + 2.0,
                                                        ),
                                                        tile_rect.right_bottom(),
                                                    );
                                                    tile.put(
                                                        name_rect,
                                                        egui::Label::new(
                                                            egui::RichText::new(stratagem.name)
                                                                .size(11.0),
                                                        )
                                                        .wrap()
                                                        .halign(egui::Align::Center),
                                                    );
                                                },
                                            );
                                        });
                                    if (position + 1) % 4 == 0 {
                                        ui.end_row();
                                    }
                                }
                                if entries.len() % 4 != 0 {
                                    ui.end_row();
                                }
                            });
                    });
                    let used = section_ui.min_rect();
                    ui.allocate_rect(
                        egui::Rect::from_min_max(egui::pos2(origin_x, used.min.y), used.max),
                        egui::Sense::hover(),
                    );
                    ui.add_space(7.0);
                }
            });
    }

    fn macro_cell_ui(
        ui: &mut egui::Ui,
        cell: &mut MacroCell,
        row_index: usize,
        column_index: usize,
        focused_shortcut_rect: &mut Option<egui::Rect>,
        clicked_shortcut: &mut Option<(usize, usize)>,
    ) -> CellOutput {
        let selected = cell.stratagem_index.and_then(|index| STRATAGEMS.get(index));
        let (tile_rect, _) = ui.allocate_exact_size(egui::vec2(64.0, 64.0), egui::Sense::hover());
        let tile_button = selected
            .map_or_else(|| egui::Button::new("+"), |_| egui::Button::new(""))
            .fill(selected.map_or_else(
                || egui::Color32::from_rgb(20, 22, 23),
                |item| Self::category_color(item.category),
            ))
            .stroke(egui::Stroke::new(
                1.0_f32,
                egui::Color32::from_rgb(75, 68, 47),
            ))
            .sense(egui::Sense::click_and_drag());
        let shortcut_label = if cell.listening {
            "...".to_owned()
        } else {
            cell.shortcut
                .map_or_else(|| "—".to_owned(), |key| format!("{key:?}"))
        };
        let shortcut_rect = egui::Rect::from_center_size(
            egui::pos2(tile_rect.center().x, tile_rect.bottom() - 7.0),
            egui::vec2(52.0, 12.0),
        );
        if cell.listening {
            *focused_shortcut_rect = Some(shortcut_rect);
        }
        let clicks = ui
            .allocate_new_ui(egui::UiBuilder::new().max_rect(tile_rect), |tile_ui| {
                let tile_response = tile_ui
                    .put(tile_rect, tile_button)
                    .on_hover_text(selected.map_or("Select stratagem", |item| item.name));

                if let Some(item) = selected {
                    let icon_rect = egui::Rect::from_center_size(
                        egui::pos2(tile_rect.center().x, tile_rect.top() + 21.0),
                        egui::vec2(38.0, 38.0),
                    );
                    tile_ui.put(icon_rect, Self::stratagem_icon(item, 38.0));
                    Self::sequence_arrows(
                        tile_ui,
                        egui::Rect::from_min_size(
                            egui::pos2(tile_rect.left() + 4.0, tile_rect.top() + 41.0),
                            egui::vec2(tile_rect.width() - 8.0, 8.0),
                        ),
                        item.sequence,
                    );
                }

                tile_ui.painter().rect_filled(
                    shortcut_rect,
                    2.0,
                    egui::Color32::from_rgb(18, 20, 21),
                );
                let shortcut_response = tile_ui
                    .interact(
                        shortcut_rect,
                        egui::Id::new(("macro-shortcut", row_index, column_index)),
                        egui::Sense::click(),
                    )
                    .on_hover_text("Click to set a key. Right-click to clear it.");
                tile_ui.painter().text(
                    shortcut_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    shortcut_label,
                    egui::FontId::proportional(9.0),
                    egui::Color32::WHITE,
                );
                CellClicks {
                    tile: tile_response.clicked(),
                    tile_drag_started: tile_response.drag_started_by(egui::PointerButton::Primary),
                    shortcut: shortcut_response.clicked(),
                    shortcut_secondary: shortcut_response.secondary_clicked(),
                    middle: tile_response.middle_clicked() || shortcut_response.middle_clicked(),
                }
            })
            .inner;

        if clicks.middle {
            cell.clear();
        } else if clicks.shortcut_secondary {
            cell.shortcut = None;
            cell.listening = false;
        } else if clicks.shortcut {
            cell.listening = true;
            *clicked_shortcut = Some((row_index, column_index));
        }

        CellOutput {
            rect: tile_rect,
            clicked: clicks.tile,
            drag_started: clicks.tile_drag_started && cell.stratagem_index.is_some(),
        }
    }

    fn grid_control(
        ui: &mut egui::Ui,
        slot_size: egui::Vec2,
        button_size: egui::Vec2,
        label: &str,
        enabled: bool,
        tooltip: String,
    ) -> bool {
        let (slot_rect, _) = ui.allocate_exact_size(slot_size, egui::Sense::hover());
        let button_rect = egui::Rect::from_center_size(slot_rect.center(), button_size);
        let response = ui
            .put(button_rect, egui::Button::new(label).min_size(button_size))
            .on_hover_text(tooltip);
        enabled && response.clicked()
    }

    fn direction_vector(step: &str) -> Option<egui::Vec2> {
        match step {
            "W" => Some(egui::vec2(0.0, -1.0)),
            "A" => Some(egui::vec2(-1.0, 0.0)),
            "S" => Some(egui::vec2(0.0, 1.0)),
            "D" => Some(egui::vec2(1.0, 0.0)),
            _ => None,
        }
    }

    fn sequence_arrows(ui: &egui::Ui, rect: egui::Rect, sequence: &[&str]) {
        let step_width = rect.width() / sequence.len().max(1) as f32;
        let painter = ui.painter();
        for (index, step) in sequence.iter().enumerate() {
            let Some(direction) = Self::direction_vector(step) else {
                continue;
            };
            let direction = direction.normalized();
            let center = egui::pos2(
                rect.left() + step_width * (index as f32 + 0.5),
                rect.center().y,
            );
            let tip = center + direction * (rect.height() * 0.36);
            let tail = center - direction * (rect.height() * 0.36);
            let head_base = tip - direction * 2.0;
            let wing = egui::vec2(-direction.y, direction.x) * 1.4;
            let stroke = egui::Stroke::new(1.2_f32, egui::Color32::WHITE);
            painter.line_segment([tail, tip], stroke);
            painter.line_segment([tip, head_base + wing], stroke);
            painter.line_segment([tip, head_base - wing], stroke);
        }
    }

    fn swap_stratagems(&mut self, first: (usize, usize), second: (usize, usize)) {
        let first_index = self.cells[first.0][first.1].stratagem_index;
        self.cells[first.0][first.1].stratagem_index =
            self.cells[second.0][second.1].stratagem_index;
        self.cells[second.0][second.1].stratagem_index = first_index;
    }

    fn update_drag(&mut self, ctx: &egui::Context, cell_rects: &[((usize, usize), egui::Rect)]) {
        let Some(source) = self.drag_source else {
            return;
        };
        let source_item = self
            .cells
            .get(source.0)
            .and_then(|row| row.get(source.1))
            .and_then(|cell| cell.stratagem_index)
            .and_then(|index| STRATAGEMS.get(index));
        let Some(source_item) = source_item else {
            self.drag_source = None;
            return;
        };

        let (pointer, primary_down) =
            ctx.input(|input| (input.pointer.latest_pos(), input.pointer.primary_down()));
        let target = pointer.and_then(|position| {
            cell_rects
                .iter()
                .find(|(_, rect)| rect.contains(position))
                .copied()
        });

        if primary_down {
            if let Some((_, rect)) = target {
                ctx.layer_painter(egui::LayerId::new(
                    egui::Order::Tooltip,
                    egui::Id::new("stratagem-drag-target"),
                ))
                .rect_stroke(
                    rect,
                    4.0,
                    egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(222, 185, 54)),
                    egui::StrokeKind::Outside,
                );
            }
            if let Some(position) = pointer {
                egui::Area::new(egui::Id::new("stratagem-drag-ghost"))
                    .order(egui::Order::Tooltip)
                    .interactable(false)
                    .fixed_pos(position - egui::vec2(20.0, 20.0))
                    .show(ctx, |ui| ui.add(Self::stratagem_icon(source_item, 40.0)));
            }
        } else {
            self.drag_source = None;
            if let Some((destination, _)) = target {
                if destination != source {
                    self.swap_stratagems(source, destination);
                }
            }
        }
    }

    fn sync_hotkeys(&self) {
        let bindings = self
            .cells
            .iter()
            .flatten()
            .filter(|cell| !cell.listening)
            .filter_map(|cell| {
                let key = virtual_key_for_egui_key(cell.shortcut?)?;
                let stratagem = STRATAGEMS.get(cell.stratagem_index?)?;
                Some((key, stratagem.sequence))
            })
            .flat_map(|(key, sequence)| {
                // egui has no numpad variants, so digits also bind the numpad VK.
                let numpad = (0x30..=0x39).contains(&key).then(|| (key + 0x30, sequence));
                std::iter::once((key, sequence)).chain(numpad)
            })
            .collect();
        let config = HotkeyConfig {
            bindings,
            use_arrow_keys: self.use_arrow_keys,
        };
        if let Ok(mut current) = self.hotkeys.lock() {
            if *current != config {
                #[cfg(target_os = "windows")]
                {
                    let keys = config
                        .bindings
                        .iter()
                        .map(|(vk, _)| format!("0x{vk:02X}"))
                        .collect::<Vec<_>>()
                        .join(",");
                    log_input_diagnostic(&format!(
                        "Hotkey map synced: {} bindings [{keys}]; arrow_keys={}",
                        config.bindings.len(),
                        config.use_arrow_keys
                    ));
                }
                *current = config;
            }
        }
    }
}

fn virtual_key_for_egui_key(key: egui::Key) -> Option<u16> {
    let name = key.name();
    let bytes = name.as_bytes();
    if bytes.len() == 1 && (bytes[0].is_ascii_uppercase() || bytes[0].is_ascii_digit()) {
        return Some(u16::from(bytes[0]));
    }
    if let Some(number) = name.strip_prefix('F').and_then(|n| n.parse::<u16>().ok()) {
        if (1..=24).contains(&number) {
            return Some(0x6F + number);
        }
    }

    match key {
        egui::Key::Space => Some(0x20),
        egui::Key::Enter => Some(0x0D),
        egui::Key::Tab => Some(0x09),
        egui::Key::Backspace => Some(0x08),
        egui::Key::Insert => Some(0x2D),
        egui::Key::Delete => Some(0x2E),
        egui::Key::Home => Some(0x24),
        egui::Key::End => Some(0x23),
        egui::Key::PageUp => Some(0x21),
        egui::Key::PageDown => Some(0x22),
        egui::Key::ArrowLeft => Some(0x25),
        egui::Key::ArrowUp => Some(0x26),
        egui::Key::ArrowRight => Some(0x27),
        egui::Key::ArrowDown => Some(0x28),
        egui::Key::Colon | egui::Key::Semicolon => Some(0xBA),
        egui::Key::Plus | egui::Key::Equals => Some(0xBB),
        egui::Key::Comma => Some(0xBC),
        egui::Key::Minus => Some(0xBD),
        egui::Key::Period => Some(0xBE),
        egui::Key::Slash | egui::Key::Questionmark => Some(0xBF),
        egui::Key::Backtick => Some(0xC0),
        egui::Key::OpenBracket => Some(0xDB),
        egui::Key::Backslash | egui::Key::Pipe => Some(0xDC),
        egui::Key::CloseBracket => Some(0xDD),
        egui::Key::Quote => Some(0xDE),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn log_input_diagnostic(message: &str) {
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(std::env::temp_dir().join("macroearth-input.log"))
    {
        let _ = writeln!(file, "{message}");
    }
}

#[cfg(target_os = "windows")]
unsafe fn press_key(vk: u16, down: bool) -> bool {
    let scan_code = unsafe { MapVirtualKeyW(u32::from(vk), MAPVK_VK_TO_VSC) } as u16;
    let mut flags = KEYEVENTF_SCANCODE;
    if matches!(vk, VK_UP | VK_DOWN | VK_LEFT | VK_RIGHT) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if !down {
        flags |= KEYEVENTF_KEYUP;
    }

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: scan_code,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let sent = unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) };
    if sent == 1 {
        true
    } else {
        log_input_diagnostic(&format!(
            "SendInput rejected vk=0x{vk:02X} down={down} last_error={}",
            unsafe { GetLastError() }
        ));
        false
    }
}

#[cfg(target_os = "windows")]
fn run_macro(sequence: &[&str], use_arrow_keys: bool) -> bool {
    const MENU_OPEN_DELAY: std::time::Duration = std::time::Duration::from_millis(60);
    const KEY_HOLD: std::time::Duration = std::time::Duration::from_millis(25);
    const KEY_GAP: std::time::Duration = std::time::Duration::from_millis(25);

    let mut accepted = unsafe { press_key(VK_LCONTROL, true) };
    std::thread::sleep(MENU_OPEN_DELAY);
    for step in sequence {
        if let Some(vk) = virtual_key_for_step(step, use_arrow_keys) {
            accepted &= unsafe { press_key(vk, true) };
            std::thread::sleep(KEY_HOLD);
            accepted &= unsafe { press_key(vk, false) };
            std::thread::sleep(KEY_GAP);
        }
    }
    accepted &= unsafe { press_key(VK_LCONTROL, false) };
    accepted
}

#[cfg(target_os = "windows")]
fn refresh_registered_hotkeys(config: &Arc<Mutex<HotkeyConfig>>, registered: &mut Vec<u16>) {
    let mut desired = config
        .lock()
        .map(|config| {
            config
                .bindings
                .iter()
                .map(|(vk, _)| *vk)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    desired.sort_unstable();
    desired.dedup();
    if desired == *registered {
        return;
    }

    for vk in registered.drain(..) {
        unsafe { UnregisterHotKey(0, i32::from(vk)) };
    }
    for vk in desired {
        if unsafe { RegisterHotKey(0, i32::from(vk), MOD_NOREPEAT, u32::from(vk)) } != 0 {
            registered.push(vk);
            log_input_diagnostic(&format!("Registered game hotkey vk=0x{vk:02X}"));
        } else {
            log_input_diagnostic(&format!(
                "Could not register game hotkey vk=0x{vk:02X} last_error={}",
                unsafe { GetLastError() }
            ));
        }
    }
}

#[cfg(target_os = "windows")]
fn spawn_hotkey_thread(config: Arc<Mutex<HotkeyConfig>>) {
    std::thread::spawn(move || {
        let mut message: MSG = unsafe { std::mem::zeroed() };
        unsafe { PeekMessageW(&mut message, 0, 0, 0, PM_NOREMOVE) };
        let timer = unsafe { SetTimer(0, 0, 16, None) };
        if timer == 0 {
            log_input_diagnostic("Could not start global hotkey refresh timer");
            return;
        }

        log_input_diagnostic("Global RegisterHotKey worker started");
        let mut registered = Vec::new();
        let mut was_down = [false; 256];
        let mut last_fired: Option<(u16, std::time::Instant)> = None;
        refresh_registered_hotkeys(&config, &mut registered);

        loop {
            let result = unsafe { GetMessageW(&mut message, 0, 0, 0) };
            if result <= 0 {
                break;
            }

            if message.message == WM_TIMER {
                refresh_registered_hotkeys(&config, &mut registered);
                let snapshot = config
                    .lock()
                    .ok()
                    .map(|config| (config.bindings.clone(), config.use_arrow_keys));
                if let Some((bindings, use_arrow_keys)) = snapshot {
                    for (vk, sequence) in bindings {
                        let down = key_is_down(vk);
                        let previous =
                            std::mem::replace(&mut was_down[usize::from(vk & 0xFF)], down);
                        if down != previous {
                            log_input_diagnostic(&format!(
                                "Fallback key vk=0x{vk:02X} down={down}"
                            ));
                        }
                        if !down || previous {
                            continue;
                        }

                        let now = std::time::Instant::now();
                        let duplicate = last_fired.is_some_and(|(last_vk, last_time)| {
                            last_vk == vk
                                && now.duration_since(last_time)
                                    < std::time::Duration::from_millis(300)
                        });
                        if duplicate {
                            continue;
                        }
                        last_fired = Some((vk, now));
                        if run_macro(sequence, use_arrow_keys) {
                            log_input_diagnostic("Fallback macro input accepted by Windows");
                        } else {
                            log_input_diagnostic("Fallback macro input rejected by Windows");
                        }
                    }
                }
            } else if message.message == WM_HOTKEY {
                let vk = message.wParam as u16;
                let binding = config.lock().ok().and_then(|config| {
                    config
                        .bindings
                        .iter()
                        .find(|(binding_vk, _)| *binding_vk == vk)
                        .map(|(_, sequence)| (*sequence, config.use_arrow_keys))
                });
                let Some((sequence, use_arrow_keys)) = binding else {
                    continue;
                };

                log_input_diagnostic(&format!("WM_HOTKEY vk=0x{vk:02X}"));
                if run_macro(sequence, use_arrow_keys) {
                    log_input_diagnostic("Macro input accepted by Windows");
                } else {
                    log_input_diagnostic("Macro input rejected by Windows");
                }
                last_fired = Some((vk, std::time::Instant::now()));
                was_down[usize::from(vk & 0xFF)] = key_is_down(vk);
            }
        }

        for vk in registered {
            unsafe { UnregisterHotKey(0, i32::from(vk)) };
        }
        unsafe { KillTimer(0, timer) };
    });
}

#[cfg(target_os = "windows")]
fn virtual_key_for_step(step: &str, use_arrow_keys: bool) -> Option<u16> {
    match step {
        "W" => Some(if use_arrow_keys { VK_UP } else { VK_W }),
        "A" => Some(if use_arrow_keys { VK_LEFT } else { VK_A }),
        "S" => Some(if use_arrow_keys { VK_DOWN } else { VK_S }),
        "D" => Some(if use_arrow_keys { VK_RIGHT } else { VK_D }),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn key_is_down(vk: u16) -> bool {
    unsafe { GetAsyncKeyState(i32::from(vk)) as u16 & 0x8000 != 0 }
}

#[cfg(target_os = "windows")]
fn set_window_opacity(frame: &eframe::Frame, game_frontmost: bool) {
    let Ok(window_handle) = frame.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(window_handle) = window_handle.as_raw() else {
        return;
    };
    let hwnd = window_handle.hwnd.get() as windows_sys::Win32::Foundation::HWND;

    unsafe {
        let extended_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        if extended_style & WS_EX_LAYERED as isize == 0 {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, extended_style | WS_EX_LAYERED as isize);
        }
        SetLayeredWindowAttributes(hwnd, 0, if game_frontmost { 242 } else { u8::MAX }, LWA_ALPHA);
    }
}

impl eframe::App for StratagemMacrosApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        #[cfg(target_os = "windows")]
        set_window_opacity(frame, is_helldivers_frontmost());
        ctx.set_visuals(egui::Visuals::dark());

        // Click-through while the game is in front; Alt+Tab to MacroEarth makes it interactive.
        #[cfg(target_os = "windows")]
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(
                is_helldivers_frontmost(),
            ));
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        egui::TopBottomPanel::bottom("creator-signature")
            .exact_height(40.0)
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(8, 10, 11)))
            .show(ctx, |ui| {
                const CREATOR_GITHUB: &str = "https://github.com/danilotegon";
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    let avatar = ui
                        .add(
                            egui::Image::from_bytes(
                                "bytes://cybeduck-thumbnail.png",
                                include_bytes!("../assets/cybeduck_thumbnail.png"),
                            )
                            .fit_to_exact_size(egui::vec2(30.0, 30.0))
                            .alt_text("Danilo Cybeduck Tegon")
                            .sense(egui::Sense::click()),
                        )
                        .on_hover_text("Open Danilo's GitHub profile")
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if avatar.clicked() {
                        ctx.open_url(egui::OpenUrl::new_tab(CREATOR_GITHUB));
                    }
                    ui.add_space(6.0);
                    let creator = ui
                        .add(
                            egui::Label::new(
                                egui::RichText::new("Created by Danilo 'Cybeduck' Tegon")
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(190, 194, 194)),
                            )
                            .sense(egui::Sense::click()),
                        )
                        .on_hover_text("Open Danilo's GitHub profile")
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if creator.clicked() {
                        ctx.open_url(egui::OpenUrl::new_tab(CREATOR_GITHUB));
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(8, 10, 11)))
            .show(ctx, |ui| {
                // Selectable labels and the grid's drag-to-scroll would swallow the window drag.
                ui.style_mut().interaction.selectable_labels = false;
                // Added first so every real widget sits above it and wins the hit test.
                let background = ui.interact(
                    ctx.screen_rect(),
                    egui::Id::new("main-drag-background"),
                    egui::Sense::drag(),
                );
                if background.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                let item_spacing_y = ui.spacing().item_spacing.y;
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::from_bytes(
                            "bytes://super-earth-emblem.svg",
                            include_bytes!("../assets/super_earth_emblem.svg"),
                        )
                        .fit_to_exact_size(egui::vec2(22.0, 22.0))
                        .alt_text("MacroEarth emblem"),
                    );
                    ui.add_sized(
                        [ui.available_width() - 44.0, 22.0],
                        egui::Label::new(
                            egui::RichText::new("MacroEarth")
                                .size(15.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        ),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        if ui.small_button("×").on_hover_text("Close").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.small_button("−").on_hover_text("Minimize").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    });
                });

                ui.add(egui::Separator::default().spacing(0.0));
                ui.spacing_mut().item_spacing.y = item_spacing_y;
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.add_space(Self::GRID_CONTROL_INDENT);
                    if ui.button("Settings").clicked() {
                        self.show_settings_window = !self.show_settings_window;
                    }
                    ui.checkbox(&mut self.use_arrow_keys, "Use arrow keys");
                });

                let mut clicked_cell = None;
                let mut remove_row = false;
                let mut remove_column = false;
                let mut add_row = false;
                let mut add_column = false;
                let mut focused_shortcut_rect = None;
                let mut clicked_shortcut = None;
                let mut cell_rects = Vec::new();
                let mut drag_started = None;
                let column_count = self.columns;
                let row_count = self.rows;
                egui::ScrollArea::both()
                    .id_salt("macro-grid-v3")
                    .drag_to_scroll(false)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                            for column_index in 0..column_count {
                                if column_index == 0 {
                                    remove_row = Self::grid_control(
                                        ui,
                                        egui::vec2(64.0, 24.0),
                                        egui::vec2(24.0, 24.0),
                                        "−",
                                        row_count > 1,
                                        "Remove row".to_owned(),
                                    );
                                } else {
                                    ui.allocate_exact_size(
                                        egui::vec2(64.0, 24.0),
                                        egui::Sense::hover(),
                                    );
                                }
                            }
                            ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                        });

                        for row_index in 0..row_count {
                            let row = &mut self.cells[row_index];
                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                if row_index == 0 {
                                    remove_column = Self::grid_control(
                                        ui,
                                        egui::vec2(24.0, 64.0),
                                        egui::vec2(24.0, 24.0),
                                        "−",
                                        column_count > 1,
                                        "Remove column".to_owned(),
                                    );
                                } else {
                                    ui.allocate_exact_size(
                                        egui::vec2(24.0, 64.0),
                                        egui::Sense::hover(),
                                    );
                                }
                                for (col_index, cell) in row.iter_mut().enumerate() {
                                    let output = Self::macro_cell_ui(
                                        ui,
                                        cell,
                                        row_index,
                                        col_index,
                                        &mut focused_shortcut_rect,
                                        &mut clicked_shortcut,
                                    );
                                    cell_rects.push(((row_index, col_index), output.rect));
                                    if output.clicked {
                                        clicked_cell = Some((row_index, col_index, output.rect));
                                    }
                                    if output.drag_started {
                                        drag_started = Some((row_index, col_index));
                                    }
                                }
                                if row_index == 0 {
                                    add_column = Self::grid_control(
                                        ui,
                                        egui::vec2(24.0, 64.0),
                                        egui::vec2(24.0, 24.0),
                                        "+",
                                        true,
                                        "Add column".to_owned(),
                                    );
                                } else {
                                    ui.allocate_exact_size(
                                        egui::vec2(24.0, 64.0),
                                        egui::Sense::hover(),
                                    );
                                }
                            });
                        }

                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                            add_row = Self::grid_control(
                                ui,
                                egui::vec2(64.0, 24.0),
                                egui::vec2(24.0, 24.0),
                                "+",
                                true,
                                "Add row".to_owned(),
                            );
                            for _ in 1..column_count {
                                ui.allocate_exact_size(
                                    egui::vec2(64.0, 24.0),
                                    egui::Sense::hover(),
                                );
                            }
                            ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::hover());
                        });
                    });

                if drag_started.is_some() {
                    self.drag_source = drag_started;
                }
                self.update_drag(ctx, &cell_rects);

                if remove_column {
                    self.change_grid_size(ctx, 0, -1);
                } else if remove_row {
                    self.change_grid_size(ctx, -1, 0);
                } else if add_column {
                    self.change_grid_size(ctx, 0, 1);
                } else if add_row {
                    self.change_grid_size(ctx, 1, 0);
                }

                let (any_click, click_position, focused) = ctx.input(|input| {
                    (
                        input.pointer.any_click(),
                        input.pointer.interact_pos(),
                        input.viewport().focused,
                    )
                });
                self.update_shortcut_focus(
                    any_click,
                    click_position,
                    focused_shortcut_rect,
                    clicked_shortcut,
                    focused,
                );

                self.capture_shortcut(ctx);
                self.sync_hotkeys();

                if let Some((row_index, column_index, button_rect)) = clicked_cell {
                    let viewport = ctx.input(|input| input.viewport().clone());
                    let window_origin = viewport
                        .outer_rect
                        .map_or(egui::Pos2::ZERO, |rect| rect.min);
                    let monitor_size = viewport.monitor_size.unwrap_or(egui::vec2(1920.0, 1080.0));
                    let anchor_rect = button_rect.translate(window_origin.to_vec2());
                    let popup_size = egui::vec2(
                        Self::STRATAGEM_WINDOW_SIZE[0],
                        Self::STRATAGEM_WINDOW_SIZE[1],
                    );
                    let below_right = anchor_rect.right_bottom() + egui::vec2(4.0, 4.0);
                    let x = if below_right.x + popup_size.x <= monitor_size.x {
                        below_right.x
                    } else {
                        (anchor_rect.left() - popup_size.x - 4.0).max(0.0)
                    };
                    let y = if below_right.y + popup_size.y <= monitor_size.y {
                        below_right.y
                    } else {
                        (monitor_size.y - popup_size.y).max(0.0)
                    };
                    self.show_selector = Some((row_index, column_index, egui::pos2(x, y)));
                }
            });
        self.show_selector_window(ctx);
        self.show_settings_window(ctx);
        self.persist_settings_if_changed();
    }
}

#[test]
fn test_direction_vectors_match_arrow_orientation() {
    assert_eq!(
        StratagemMacrosApp::direction_vector("W"),
        Some(egui::vec2(0.0, -1.0))
    );
    assert_eq!(
        StratagemMacrosApp::direction_vector("A"),
        Some(egui::vec2(-1.0, 0.0))
    );
    assert_eq!(
        StratagemMacrosApp::direction_vector("S"),
        Some(egui::vec2(0.0, 1.0))
    );
    assert_eq!(
        StratagemMacrosApp::direction_vector("D"),
        Some(egui::vec2(1.0, 0.0))
    );
}

#[test]
fn test_direction_vectors_ignore_unknown_steps() {
    assert_eq!(StratagemMacrosApp::direction_vector("X"), None);
}

#[cfg(target_os = "windows")]
#[test]
fn test_arrow_mode_sends_arrow_virtual_keys() {
    assert_eq!(virtual_key_for_step("W", true), Some(VK_UP));
    assert_eq!(virtual_key_for_step("A", true), Some(VK_LEFT));
    assert_eq!(virtual_key_for_step("S", true), Some(VK_DOWN));
    assert_eq!(virtual_key_for_step("D", true), Some(VK_RIGHT));
    assert_eq!(virtual_key_for_step("W", false), Some(VK_W));
    assert_eq!(virtual_key_for_step("A", false), Some(VK_A));
    assert_eq!(virtual_key_for_step("S", false), Some(VK_S));
    assert_eq!(virtual_key_for_step("D", false), Some(VK_D));
}

#[test]
fn test_verified_stratagem_sequences_are_present() {
    let precision_strike = STRATAGEMS
        .iter()
        .find(|item| item.name == "Orbital Precision Strike")
        .unwrap();
    let railcannon = STRATAGEMS
        .iter()
        .find(|item| item.name == "Orbital Railcannon Strike")
        .unwrap();
    let reinforce = STRATAGEMS
        .iter()
        .find(|item| item.name == "Reinforce")
        .unwrap();

    assert_eq!(precision_strike.sequence, ["D", "D", "W"]);
    assert_eq!(railcannon.sequence, ["D", "W", "S", "S", "D"]);
    assert_eq!(reinforce.sequence, ["W", "S", "D", "A", "W"]);
}

#[test]
fn test_grid_bounds_and_window_size_are_responsive() {
    assert_eq!(StratagemMacrosApp::adjust_grid_dimension(1, -1), 1);
    assert_eq!(StratagemMacrosApp::adjust_grid_dimension(10, 1), 10);

    let smallest = StratagemMacrosApp::grid_window_size(1, 1);
    let largest = StratagemMacrosApp::grid_window_size(10, 10);
    assert!(largest.x > smallest.x);
    assert!(largest.y > smallest.y);
    assert_eq!(smallest, egui::vec2(152.0, 246.0));
    assert_eq!(largest, egui::vec2(818.0, 912.0));
}

#[test]
fn test_shortcut_focus_moves_exclusively_and_clears_on_outside_click() {
    let mut app = StratagemMacrosApp::default();
    app.cells[0][0].listening = true;
    app.cells[0][1].listening = true;
    let shortcut_rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(52.0, 12.0));

    app.update_shortcut_focus(
        true,
        Some(egui::pos2(100.0, 100.0)),
        Some(shortcut_rect),
        Some((0, 1)),
        Some(true),
    );
    assert!(!app.cells[0][0].listening);
    assert!(app.cells[0][1].listening);

    app.update_shortcut_focus(
        true,
        Some(egui::pos2(100.0, 100.0)),
        Some(shortcut_rect),
        None,
        Some(true),
    );
    assert!(app.cells.iter().flatten().all(|cell| !cell.listening));
}

#[test]
fn test_grid_starts_with_four_rows_and_columns() {
    let app = StratagemMacrosApp::default();
    assert_eq!(app.rows, 4);
    assert_eq!(app.columns, 4);
    assert_eq!(app.cells.len(), 4);
    assert!(app.cells.iter().all(|row| row.len() == 4));
    assert_eq!(
        StratagemMacrosApp::grid_window_size(app.rows, app.columns),
        egui::vec2(374.0, 468.0)
    );
}

#[test]
fn test_catalog_has_all_114_stratagems_and_missions_first() {
    assert_eq!(STRATAGEMS.len(), 114);
    assert_eq!(STRATAGEMS[0].category, "Common");
    assert!(STRATAGEMS.iter().any(|item| item.category == "Objectives"));
    assert!(
        STRATAGEMS
            .iter()
            .all(|item| !item.sequence.is_empty() && !item.department.is_empty())
    );
}

#[test]
fn test_swap_exchanges_stratagems_but_keeps_shortcuts() {
    let mut app = StratagemMacrosApp::default();
    app.cells[0][0].stratagem_index = Some(1);
    app.cells[0][0].shortcut = Some(egui::Key::Q);
    app.cells[1][1].stratagem_index = Some(2);
    app.cells[1][1].shortcut = Some(egui::Key::E);

    app.swap_stratagems((0, 0), (1, 1));

    assert_eq!(app.cells[0][0].stratagem_index, Some(2));
    assert_eq!(app.cells[1][1].stratagem_index, Some(1));
    assert_eq!(app.cells[0][0].shortcut, Some(egui::Key::Q));
    assert_eq!(app.cells[1][1].shortcut, Some(egui::Key::E));
}

#[test]
fn test_clear_removes_stratagem_and_shortcut() {
    let mut cell = MacroCell {
        stratagem_index: Some(3),
        shortcut: Some(egui::Key::X),
        listening: true,
    };
    cell.clear();
    assert!(cell.stratagem_index.is_none() && cell.shortcut.is_none() && !cell.listening);
}

#[test]
fn test_saved_config_roundtrip_preserves_hotkeys_and_grid() {
    let config = SavedAppConfig {
        rows: 2,
        columns: 3,
        cells: vec![
            vec![
                SavedMacroCell {
                    stratagem_index: Some(7),
                    shortcut: Some("F5".to_owned()),
                },
                SavedMacroCell {
                    stratagem_index: None,
                    shortcut: Some("Q".to_owned()),
                },
                SavedMacroCell {
                    stratagem_index: None,
                    shortcut: None,
                },
            ],
            vec![SavedMacroCell {
                stratagem_index: Some(2),
                shortcut: Some("0".to_owned()),
            }],
        ],
        use_arrow_keys: true,
        enabled_stratagems: vec![true; STRATAGEMS.len()],
        show_disabled_stratagems: true,
    };

    let serialized = serde_json::to_vec(&config).unwrap();
    let restored: SavedAppConfig = serde_json::from_slice(&serialized).unwrap();
    assert_eq!(restored, config);
}

#[test]
fn test_egui_keys_map_to_windows_virtual_keys() {
    assert_eq!(virtual_key_for_egui_key(egui::Key::A), Some(0x41));
    assert_eq!(virtual_key_for_egui_key(egui::Key::Num0), Some(0x30));
    assert_eq!(virtual_key_for_egui_key(egui::Key::F1), Some(0x70));
    assert_eq!(virtual_key_for_egui_key(egui::Key::F12), Some(0x7B));
    assert_eq!(virtual_key_for_egui_key(egui::Key::Space), Some(0x20));
    assert_eq!(virtual_key_for_egui_key(egui::Key::ArrowUp), Some(0x26));
}

#[test]
fn test_super_earth_window_icon_renders() {
    let icon = super_earth_icon();
    assert_eq!(icon.width, 128);
    assert_eq!(icon.height, 128);
    assert_eq!(icon.rgba.len(), (icon.width * icon.height * 4) as usize);
}

/// Barlow Semi Condensed (OFL) stands in for the proprietary FS Sinclair used by Helldivers 2.
fn install_game_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "game-ui".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/BarlowSemiCondensed-Medium.ttf"
        ))),
    );
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.insert(0, "game-ui".to_owned());
    }
    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result<()> {
    let initial_size = StratagemMacrosApp::grid_window_size(4, 4);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(initial_size)
            .with_max_inner_size([1000.0, 1250.0])
            .with_clamp_size_to_monitor_size(true)
            .with_position([55.0, 55.0])
            .with_decorations(false)
            .with_icon(std::sync::Arc::new(super_earth_icon()))
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "MacroEarth",
        options,
        Box::new(|creation_context| {
            install_game_font(&creation_context.egui_ctx);
            let app = StratagemMacrosApp::default();
            #[cfg(target_os = "windows")]
            spawn_hotkey_thread(app.hotkeys.clone());
            Ok(Box::new(app))
        }),
    )
}

fn super_earth_icon() -> egui::IconData {
    let parsed_tree = resvg::usvg::Tree::from_data(
        include_bytes!("../assets/super_earth_emblem.svg"),
        &resvg::usvg::Options::default(),
    )
    .expect("Super Earth emblem SVG must be valid");
    let tree = resvg::Tree::from_usvg(&parsed_tree);
    let size = tree.size.to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .expect("Super Earth emblem dimensions must be valid");
    tree.render(resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

    egui::IconData {
        rgba: pixmap.data().to_vec(),
        width: size.width(),
        height: size.height(),
    }
}
