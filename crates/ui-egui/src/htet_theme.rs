//! HtetWord's Feminine Floating & Liquid Glass Theme and Visual Effects.
//!
//! Defines the custom `craft-design-theme` (`htetword.rose_glass`) with:
//! - Feminine Rose Quartz & Blossom Cream light palette + Velvet Plum dark palette
//! - Soft capsule/pill geometry (`button_radius: 10.0`, generous surface & window radii, round color wells)
//! - Two-tone Liquid Glass specular bevel edge (`edge.light` = luminous white top-left highlight,
//!   `edge.dark` = soft rose refraction bottom-right edge)
//! - Floating Liquid Glass chrome overlays (specular sheen, soft rose ambient glow, floating glass cards)

use std::sync::{Arc, LazyLock};

use craft::theme::ThemeDef;
use craft::ui::runtime::env::ShellRegions;
use egui::{Color32, CornerRadius, Pos2, Rect, Stroke, StrokeKind, Ui, pos2, vec2};

/// HtetWord's Feminine Rose-Quartz & Velvet-Plum Liquid Glass theme JSON definition.
const HTET_ROSE_GLASS_JSON: &str = r##"{
  "schema": "craft-design-theme",
  "schema_version": 1,
  "id": "htetword.rose_glass",
  "name": "HtetWord Rose Quartz Glass",
  "version": 1,
  "variants": "both",
  "contrast_intent": "standard",
  "seeds": {
    "accent": "#c23872",
    "neutral": "#422836",
    "status": {
      "bad": "#c92a49",
      "good": "#247a54",
      "info": "#9b3f96",
      "warn": "#b85a14"
    }
  },
  "shape": {
    "button_radius": 10.0,
    "edge": {
      "bevel": {
        "width": 1.25
      }
    },
    "radius_offset": {
      "control": 4.0,
      "surface": 8.0,
      "window": 10.0
    },
    "wells": "round"
  },
  "light": {
    "seeds": {
      "accent": "#c23872",
      "neutral": "#fceef5",
      "status": {
        "bad": "#b42341",
        "good": "#1f754e",
        "info": "#8d348a",
        "warn": "#9c4808"
      }
    },
    "accent.border": "#c23872b8",
    "accent.soft": "#fbd5e7",
    "accent.solid": "#c23872",
    "accent.strong": "#9c2458",
    "accent.text": "#8f1d4e",
    "caption.close_hover": "#c92a49",
    "caption.close_text": "#ffffff",
    "edge.light": "#fffffff5",
    "edge.dark": "#d99ab866",
    "line.border": "#ecc8da",
    "line.control_edge": "#cfa0b8",
    "line.divider": "#efd1e0",
    "state.hover": "#fbe0ee",
    "state.pressed": "#f5c8df",
    "state.selected": "#fad2e6",
    "state.selected_hover": "#f7c2dd",
    "surface.background": "#fff5fa",
    "surface.canvas": "#f7e2ec",
    "surface.chrome": "#fdebf3",
    "surface.panel": "#fffafc",
    "surface.raised": "#ffffff",
    "surface.sunken": "#fffbfd",
    "surface.shadow": "#6e224626",
    "text.disabled": "#b092a3",
    "text.faint": "#755064",
    "text.muted": "#5c3b4d",
    "text.normal": "#2b1622",
    "text.on_accent": "#ffffff"
  },
  "dark": {
    "accent.border": "#f288b6b0",
    "accent.soft": "#5c284b",
    "accent.solid": "#f288b6",
    "accent.strong": "#c2447c",
    "accent.text": "#ffafd2",
    "caption.close_hover": "#d63c58",
    "caption.close_text": "#ffffff",
    "edge.light": "#ffd6ec38",
    "edge.dark": "#0a040e90",
    "line.border": "#4d3259",
    "line.control_edge": "#6e4a7d",
    "line.divider": "#432b4f",
    "state.hover": "#422a4f",
    "state.pressed": "#543563",
    "state.selected": "#5a284b",
    "state.selected_hover": "#6b3059",
    "surface.background": "#25182d",
    "surface.canvas": "#160e1b",
    "surface.chrome": "#1f1426",
    "surface.panel": "#2c1d36",
    "surface.raised": "#352340",
    "surface.sunken": "#1d1222",
    "surface.shadow": "#00000078",
    "text.disabled": "#7d6275",
    "text.faint": "#baa0b2",
    "text.muted": "#dec4d5",
    "text.normal": "#faedf5",
    "text.on_accent": "#240d1b"
  }
}"##;

/// Parsed singleton `ThemeDef` for HtetWord's Feminine Liquid Glass design.
pub static HTET_THEME: LazyLock<Arc<ThemeDef>> = LazyLock::new(|| match craft::theme::format::parse(HTET_ROSE_GLASS_JSON.as_bytes()) {
    Ok(file) => Arc::new(file.theme),
    Err(_) => Arc::new(
        craft::theme::builtins()
            .iter()
            .find(|t| t.id.as_str() == "craft.soft")
            .or_else(|| craft::theme::builtins().first())
            .cloned()
            .unwrap_or_else(|| unreachable_fallback()),
    ),
});

fn unreachable_fallback() -> ThemeDef {
    // `craft::theme::builtins()` always has 10 embedded themes; this branch is never reached.
    craft::theme::builtins()[0].clone()
}

/// Paints HtetWord's Floating & Liquid Glass specular highlights, translucent frosted capsules,
/// and soft feminine rose/lavender refraction rims over the shell's bars and panels.
pub fn paint_liquid_glass_overlays(ui: &Ui, regions: &ShellRegions, dark: bool) {
    let painter = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::unique("htetword_liquid_glass_sheen")));

    let specular_top =
        if dark { Color32::from_rgba_unmultiplied(0xFF, 0xD8, 0xEE, 36) } else { Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 210) };
    let glass_rim = if dark { Color32::from_rgba_unmultiplied(0xF2, 0x88, 0xB6, 48) } else { Color32::from_rgba_unmultiplied(0xC2, 0x38, 0x72, 34) };
    let soft_glow = if dark { Color32::from_rgba_unmultiplied(0xF2, 0x88, 0xB6, 12) } else { Color32::from_rgba_unmultiplied(0xFF, 0xC8, 0xE2, 26) };

    for (slot_key, rect) in &regions.slots {
        // 1. Floating Liquid Glass Ribbon Capsule Outline & Specular Top Highlight
        if *slot_key == "ribbon" || *slot_key == "ribbonDrop" {
            let r = *rect;
            if r.width() > 80.0 && r.height() > 20.0 {
                let inset = r.shrink2(vec2(6.0, 2.0));
                let radius = CornerRadius::same(14);
                // Soft outer rose bloom
                painter.rect_stroke(inset.expand(1.0), radius, Stroke::new(1.5, soft_glow), StrokeKind::Outside);
                // Frosted glass rim
                painter.rect_stroke(inset, radius, Stroke::new(1.0, glass_rim), StrokeKind::Inside);
                // Top specular glass reflection line
                let line_y = inset.min.y + 1.0;
                painter.line_segment([pos2(inset.min.x + 14.0, line_y), pos2(inset.max.x - 14.0, line_y)], Stroke::new(1.25, specular_top));
            }
        } else if *slot_key == "status" {
            // 2. Floating Liquid Glass Status Bar Capsule Rim & Specular Edge
            let r = *rect;
            if r.width() > 80.0 && r.height() > 14.0 {
                let inset = r.shrink2(vec2(8.0, 2.0));
                let radius = CornerRadius::same(12);
                painter.rect_stroke(inset, radius, Stroke::new(1.0, glass_rim), StrokeKind::Inside);
                let line_y = inset.min.y + 1.0;
                painter.line_segment([pos2(inset.min.x + 12.0, line_y), pos2(inset.max.x - 12.0, line_y)], Stroke::new(1.0, specular_top));
            }
        }
    }
}

/// Paints a soft feminine Rose-Quartz & Lavender-Mist ambient aurora glow on the canvas surround
/// behind the document pages, plus a floating glass frame around each placed page.
pub fn paint_canvas_aurora_and_page_glass(ui: &Ui, canvas_rect: Rect, page_rects: &[Rect], dark: bool) {
    if !canvas_rect.is_positive() {
        return;
    }
    let painter = ui.painter().with_clip_rect(canvas_rect);

    // Subtle corner pastel aurora blooms on the canvas surround (drawn only around the pages)
    let orb_rose = if dark { Color32::from_rgba_unmultiplied(0xC2, 0x38, 0x72, 16) } else { Color32::from_rgba_unmultiplied(0xFF, 0xB8, 0xD8, 36) };
    let orb_lavender =
        if dark { Color32::from_rgba_unmultiplied(0x8E, 0x44, 0xAD, 14) } else { Color32::from_rgba_unmultiplied(0xDC, 0xC2, 0xFF, 32) };

    // Soft glass frame & specular highlight around each page on screen
    let page_rim = if dark { Color32::from_rgba_unmultiplied(0xF2, 0x88, 0xB6, 64) } else { Color32::from_rgba_unmultiplied(0xC2, 0x38, 0x72, 48) };
    let page_glow = if dark { Color32::from_rgba_unmultiplied(0xF2, 0x88, 0xB6, 18) } else { Color32::from_rgba_unmultiplied(0xD8, 0x64, 0x98, 22) };

    // Draw subtle ambient side glow strips in the canvas margins
    let left_margin = Rect::from_min_max(canvas_rect.min, Pos2::new((canvas_rect.min.x + 72.0).min(canvas_rect.max.x), canvas_rect.max.y));
    let right_margin = Rect::from_min_max(Pos2::new((canvas_rect.max.x - 72.0).max(canvas_rect.min.x), canvas_rect.min.y), canvas_rect.max);
    if page_rects.iter().all(|p| p.min.x > left_margin.max.x) {
        painter.rect_filled(left_margin, CornerRadius::ZERO, orb_rose);
    }
    if page_rects.iter().all(|p| p.max.x < right_margin.min.x) {
        painter.rect_filled(right_margin, CornerRadius::ZERO, orb_lavender);
    }

    for &page_rect in page_rects {
        if !page_rect.is_positive() {
            continue;
        }
        painter.rect_stroke(page_rect.expand(2.0), CornerRadius::same(4), Stroke::new(2.5, page_glow), StrokeKind::Outside);
        painter.rect_stroke(page_rect, CornerRadius::same(2), Stroke::new(1.0, page_rim), StrokeKind::Outside);
    }
}
