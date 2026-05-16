use eframe::egui;
use egui::{Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::crawler;
use crate::squarify::{self, TileRect};
use crate::tree::{Arena, NodeId};

const PALETTE: &[Color32] = &[
    Color32::from_rgb(192, 57, 43),  Color32::from_rgb(41, 128, 185),
    Color32::from_rgb(39, 174, 96),  Color32::from_rgb(243, 156, 18),
    Color32::from_rgb(142, 68, 173), Color32::from_rgb(22, 160, 133),
    Color32::from_rgb(211, 84, 0),   Color32::from_rgb(36, 113, 163),
    Color32::from_rgb(30, 132, 73),  Color32::from_rgb(203, 67, 53),
    Color32::from_rgb(125, 60, 152), Color32::from_rgb(20, 143, 119),
    Color32::from_rgb(183, 149, 11), Color32::from_rgb(26, 82, 118),
    Color32::from_rgb(146, 43, 33),
];

fn palette(i: usize) -> Color32 { PALETTE[i % PALETTE.len()] }

fn lighten(c: Color32, f: f32) -> Color32 {
    Color32::from_rgb(
        (c.r() as f32 + (255.0 - c.r() as f32) * f).min(255.0) as u8,
        (c.g() as f32 + (255.0 - c.g() as f32) * f).min(255.0) as u8,
        (c.b() as f32 + (255.0 - c.b() as f32) * f).min(255.0) as u8,
    )
}

fn darken(c: Color32, f: f32) -> Color32 {
    Color32::from_rgb(
        (c.r() as f32 * (1.0 - f)) as u8,
        (c.g() as f32 * (1.0 - f)) as u8,
        (c.b() as f32 * (1.0 - f)) as u8,
    )
}

fn disk_usage(path: &str) -> (u64, u64) {
    use std::ffi::CString;
    use std::mem::MaybeUninit;
    let c_path = CString::new(path).unwrap_or_default();
    unsafe {
        let mut stat: libc::statvfs = MaybeUninit::zeroed().assume_init();
        if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
            let total = stat.f_blocks * stat.f_frsize;
            let free  = stat.f_bfree  * stat.f_frsize;
            let used  = total.saturating_sub(free);
            return (used, total);
        }
    }
    (0, 0)
}

enum ScanState {
    Idle,
    Scanning,
    Done { arena: Arena, root_id: NodeId },
}

pub struct DiskMapperApp {
    scan_path:   String,
    state:       Arc<Mutex<ScanState>>,
    current_id:  Option<NodeId>,
    tiles:       Vec<TileRect>,
    hovered_id:   Option<NodeId>,
    sort_col:     usize,
    sort_desc:    bool,
    list_height:  f32,
    expanded_ids: std::collections::HashSet<NodeId>,
}

impl DiskMapperApp {
    pub fn new() -> Self {
        Self {
            scan_path:   std::env::var("HOME").unwrap_or_else(|_| "/home".into()),
            state:       Arc::new(Mutex::new(ScanState::Idle)),
            current_id:  None,
            tiles:       Vec::new(),
            hovered_id:   None,
            sort_col:     2,
            sort_desc:    true,
            list_height:  280.0,
            expanded_ids: std::collections::HashSet::new(),
        }
    }

    fn draw_tree_rows_flat(
        ui: &mut egui::Ui,
        state: &std::sync::Arc<std::sync::Mutex<ScanState>>,
        children: &[(NodeId, String, u64, bool, Vec<NodeId>)],
        parent_size: f64,
        expanded: &std::collections::HashSet<NodeId>,
        hovered_id: &mut Option<NodeId>,
        to_expand: &mut Option<NodeId>,
        to_collapse: &mut Option<NodeId>,
        depth: usize,
    ) {
        for (node_id, name, size, is_dir, sub_children) in children {
            let pct = *size as f64 / parent_size * 100.0;
            let is_expanded = expanded.contains(node_id);
            let indent = "  ".repeat(depth);
            let arrow = if *is_dir && !sub_children.is_empty() {
                if is_expanded { "v " } else { "> " }
            } else { "  " };
            let nc = if *is_dir { Color32::from_rgb(126, 200, 227) } else { Color32::from_rgb(200, 204, 212) };
            let name_txt = format!("{}{}{}", indent, arrow, name);
            let resp = ui.add(
                egui::Label::new(egui::RichText::new(&name_txt).monospace().size(12.0).color(nc))
                    .sense(egui::Sense::click())
            );
            if resp.hovered() { *hovered_id = Some(*node_id); }
            if resp.clicked() && *is_dir && !sub_children.is_empty() {
                if is_expanded { *to_collapse = Some(*node_id); }
                else           { *to_expand   = Some(*node_id); }
            }
            ui.label(egui::RichText::new(Self::fmt_size(*size)).monospace().size(12.0).color(Color32::from_rgb(200, 200, 200)));
            let pc = if pct > 40.0 { Color32::from_rgb(239, 83, 80) }
                else if pct > 15.0 { Color32::from_rgb(255, 167, 38) }
                else               { Color32::from_rgb(102, 187, 106) };
            ui.label(egui::RichText::new(format!("{:.1}%", pct)).monospace().size(12.0).color(pc));
            ui.label(egui::RichText::new(format!("{}", sub_children.len())).monospace().size(12.0).color(Color32::from_rgb(130, 130, 150)));
            ui.end_row();
            if is_expanded && *is_dir && !sub_children.is_empty() {
                let sub_data: Vec<(NodeId, String, u64, bool, Vec<NodeId>)> = {
                    let st = state.lock().unwrap();
                    if let ScanState::Done { arena, .. } = &*st {
                        let mut v: Vec<_> = sub_children.iter().map(|&id| {
                            let n = arena.get(id);
                            (id, n.name.clone(), n.size, n.is_dir, n.children.clone())
                        }).collect();
                        v.sort_by(|a, b| b.2.cmp(&a.2));
                        v
                    } else { vec![] }
                };
                Self::draw_tree_rows_flat(ui, state, &sub_data, *size as f64,
                    expanded, hovered_id, to_expand, to_collapse, depth + 1);
            }
        }
    }

    fn start_scan(&mut self, path: String) {
        let state = Arc::clone(&self.state);
        *state.lock().unwrap() = ScanState::Scanning;
        self.tiles.clear();
        self.current_id = None;
        self.hovered_id = None;
        thread::spawn(move || {
            let files = crawler::scan(&path);
            let (arena, root_id) = crate::tree::build_tree(&path, files);
            *state.lock().unwrap() = ScanState::Done { arena, root_id };
        });
    }

    fn fmt_size(bytes: u64) -> String {
        const GB: u64 = 1 << 30;
        const MB: u64 = 1 << 20;
        const KB: u64 = 1 << 10;
        if bytes >= GB      { format!("{:.2} GB", bytes as f64 / GB as f64) }
        else if bytes >= MB { format!("{:.1} MB", bytes as f64 / MB as f64) }
        else if bytes >= KB { format!("{:.0} KB", bytes as f64 / KB as f64) }
        else                { format!("{} B", bytes) }
    }

    // Sadece renk + bevel + border çizer. Label YOK.
    fn draw_block(painter: &egui::Painter, rect: Rect, color: Color32, hovered: bool, depth: usize) {
        if rect.width() < 2.0 || rect.height() < 2.0 { return; }
        let b   = 3.0_f32;
        let top = lighten(color, 0.50);
        let bot = darken(color, 0.50);
        let face = if hovered { lighten(color, 0.28) } else { color };

        painter.rect_filled(Rect::from_min_size(rect.min, Vec2::new(rect.width(), b)), Rounding::ZERO, top);
        painter.rect_filled(Rect::from_min_size(rect.min, Vec2::new(b, rect.height())), Rounding::ZERO, top);
        painter.rect_filled(Rect::from_min_size(Pos2::new(rect.min.x, rect.max.y - b), Vec2::new(rect.width(), b)), Rounding::ZERO, bot);
        painter.rect_filled(Rect::from_min_size(Pos2::new(rect.max.x - b, rect.min.y), Vec2::new(b, rect.height())), Rounding::ZERO, bot);

        let inner = Rect::from_min_max(
            Pos2::new(rect.min.x + b, rect.min.y + b),
            Pos2::new(rect.max.x - b, rect.max.y - b),
        );
        if inner.is_positive() {
            painter.rect_filled(inner, Rounding::ZERO, face);
        }

        let bw = if depth == 0 { 2.0 } else { 1.0 };
        let bc = if depth == 0 { Color32::from_rgb(5, 5, 15) } else { Color32::from_rgb(20, 20, 35) };
        painter.rect_stroke(rect, Rounding::ZERO, Stroke::new(bw, bc));

        if hovered {
            painter.rect_stroke(rect.shrink(1.5), Rounding::ZERO, Stroke::new(2.5, Color32::from_rgb(255, 255, 60)));
        }
    }

    // depth==0 ve depth==1 bloklar için header şeridi + label çizer
    fn draw_label(painter: &egui::Painter, rect: Rect, color: Color32, name: &str, size_str: &str) {
        let b = 3.0_f32;
        let inner = Rect::from_min_max(
            Pos2::new(rect.min.x + b, rect.min.y + b),
            Pos2::new(rect.max.x - b, rect.max.y - b),
        );
        if !inner.is_positive() { return; }

        let fsz: f32 = if inner.width() > 300.0 { 13.0 }
                       else if inner.width() > 150.0 { 11.0 }
                       else if inner.width() > 80.0  {  9.0 }
                       else if inner.width() > 40.0  {  8.0 }
                       else { 7.0 };
        let header_h = fsz + 6.0;

        if inner.height() > header_h + 4.0 && inner.width() > 14.0 {
            // Koyu header şeridi
            let hdr = Rect::from_min_size(inner.min, Vec2::new(inner.width(), header_h));
            painter.rect_filled(hdr, Rounding::ZERO, darken(color, 0.45));

            // İsim — clip ile sınırla
            let clipped = painter.with_clip_rect(hdr);
            clipped.text(
                Pos2::new(inner.min.x + 3.0, inner.min.y + 2.0),
                egui::Align2::LEFT_TOP, name,
                egui::FontId::monospace(fsz), Color32::WHITE,
            );

            // Boyut sağda — isim ile çakışmıyorsa
            let size_fsz = (fsz - 1.0).max(7.0);
            let name_px = name.len() as f32 * fsz * 0.62 + 10.0;
            let size_px = size_str.len() as f32 * size_fsz * 0.62 + 6.0;
            if inner.width() > name_px + size_px + 4.0 {
                clipped.text(
                    Pos2::new(inner.max.x - 3.0, inner.min.y + 2.0),
                    egui::Align2::RIGHT_TOP, size_str,
                    egui::FontId::monospace(size_fsz),
                    Color32::from_rgba_premultiplied(210, 210, 210, 210),
                );
            }
        } else if inner.width() > 14.0 && inner.height() > 8.0 {
            // Çok küçük — isim clip ile
            let clipped = painter.with_clip_rect(inner);
            clipped.text(
                Pos2::new(inner.min.x + 2.0, inner.min.y + 1.0),
                egui::Align2::LEFT_TOP, name,
                egui::FontId::monospace(fsz.min(8.0)), Color32::WHITE,
            );
        }
    }
}

impl DiskMapperApp {
    fn draw_tree_rows(
        ui: &mut egui::Ui,
        arena: &crate::tree::Arena,
        children: &[(NodeId, &crate::tree::Node)],
        parent_size: f64,
        expanded: &std::collections::HashSet<NodeId>,
        hovered_id: &mut Option<NodeId>,
        to_expand: &mut Option<NodeId>,
        to_collapse: &mut Option<NodeId>,
        depth: usize,
    ) {
        for (node_id, node) in children {
            let pct = node.size as f64 / parent_size * 100.0;
            let is_expanded = expanded.contains(node_id);
            let indent = "  ".repeat(depth);

            // Arrow icon
            let arrow = if node.is_dir && !node.children.is_empty() {
                if is_expanded { "v " } else { "> " }
            } else { "  " };

            let nc = if node.is_dir {
                Color32::from_rgb(126, 200, 227)
            } else {
                Color32::from_rgb(200, 204, 212)
            };

            let name_txt = format!("{}{}{}", indent, arrow, node.name);
            let resp = ui.add(
                egui::Label::new(
                    egui::RichText::new(&name_txt).monospace().size(12.0).color(nc)
                ).sense(egui::Sense::click())
            );

            if resp.hovered() { *hovered_id = Some(*node_id); }
            if resp.clicked() {
                if node.is_dir && !node.children.is_empty() {
                    if is_expanded { *to_collapse = Some(*node_id); }
                    else           { *to_expand   = Some(*node_id); }
                }
            }

            // Size
            ui.label(egui::RichText::new(Self::fmt_size(node.size))
                .monospace().size(12.0).color(Color32::from_rgb(200, 200, 200)));

            // %
            let pc = if pct > 40.0 { Color32::from_rgb(239, 83, 80) }
                else if pct > 15.0 { Color32::from_rgb(255, 167, 38) }
                else               { Color32::from_rgb(102, 187, 106) };
            ui.label(egui::RichText::new(format!("{:.1}%", pct))
                .monospace().size(12.0).color(pc));

            // Files
            ui.label(egui::RichText::new(format!("{}", node.children.len()))
                .monospace().size(12.0).color(Color32::from_rgb(130, 130, 150)));

            ui.end_row();

            // Alt öğeler — expand edilmişse göster
            if is_expanded && node.is_dir && !node.children.is_empty() {
                let mut sub: Vec<(NodeId, &crate::tree::Node)> = node.children.iter()
                    .map(|&id| (id, arena.get(id)))
                    .collect();
                sub.sort_by(|a, b| b.1.size.cmp(&a.1.size));
                Self::draw_tree_rows(
                    ui, arena, &sub, node.size.max(1) as f64,
                    expanded, hovered_id, to_expand, to_collapse, depth + 1,
                );
            }
        }
    }
}

impl eframe::App for DiskMapperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill       = Color32::from_rgb(13, 13, 26);
        visuals.extreme_bg_color = Color32::from_rgb(8, 8, 18);
        ctx.set_visuals(visuals);

        {
            let st = self.state.lock().unwrap();
            match &*st {
                ScanState::Scanning => ctx.request_repaint(),
                ScanState::Done { .. } if self.tiles.is_empty() => ctx.request_repaint(),
                _ => {}
            }
        }

        // ── Toolbar ───────────────────────────────────────────
        egui::TopBottomPanel::top("toolbar")
            .exact_height(52.0)
            .frame(egui::Frame::default()
                .fill(Color32::from_rgb(18, 18, 35))
                .inner_margin(egui::Margin::symmetric(10.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.scan_path)
                        .desired_width(300.0)
                        .font(egui::FontId::monospace(13.0))
                        .text_color(Color32::from_rgb(100, 180, 255)));

                    let scanning = matches!(*self.state.lock().unwrap(), ScanState::Scanning);

                    if ui.add_enabled(!scanning,
                        egui::Button::new(egui::RichText::new("  Scan  ").monospace().size(13.0))
                            .fill(Color32::from_rgb(21, 101, 192))
                    ).clicked() {
                        self.start_scan(self.scan_path.clone());
                    }

                    let can_up = self.current_id.map_or(false, |id| {
                        let st = self.state.lock().unwrap();
                        if let ScanState::Done { arena, .. } = &*st {
                            arena.get(id).parent.is_some()
                        } else { false }
                    });

                    if ui.add_enabled(can_up,
                        egui::Button::new(egui::RichText::new("  Up  ").monospace().size(13.0))
                            .fill(Color32::from_rgb(30, 30, 55))
                    ).clicked() {
                        let pid: Option<NodeId> = {
                            let st = self.state.lock().unwrap();
                            if let ScanState::Done { arena, .. } = &*st {
                                self.current_id.and_then(|id| arena.get(id).parent)
                            } else { None }
                        };
                        if let Some(p) = pid {
                            self.current_id = Some(p);
                            self.tiles.clear();
                        }
                    }

                    if scanning {
                        ui.spinner();
                        ui.label(egui::RichText::new("Scanning...").monospace().color(Color32::YELLOW).size(13.0));
                    }

                    ui.separator();

                    let st = self.state.lock().unwrap();
                    if let ScanState::Done { arena, root_id } = &*st {
                        if let Some(cur) = self.current_id {
                            let node = arena.get(cur);
                            ui.label(egui::RichText::new(&node.path)
                                .monospace().size(11.0)
                                .color(Color32::from_rgb(100, 180, 255)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let root_size = arena.get(*root_id).size;
                                let cur_size  = arena.get(cur).size;
                                // Gerçek disk kullanımını al
                                let (disk_used, disk_total) = {
                                    use std::fs;
                                    let path = &arena.get(*root_id).path;
                                    if let Ok(stat) = fs::metadata(path) {
                                        // statvfs ile disk bilgisi
                                        let s = disk_usage(path);
                                        s
                                    } else {
                                        (root_size, root_size)
                                    }
                                };
                                let pct = if disk_total > 0 { disk_used as f64 / disk_total as f64 * 100.0 } else { 0.0 };
                                ui.label(egui::RichText::new(format!(
                                    "Disk: {} / {} ({:.1}% used)",
                                    Self::fmt_size(disk_used),
                                    Self::fmt_size(disk_total),
                                    pct,
                                )).monospace().size(11.0).color(Color32::from_rgb(140, 200, 140)));
                            });
                        }
                    }
                });
            });

        // ── File List (bottom, resizable) ─────────────────────
        egui::TopBottomPanel::bottom("file_list_panel")
            .resizable(true)
            .min_height(80.0)
            .max_height(600.0)
            .default_height(self.list_height)
            .frame(egui::Frame::default().fill(Color32::from_rgb(13, 13, 26)))
            .show(ctx, |ui| {
                // Önce veriyi lock içinde kopyala, sonra lock'u bırak
                let list_data: Option<(Vec<(NodeId, String, u64, bool, Vec<NodeId>)>, f64)> = {
                    let st = self.state.lock().unwrap();
                    if let ScanState::Done { arena, .. } = &*st {
                        if let Some(cur_id) = self.current_id {
                            let cur = arena.get(cur_id);
                            let parent_size = cur.size.max(1) as f64;
                            let mut children: Vec<(NodeId, String, u64, bool, Vec<NodeId>)> =
                                cur.children.iter().map(|&id| {
                                    let n = arena.get(id);
                                    (id, n.name.clone(), n.size, n.is_dir, n.children.clone())
                                }).collect();
                            match self.sort_col {
                                0 => children.sort_by(|a, b| {
                                    let o = a.1.cmp(&b.1);
                                    if self.sort_desc { o.reverse() } else { o }
                                }),
                                _ => children.sort_by(|a, b| {
                                    let o = a.2.cmp(&b.2);
                                    if self.sort_desc { o.reverse() } else { o }
                                }),
                            }
                            Some((children, parent_size))
                        } else { None }
                    } else { None }
                };

                let is_scanning = matches!(*self.state.lock().unwrap(), ScanState::Scanning);

                if let Some((children, parent_size)) = list_data {
                    let mut to_expand:   Option<NodeId> = None;
                    let mut to_collapse: Option<NodeId> = None;

                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        egui::Grid::new("file_list").striped(true).spacing([4.0, 1.0]).min_col_width(60.0).show(ui, |ui| {
                            // Header
                            let cols = ["Name", "Size", "% of Parent", "Files"];
                            for (i, label) in cols.iter().enumerate() {
                                let txt = if self.sort_col == i {
                                    format!("{} {}", label, if self.sort_desc { "v" } else { "^" })
                                } else { label.to_string() };
                                if ui.add(egui::Label::new(
                                    egui::RichText::new(txt).monospace().size(11.0).strong()
                                        .color(Color32::from_rgb(126, 200, 227))
                                ).sense(egui::Sense::click())).clicked() {
                                    if self.sort_col == i { self.sort_desc = !self.sort_desc; }
                                    else { self.sort_col = i; self.sort_desc = true; }
                                }
                            }
                            ui.end_row();

                            // Tree rows
                            Self::draw_tree_rows_flat(
                                ui, &self.state, &children, parent_size,
                                &self.expanded_ids, &mut self.hovered_id,
                                &mut to_expand, &mut to_collapse, 0,
                            );
                        });
                    });

                    if let Some(id) = to_expand   { self.expanded_ids.insert(id); }
                    if let Some(id) = to_collapse { self.expanded_ids.remove(&id); }

                } else if is_scanning {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("Scanning...").monospace().size(14.0).color(Color32::YELLOW));
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("Enter a path and click [Scan]").monospace().size(13.0).color(Color32::from_rgb(70, 70, 100)));
                    });
                }
            });

        // ── Treemap (center) ──────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::from_rgb(10, 10, 20)))
            .show(ctx, |ui| {
                let tr = ui.available_rect_before_wrap();
                let painter = ui.painter_at(tr);
                painter.rect_filled(tr, Rounding::ZERO, Color32::from_rgb(10, 10, 20));

                // Tile rebuild
                if self.tiles.is_empty() {
                    let target: Option<NodeId> = {
                        let st = self.state.lock().unwrap();
                        if let ScanState::Done { arena, root_id } = &*st {
                            Some(self.current_id.unwrap_or(*root_id))
                        } else { None }
                    };
                    if let Some(tid) = target {
                        let st = self.state.lock().unwrap();
                        if let ScanState::Done { arena, .. } = &*st {
                            let bounds = squarify::Rect::new(tr.min.x, tr.min.y, tr.width(), tr.height());
                            self.tiles = squarify::squarify(arena, tid, bounds, 0);
                            self.current_id = Some(tid);
                        }
                    }
                }

                if self.tiles.is_empty() {
                    painter.text(tr.center(), egui::Align2::CENTER_CENTER,
                        "Select a directory to scan",
                        egui::FontId::monospace(14.0), Color32::from_rgb(50, 50, 80));
                    return;
                }

                let mouse_pos = ctx.input(|i| i.pointer.hover_pos());
                let clicked   = ctx.input(|i| i.pointer.primary_clicked());
                let mut new_hov: Option<NodeId> = None;
                let mut nav_to: Option<NodeId>  = None;

                // Pass 1: sadece renk + bevel
                for tile in self.tiles.iter() {
                    let r = Rect::from_min_max(
                        Pos2::new(tile.rect.x, tile.rect.y),
                        Pos2::new(tile.rect.x + tile.rect.w, tile.rect.y + tile.rect.h),
                    );
                    if !tr.intersects(r) { continue; }
                    let hov = mouse_pos.map_or(false, |p| r.contains(p) && tr.contains(p));
                    if hov {
                        new_hov = Some(tile.node_id);
                        if clicked { nav_to = Some(tile.node_id); }
                    }
                    Self::draw_block(&painter, r, palette(tile.color_idx), hov, tile.depth);
                }

                // Pass 2: depth==0 ve yeterince büyük depth==1 bloklara label
                for tile in self.tiles.iter().filter(|t| {
                    t.depth == 0 || (t.depth == 1 && t.rect.w > 60.0 && t.rect.h > 20.0)
                }) {
                    let r = Rect::from_min_max(
                        Pos2::new(tile.rect.x, tile.rect.y),
                        Pos2::new(tile.rect.x + tile.rect.w, tile.rect.y + tile.rect.h),
                    );
                    if !tr.intersects(r) { continue; }

                    let hov = mouse_pos.map_or(false, |p| r.contains(p) && tr.contains(p));
                    let st = self.state.lock().unwrap();
                    if let ScanState::Done { arena, .. } = &*st {
                        let node = arena.get(tile.node_id);
                        let sz = Self::fmt_size(node.size);
                        Self::draw_label(&painter, r, palette(tile.color_idx), &node.name, &sz);

                        // Tooltip
                        if hov && node.is_dir && !node.children.is_empty() {
                            let mut kids: Vec<_> = node.children.iter().map(|&id| arena.get(id)).collect();
                            kids.sort_by(|a, b| b.size.cmp(&a.size));
                            let mut tip = format!("{}\n{}\n----------------\n", node.name, Self::fmt_size(node.size));
                            for kid in kids.iter().take(8) {
                                let pct = kid.size as f64 / node.size.max(1) as f64 * 100.0;
                                tip.push_str(&format!("{} {:>9}  {:4.1}%  {}\n",
                                    if kid.is_dir { "[D]" } else { "   " },
                                    Self::fmt_size(kid.size), pct, kid.name));
                            }
                            if node.children.len() > 8 {
                                tip.push_str(&format!("... {} more", node.children.len() - 8));
                            }
                            egui::show_tooltip_at_pointer(ctx, ui.layer_id(), egui::Id::new("tt"), |ui| {
                                ui.label(egui::RichText::new(&tip).monospace().size(11.0).color(Color32::from_rgb(220, 220, 220)));
                            });
                        }
                    }
                }

                self.hovered_id = new_hov;

                if let Some(cid) = nav_to {
                    let ok = {
                        let st = self.state.lock().unwrap();
                        if let ScanState::Done { arena, .. } = &*st {
                            let n = arena.get(cid);
                            n.is_dir && !n.children.is_empty()
                        } else { false }
                    };
                    if ok {
                        self.current_id = Some(cid);
                        self.tiles.clear();
                    }
                }
            });
    }
}