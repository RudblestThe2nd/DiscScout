
use crate::tree::{Arena, NodeId};

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self { Self { x, y, w, h } }
    pub fn area(&self) -> f32 { self.w * self.h }
    pub fn shorter_side(&self) -> f32 { self.w.min(self.h) }
}

#[derive(Debug, Clone)]
pub struct TileRect {
    pub node_id: NodeId,
    pub rect:    Rect,
    pub depth:   usize,
    pub color_idx: usize,
}

fn worst_ratio(row: &[(NodeId, f32)], side: f32) -> f32 {
    let sum: f32 = row.iter().map(|(_, s)| s).sum();
    if sum == 0.0 || side == 0.0 { return f32::MAX; }
    row.iter().map(|(_, s)| {
        let a = side * side * s / (sum * sum);
        let b = sum * sum / (side * side * s);
        a.max(b)
    }).fold(f32::MIN, f32::max)
}

fn layout_row(
    row: &[(NodeId, f32)],
    bounds: &mut Rect,
    results: &mut Vec<TileRect>,
    depth: usize,
    color_base: usize,
) {
    let sum: f32 = row.iter().map(|(_, s)| s).sum();
    if sum == 0.0 { return; }
    let horiz = bounds.w >= bounds.h;
    let frac  = sum / (bounds.w * bounds.h);

    if horiz {
        let row_w = bounds.w * frac;
        let mut y = bounds.y;
        for (i, (node_id, size)) in row.iter().enumerate() {
            let h = bounds.h * (size / sum);
            results.push(TileRect {
                node_id: *node_id,
                rect: Rect::new(bounds.x, y, row_w, h),
                depth,
                color_idx: color_base + i,
            });
            y += h;
        }
        bounds.x += row_w;
        bounds.w -= row_w;
    } else {
        let row_h = bounds.h * frac;
        let mut x = bounds.x;
        for (i, (node_id, size)) in row.iter().enumerate() {
            let w = bounds.w * (size / sum);
            results.push(TileRect {
                node_id: *node_id,
                rect: Rect::new(x, bounds.y, w, row_h),
                depth,
                color_idx: color_base + i,
            });
            x += w;
        }
        bounds.y += row_h;
        bounds.h -= row_h;
    }
}

pub fn squarify(
    arena:   &Arena,
    node_id: NodeId,
    bounds:  Rect,
    depth:   usize,
) -> Vec<TileRect> {
    squarify_inner(arena, node_id, bounds, depth, 0)
}

fn squarify_inner(
    arena:      &Arena,
    node_id:    NodeId,
    bounds:     Rect,
    depth:      usize,
    color_base: usize,
) -> Vec<TileRect> {
    let node = arena.get(node_id);
    if node.children.is_empty() || bounds.area() < 4.0 { return vec![]; }

    let total = node.size as f32;
    if total == 0.0 { return vec![]; }

    let mut items: Vec<(NodeId, f32)> = node.children.iter()
        .filter_map(|&cid| {
            let sz = arena.get(cid).size as f32;
            if sz > 0.0 { Some((cid, sz)) } else { None }
        })
        .collect();
    items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let area = bounds.area();
    let mut normalized: Vec<(NodeId, f32)> = items.iter()
        .map(|(id, sz)| (*id, sz / total * area))
        .collect();

    let mut results = Vec::new();
    let mut current = bounds.clone();
    let mut row: Vec<(NodeId, f32)> = Vec::new();
    let mut placed = 0usize;

    while !normalized.is_empty() {
        let item = normalized[0].clone();
        let side  = current.shorter_side();
        let mut test = row.clone();
        test.push(item.clone());

        if row.is_empty() || worst_ratio(&test, side) <= worst_ratio(&row, side) {
            row.push(normalized.remove(0));
        } else {
            layout_row(&row, &mut current, &mut results, depth, color_base + placed);
            placed += row.len();
            row.clear();
        }
    }
    if !row.is_empty() {
        layout_row(&row, &mut current, &mut results, depth, color_base + placed);
    }

    // Recursive: her klasörün içine alt treemap çiz
    // Sadece belirli derinlik ve minimum boyut sınırı ile
    if depth < 2 {
        let children_tiles: Vec<TileRect> = results.clone();
        for tile in &children_tiles {
            let child = arena.get(tile.node_id);
            if !child.is_dir || child.children.is_empty() { continue; }

            // Header yüksekliğini blok genişliğine göre hesapla (app.rs ile senkron)
            let fsz: f32 = if tile.rect.w > 300.0 { 13.0 }
                           else if tile.rect.w > 150.0 { 11.0 }
                           else if tile.rect.w > 80.0  {  9.0 }
                           else if tile.rect.w > 40.0  {  8.0 }
                           else { 7.0 };
            let header_h = fsz + 6.0;
            let bevel    = 3.0;
            let pad      = 2.0;

            let inner = Rect::new(
                tile.rect.x + bevel + pad,
                tile.rect.y + bevel + header_h + pad,
                tile.rect.w - (bevel + pad) * 2.0,
                tile.rect.h - bevel - header_h - pad * 2.0 - bevel,
            );
            if inner.w < 6.0 || inner.h < 6.0 { continue; }

            let mut sub = squarify_inner(arena, tile.node_id, inner, depth + 1, tile.color_idx * 3);
            results.append(&mut sub);
        }
    }

    results
}
