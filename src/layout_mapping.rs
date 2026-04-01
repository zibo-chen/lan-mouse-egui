use std::collections::HashMap;

use lan_mouse_ipc::{ClientHandle, DisplayInfo, LayoutRect};

/// Layout mapping data shared with the capture task for coordinate transformation.
#[derive(Clone, Debug, Default)]
pub(crate) struct LayoutMapping {
    pub local_screens: Vec<DisplayInfo>,
    pub local_layout_rects: Vec<LayoutRect>,
    pub client_info: HashMap<ClientHandle, ClientLayoutInfo>,
}

/// Per-client layout info needed for coordinate mapping.
#[derive(Clone, Debug)]
pub(crate) struct ClientLayoutInfo {
    pub layout_rects: Vec<LayoutRect>,
    pub screens: Vec<DisplayInfo>,
}

impl LayoutMapping {
    /// Compute the entry point on a target client's display space
    /// given a crossing point in local display coordinates.
    pub fn compute_entry_point(
        &self,
        crossing_x: f64,
        crossing_y: f64,
        handle: ClientHandle,
    ) -> Option<(f64, f64)> {
        let client = self.client_info.get(&handle)?;
        if client.layout_rects.is_empty() || self.local_layout_rects.is_empty() {
            return None;
        }

        // Map crossing point from local display coords to layout space
        let (layout_x, layout_y) = local_display_to_layout(
            crossing_x,
            crossing_y,
            &self.local_screens,
            &self.local_layout_rects,
        );

        // Find adjacent client rect and compute entry point in layout space
        let (entry_lx, entry_ly, rect_idx) =
            find_entry_on_client_rect(layout_x, layout_y, &client.layout_rects);

        // Convert from layout space to client's local display coords
        if rect_idx < client.layout_rects.len() && rect_idx < client.screens.len() {
            let lr = &client.layout_rects[rect_idx];
            let ds = &client.screens[rect_idx];
            let frac_x = if lr.w > 0.0 {
                (entry_lx - lr.x) / lr.w
            } else {
                0.5
            };
            let frac_y = if lr.h > 0.0 {
                (entry_ly - lr.y) / lr.h
            } else {
                0.5
            };
            let display_x = ds.x as f64 + frac_x * ds.width as f64;
            let display_y = ds.y as f64 + frac_y * ds.height as f64;
            Some((display_x, display_y))
        } else if let Some(ds) = client.screens.first() {
            Some((
                ds.x as f64 + ds.width as f64 * 0.5,
                ds.y as f64 + ds.height as f64 * 0.5,
            ))
        } else {
            None
        }
    }
}

/// Map a point in local display coordinates to layout space.
/// Finds which local display contains the point, then maps via the layout rect.
pub(crate) fn local_display_to_layout(
    display_x: f64,
    display_y: f64,
    local_screens: &[DisplayInfo],
    local_layout_rects: &[LayoutRect],
) -> (f64, f64) {
    for (i, screen) in local_screens.iter().enumerate() {
        let sx = screen.x as f64;
        let sy = screen.y as f64;
        let sw = screen.width as f64;
        let sh = screen.height as f64;
        // Check if point is within (or at the edge of) this display
        if display_x >= sx - 1.0
            && display_x <= sx + sw + 1.0
            && display_y >= sy - 1.0
            && display_y <= sy + sh + 1.0
        {
            if let Some(lr) = local_layout_rects.get(i) {
                let frac_x = if sw > 0.0 { (display_x - sx) / sw } else { 0.5 };
                let frac_y = if sh > 0.0 { (display_y - sy) / sh } else { 0.5 };
                return (lr.x + frac_x * lr.w, lr.y + frac_y * lr.h);
            }
        }
    }
    // fallback: use first screen or return as-is
    if let (Some(screen), Some(lr)) = (local_screens.first(), local_layout_rects.first()) {
        let frac_x = if screen.width > 0 {
            (display_x - screen.x as f64) / screen.width as f64
        } else {
            0.5
        };
        let frac_y = if screen.height > 0 {
            (display_y - screen.y as f64) / screen.height as f64
        } else {
            0.5
        };
        (lr.x + frac_x * lr.w, lr.y + frac_y * lr.h)
    } else {
        (display_x, display_y)
    }
}

/// Find the entry point on the nearest client rect edge given a layout-space point.
/// Returns (entry_x, entry_y, rect_index).
pub(crate) fn find_entry_on_client_rect(
    lx: f64,
    ly: f64,
    client_rects: &[LayoutRect],
) -> (f64, f64, usize) {
    if client_rects.is_empty() {
        return (lx, ly, 0);
    }

    // Find the nearest client rect to the layout point
    let mut best_idx = 0;
    let mut best_dist = f64::MAX;
    for (i, r) in client_rects.iter().enumerate() {
        let nearest_x = lx.clamp(r.x, r.x + r.w);
        let nearest_y = ly.clamp(r.y, r.y + r.h);
        let dx = lx - nearest_x;
        let dy = ly - nearest_y;
        let dist = dx * dx + dy * dy;
        if dist < best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }

    let r = &client_rects[best_idx];
    // Clamp the layout point to just inside the client rect
    let entry_x = lx.clamp(r.x, r.x + r.w - 1.0);
    let entry_y = ly.clamp(r.y, r.y + r.h - 1.0);

    (entry_x, entry_y, best_idx)
}
