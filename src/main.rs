#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui;
#[cfg(target_os = "windows")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use resvg::usvg::TreeParsing;

#[cfg(target_os = "windows")]
use windows_sys::{
    Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
    },
    Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_A, VK_CONTROL, VK_D,
        VK_DOWN, VK_LEFT, VK_RIGHT, VK_S, VK_UP, VK_W,
    },
    Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetForegroundWindow, GetWindowLongPtrW, GetWindowThreadProcessId, LWA_ALPHA,
        SetLayeredWindowAttributes, SetWindowLongPtrW, WS_EX_LAYERED,
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

struct StratagemMacrosApp {
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
        let rows = 4;
        let columns = 4;
        Self {
            use_arrow_keys: false,
            rows,
            columns,
            cells: vec![vec![MacroCell::default(); columns]; rows],
            enabled_stratagems: vec![true; STRATAGEMS.len()],
            show_disabled_stratagems: false,
            show_settings_window: false,
            show_selector: None,
            show_selector_just_opened: false,
        }
    }
}

impl StratagemMacrosApp {
    const MAX_GRID_DIMENSION: usize = 10;
    const GRID_CONTROL_INDENT: f32 = 40.0;

    fn grid_window_size(rows: usize, columns: usize) -> egui::Vec2 {
        let rows = rows.clamp(1, Self::MAX_GRID_DIMENSION) as f32;
        let columns = columns.clamp(1, Self::MAX_GRID_DIMENSION) as f32;
        let width = 78.0 + columns * 74.0;
        let height = 194.0 + rows * 74.0;
        egui::vec2(width, height)
    }

    fn adjust_grid_dimension(current: usize, delta: isize) -> usize {
        (current as isize + delta).clamp(1, Self::MAX_GRID_DIMENSION as isize) as usize
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
                .with_inner_size([320.0, 400.0])
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
                            .inner_margin(egui::Margin::same(6)),
                    )
                    .show(selector_ctx, |ui| {
                        ui.set_min_width(320.0);
                        ui.set_max_width(320.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("STRATAGEMS")
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
                            "Show unavailable stratagems",
                        );
                        ui.separator();

                        egui::ScrollArea::vertical()
                            .max_height(330.0)
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

                                    ui.label(
                                        egui::RichText::new(category.to_uppercase())
                                            .strong()
                                            .color(egui::Color32::WHITE),
                                    );
                                    egui::Grid::new(("selector-grid", category))
                                        .num_columns(4)
                                        .spacing(egui::vec2(4.0, 4.0))
                                        .show(ui, |ui| {
                                            for (position, index) in items.iter().enumerate() {
                                                let stratagem = &STRATAGEMS[*index];
                                                let is_enabled = self.enabled_stratagems[*index];
                                                let is_selected = self.cells[row_index]
                                                    [column_index]
                                                    .stratagem_index
                                                    == Some(*index);
                                                ui.allocate_ui_with_layout(
                                                    egui::vec2(72.0, 68.0),
                                                    egui::Layout::top_down(egui::Align::Center),
                                                    |cell_ui| {
                                                        let button = egui::Button::image(
                                                            Self::stratagem_icon(stratagem, 34.0),
                                                        )
                                                        .fill(if is_selected {
                                                            Self::category_color(category)
                                                        } else {
                                                            egui::Color32::from_rgb(27, 29, 30)
                                                        })
                                                        .stroke(egui::Stroke::new(
                                                            1.0_f32,
                                                            egui::Color32::from_rgb(222, 185, 54),
                                                        ))
                                                        .min_size(egui::vec2(54.0, 48.0));
                                                        let response = cell_ui
                                                            .add_enabled(is_enabled, button)
                                                            .on_hover_text(stratagem.name);
                                                        cell_ui.label(
                                                            egui::RichText::new(
                                                                stratagem.short_name,
                                                            )
                                                            .size(7.0)
                                                            .color(egui::Color32::WHITE),
                                                        );
                                                        if response.clicked() {
                                                            self.cells[row_index][column_index]
                                                                .stratagem_index = Some(*index);
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
                                    ui.add_space(5.0);
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
                .with_inner_size([520.0, 820.0])
                .with_position([360.0, 70.0])
                .with_decorations(false),
            |settings_ctx, _class| {
                settings_ctx.set_visuals(egui::Visuals::dark());
                close_settings = settings_ctx.input(|input| input.viewport().close_requested());
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(8, 10, 11)))
                    .show(settings_ctx, |ui| {
                        let title = ui.add(
                            egui::Label::new(
                                egui::RichText::new("MacroEarth | Settings")
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            )
                            .sense(egui::Sense::drag()),
                        );
                        if title.drag_started() {
                            settings_ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
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
                    .strong()
                    .color(egui::Color32::WHITE),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Uncheck All").clicked() {
                    self.enabled_stratagems.fill(false);
                }
                if ui.small_button("Check All").clicked() {
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

        egui::ScrollArea::both().show(ui, |ui| {
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

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(department.to_uppercase())
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{enabled_count}/{}", entries.len()))
                                .color(egui::Color32::WHITE),
                        );
                    });
                });

                egui::Grid::new(("settings-grid", department))
                    .num_columns(4)
                    .spacing(egui::vec2(4.0, 4.0))
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
                                        egui::vec2(112.0, 112.0),
                                        egui::Layout::top_down(egui::Align::Center),
                                        |tile| {
                                            let tile_rect = tile.available_rect_before_wrap();
                                            let icon_rect = egui::Rect::from_min_size(
                                                egui::pos2(
                                                    tile_rect.center().x - 28.0,
                                                    tile_rect.top() + 4.0,
                                                ),
                                                egui::vec2(56.0, 56.0),
                                            );
                                            let icon_response = tile.put(
                                                icon_rect,
                                                Self::stratagem_icon(stratagem, 56.0),
                                            );
                                            let checkbox_rect = egui::Rect::from_min_size(
                                                icon_response.rect.left_top(),
                                                egui::vec2(17.0, 17.0),
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
                                                    egui::RichText::new(stratagem.name).size(9.0),
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
    ) -> Option<egui::Rect> {
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
            ));
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
        let (tile_clicked, shortcut_clicked, shortcut_cleared) = ui
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
                (
                    tile_response.clicked(),
                    shortcut_response.clicked(),
                    shortcut_response.secondary_clicked(),
                )
            })
            .inner;

        if shortcut_cleared {
            cell.shortcut = None;
            cell.listening = false;
        } else if shortcut_clicked {
            cell.listening = true;
            *clicked_shortcut = Some((row_index, column_index));
        }

        tile_clicked.then_some(tile_rect)
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

    fn trigger_shortcuts(&mut self, ctx: &egui::Context) {
        if !Self::can_execute_macro() {
            return;
        }

        let pressed = ctx.input(|input| {
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

        let Some(key) = pressed else {
            return;
        };

        for row in &mut self.cells {
            for cell in row {
                if cell.listening || cell.shortcut != Some(key) {
                    continue;
                }

                if let Some(index) = cell.stratagem_index {
                    if let Some(stratagem) = STRATAGEMS.get(index) {
                        Self::run_macro(stratagem.sequence, self.use_arrow_keys);
                        return;
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn can_execute_macro() -> bool {
        is_helldivers_frontmost()
    }

    #[cfg(not(target_os = "windows"))]
    fn can_execute_macro() -> bool {
        false
    }

    #[cfg(target_os = "windows")]
    fn run_macro(sequence: &[&str], use_arrow_keys: bool) {
        unsafe {
            press_key(VK_CONTROL, true);
            for step in sequence {
                if let Some(vk) = virtual_key_for_step(step, use_arrow_keys) {
                    press_key(vk, true);
                    press_key(vk, false);
                }
            }
            press_key(VK_CONTROL, false);
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn run_macro(_sequence: &[&str], _use_arrow_keys: bool) {}
}

#[cfg(target_os = "windows")]
unsafe fn press_key(vk: u16, down: bool) {
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if down { 0 } else { KEYEVENTF_KEYUP },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    unsafe {
        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
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
fn is_helldivers_frontmost() -> bool {
    let foreground = unsafe { GetForegroundWindow() };
    if foreground == 0 {
        return false;
    }

    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(foreground, &mut pid);
    }
    if pid == 0 {
        return false;
    }

    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process == 0 {
        return false;
    }

    let mut buffer = [0u16; 512];
    let mut size = buffer.len() as u32;
    let ok = unsafe {
        QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr() as *mut u16, &mut size)
    };
    if ok == 0 {
        return false;
    }

    let exe_name = String::from_utf16_lossy(&buffer[..size as usize]);
    exe_name.to_ascii_lowercase().ends_with("helldivers2.exe")
}

#[cfg(target_os = "windows")]
fn set_window_opacity(frame: &eframe::Frame, focused: bool) {
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
        SetLayeredWindowAttributes(hwnd, 0, if focused { u8::MAX } else { 242 }, LWA_ALPHA);
    }
}

impl eframe::App for StratagemMacrosApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        #[cfg(target_os = "windows")]
        set_window_opacity(
            frame,
            ctx.input(|input| input.viewport().focused.unwrap_or(true)),
        );
        ctx.set_visuals(egui::Visuals::dark());

        egui::TopBottomPanel::bottom("creator-signature")
            .exact_height(40.0)
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(8, 10, 11)))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.add(
                        egui::Image::from_bytes(
                            "bytes://cybeduck-thumbnail.png",
                            include_bytes!("../assets/cybeduck_thumbnail.png"),
                        )
                        .fit_to_exact_size(egui::vec2(30.0, 30.0))
                        .alt_text("Danilo Cybeduck Tegon"),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("Created by Danilo 'Cybeduck' Tegon")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(190, 194, 194)),
                    );
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(8, 10, 11)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::from_bytes(
                            "bytes://super-earth-emblem.svg",
                            include_bytes!("../assets/super_earth_emblem.svg"),
                        )
                        .fit_to_exact_size(egui::vec2(22.0, 22.0))
                        .alt_text("MacroEarth emblem"),
                    );
                    let title = ui.add_sized(
                        [ui.available_width() - 44.0, 28.0],
                        egui::Label::new(
                            egui::RichText::new("MacroEarth")
                                .size(15.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .sense(egui::Sense::drag()),
                    );
                    if title.drag_started() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
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

                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.add_space(Self::GRID_CONTROL_INDENT);
                    let _ = ui.selectable_label(true, "Macros");
                    if ui.button("Settings").clicked() {
                        self.show_settings_window = !self.show_settings_window;
                    }
                });
                ui.separator();

                ui.horizontal(|ui| {
                    ui.add_space(Self::GRID_CONTROL_INDENT);
                    ui.checkbox(&mut self.use_arrow_keys, "Use arrow keys");
                });

                let mut clicked_cell = None;
                let mut remove_row = false;
                let mut remove_column = false;
                let mut add_row = false;
                let mut add_column = false;
                let mut focused_shortcut_rect = None;
                let mut clicked_shortcut = None;
                let column_count = self.columns;
                let row_count = self.rows;
                egui::ScrollArea::both()
                    .id_salt("macro-grid-v3")
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
                                    if let Some(button_rect) = Self::macro_cell_ui(
                                        ui,
                                        cell,
                                        row_index,
                                        col_index,
                                        &mut focused_shortcut_rect,
                                        &mut clicked_shortcut,
                                    ) {
                                        clicked_cell = Some((row_index, col_index, button_rect));
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

                let shortcut_captured = self.capture_shortcut(ctx);
                if !shortcut_captured {
                    self.trigger_shortcuts(ctx);
                }

                if let Some((row_index, column_index, button_rect)) = clicked_cell {
                    let viewport = ctx.input(|input| input.viewport().clone());
                    let window_origin = viewport
                        .outer_rect
                        .map_or(egui::Pos2::ZERO, |rect| rect.min);
                    let monitor_size = viewport.monitor_size.unwrap_or(egui::vec2(1920.0, 1080.0));
                    let anchor_rect = button_rect.translate(window_origin.to_vec2());
                    let popup_size = egui::vec2(320.0, 400.0);
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
    assert_eq!(smallest, egui::vec2(152.0, 268.0));
    assert_eq!(largest, egui::vec2(818.0, 934.0));
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
        egui::vec2(374.0, 490.0)
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
fn test_super_earth_window_icon_renders() {
    let icon = super_earth_icon();
    assert_eq!(icon.width, 128);
    assert_eq!(icon.height, 128);
    assert_eq!(icon.rgba.len(), (icon.width * icon.height * 4) as usize);
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
        Box::new(|_creation_context| Ok(Box::new(StratagemMacrosApp::default()))),
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
