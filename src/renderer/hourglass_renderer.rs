use crate::{renderer::StyledRenderer, Progress};
use anyhow::Result;
use crossterm::style::{Color, PrintStyledContent, Stylize};
use crossterm::{cursor::MoveTo, queue};
use std::io::Write;
use std::sync::OnceLock;
use std::time::Instant;

// Fixed inner width of the hourglass content (between the side borders)
const INNER_WIDTH: usize = 9;
// Drawing characters (customize here)
const CH_BORDER_TL: char = '┏';
const CH_BORDER_TR: char = '┓';
const CH_BORDER_BL: char = '┗';
const CH_BORDER_BR: char = '┛';
const CH_BORDER_H: char = '━';
const CH_BORDER_V: char = '┃';

const CH_SAND: char = '█';
const CH_EMPTY: char = '░';

// const CH_FLOW_MAIN: char = '┋';
const CH_FLOW_MAIN: char = '┊';
const CH_FLOW_TRAIL: char = '┊';

const CH_SPACE: char = ' ';

const SEP_ARROW: &str = "→";
const INFO_DIVIDER: char = '|';

// Top/bottom reservoir row counts for the hourglass box interior
// Ensure total movable sand capacity is 60 cells.
// Top: 6 rows x 9 + joins (5+1) = 54 + 6 = 60
// Bottom: joins (1+5) + 6 rows x 9 = 6 + 54 = 60 (neck is not counted)
const TOP_ROWS: usize = 6;
const BOTTOM_ROWS: usize = 6;

pub struct HourglassRenderer {
    title: Option<String>,
    progress: Progress,
}

static START_INSTANT: OnceLock<Instant> = OnceLock::new();

impl StyledRenderer for HourglassRenderer {
    fn new(title: Option<String>, progress: Progress) -> Self {
        HourglassRenderer { title, progress }
    }

    fn render<W: Write>(&self, w: &mut W) -> Result<u16> {
        let title = self.build_title();
        let mut row = if let Some(title) = title {
            Self::render_content(w, &title, 0)?
        } else {
            0
        };

        // Build header/footer first to decide which divider to center on
        let header = self.build_information();
        let footer = self.build_footer();

        let header_divider_col = header
            .chars()
            .enumerate()
            .find(|(_, c)| *c == INFO_DIVIDER)
            .map(|(i, _)| i)
            .unwrap_or(0);
        let footer_divider_col = footer
            .chars()
            .enumerate()
            .find(|(_, c)| *c == INFO_DIVIDER)
            .map(|(i, _)| i)
            .unwrap_or(0);

        // Decide anchor by the farther-right divider position (prefer footer on tie)
        let anchor_col = if header_divider_col > footer_divider_col {
            header_divider_col
        } else {
            footer_divider_col
        };

        // Compute base center of hourglass (including left border)
        let base_center = 1 + (INNER_WIDTH / 2);
        let left_pad = anchor_col.saturating_sub(base_center);
        let pad = CH_SPACE.to_string().repeat(left_pad);

        // Render header padded so its '|' aligns to anchor
        let header_left_pad = (left_pad + base_center).saturating_sub(header_divider_col);
        let header_pad = CH_SPACE.to_string().repeat(header_left_pad);
        let header_padded = format!("{}{}", header_pad, header);
        row = Self::render_content(w, &header_padded, row)?;

        // Render the hourglass box
        for line in self.build_hourglass() {
            queue!(
                w,
                MoveTo(0, row),
                PrintStyledContent(format!("{}{}", pad, line).with(Color::Reset))
            )?;
            row += 1;
        }

        // Footer: pad so its '|' aligns to the same anchor
        let footer_left_pad = (left_pad + base_center).saturating_sub(footer_divider_col);
        let footer_pad = CH_SPACE.to_string().repeat(footer_left_pad);
        let footer_padded = format!("{}{}", footer_pad, footer);
        row = Self::render_content(w, &footer_padded, row)?;
        Ok(row)
    }
}

impl HourglassRenderer {
    fn build_title(&self) -> Option<String> {
        self.title.clone()
    }

    fn build_information(&self) -> String {
        let from = self.progress.timespan.format_from();
        let to = self.progress.timespan.format_to();
        let ratio = format!("{:.0}%", self.progress.ratio * 100.0);
        let space = CH_SPACE.to_string().repeat(3);
        [format!("{from} {SEP_ARROW} {to}"), ratio]
            .join(format!("{space}{}{space}", INFO_DIVIDER).as_str())
    }

    fn build_footer(&self) -> String {
        let space = CH_SPACE.to_string().repeat(3);
        let elapsed = format!("elapsed: {}", self.progress.format_elapsed());
        let remaining = format!("remaining: {}", self.progress.format_remaining());
        [elapsed, remaining].join(format!("{space}{}{space}", INFO_DIVIDER).as_str())
    }

    // Build full box (top border, interior lines, bottom border)
    // Interior lines = TOP_ROWS + 2 (top joins) + 1 (neck) + 2 (bottom joins) + BOTTOM_ROWS
    fn build_hourglass(&self) -> Vec<String> {
        let start = START_INSTANT.get_or_init(Instant::now);
        let elapsed = start.elapsed();
        let flow_active = !self.progress.is_complete();

        // Compute total capacity (movable sand cells) = 60
        // Done after defining `top_fun_widths` for clarity.

        // Prepare structures
        // Top reservoir (display order: top->bottom)
        let mut top_res: Vec<Vec<char>> =
            (0..TOP_ROWS).map(|_| vec![CH_SAND; INNER_WIDTH]).collect();
        // Top funnel (new appearance): two join lines with inner widths 5 and 1
        let top_fun_widths = [5usize, 1];
        let mut top_fun: Vec<Vec<char>> =
            top_fun_widths.iter().map(|&w| vec![CH_SAND; w]).collect();

        // Compute total capacity (movable sand cells) from top geometry
        let total_cells = TOP_ROWS * INNER_WIDTH + top_fun_widths.iter().sum::<usize>(); // 54 + 6 = 60

        // Bottom reservoir (display order: top->bottom)
        let mut bot_res: Vec<Vec<char>> = (0..BOTTOM_ROWS)
            .map(|_| vec![CH_EMPTY; INNER_WIDTH])
            .collect();
        // Bottom funnel (new appearance): two join lines with inner widths 1 and 5
        let bot_fun_widths = [1usize, 5];
        let mut bot_fun: Vec<Vec<char>> =
            bot_fun_widths.iter().map(|&w| vec![CH_EMPTY; w]).collect();
        // Neck cell
        let mut neck = CH_EMPTY;

        // Helper: center-out order indices for a given width
        let center_order = |w: usize| -> Vec<usize> {
            let mid = w / 2;
            let mut idx = vec![mid];
            let mut d = 1;
            loop {
                let mut pushed = false;
                if mid >= d {
                    idx.push(mid - d);
                    pushed = true;
                }
                if mid + d < w {
                    idx.push(mid + d);
                    pushed = true;
                }
                if !pushed {
                    break;
                }
                d += 1;
            }
            idx
        };

        // Build bottom fill order (reservoir bottom->top, then funnel bottom->top)
        // Note: neck is excluded from capacity so total is exactly 60.
        let mut bottom_coords: Vec<(u8, usize, usize)> = Vec::new();
        // Reservoir rows: bottom-most first
        for r in (0..BOTTOM_ROWS).rev() {
            for c in center_order(INNER_WIDTH) {
                bottom_coords.push((0, r, c)); // 0 = bot_res
            }
        }
        // Funnel rows: bottom-most first (width 5), then (width 1)
        for (i, &w) in bot_fun_widths.iter().enumerate().rev() {
            for c in center_order(w) {
                bottom_coords.push((1, i, c)); // 1 = bot_fun
            }
        }
        // total_cells is fixed from top geometry (60)
        let filled = (self.progress.ratio.clamp(0.0, 1.0) * total_cells as f64).round() as usize;

        // Apply bottom filled cells according to progress
        for (k, &(section, i, j)) in bottom_coords.iter().enumerate() {
            if k >= filled {
                break;
            }
            match section {
                0 => bot_res[i][j] = CH_SAND,
                1 => bot_fun[i][j] = CH_SAND,
                _ => {}
            }
        }

        // At 100% completion, keep the neck visually open (empty)
        if self.progress.is_complete() {
            neck = CH_EMPTY;
        }

        // Build top empty order (reservoir top->down first, then funnel top side)
        let mut top_coords: Vec<(u8, usize, usize)> = Vec::with_capacity(total_cells);
        // Reservoir rows: top-most first => 0..4
        for r in 0..TOP_ROWS {
            for c in center_order(INNER_WIDTH) {
                top_coords.push((4, r, c)); // 4 = top_res
            }
        }
        // Funnel rows: top side first in new appearance => widths 5, then 1
        for i in 0..top_fun_widths.len() {
            let w = top_fun_widths[i];
            for c in center_order(w) {
                top_coords.push((3, i, c)); // 3 = top_fun
            }
        }
        // Apply top emptied cells equal to bottom filled cells
        for (k, &(section, i, j)) in top_coords.iter().enumerate() {
            if k >= filled {
                break;
            }
            match section {
                3 => top_fun[i][j] = CH_EMPTY,
                _ => top_res[i][j] = CH_EMPTY,
            }
        }

        // Moving droplet animation using '┋' (active) and '┊' (trail)
        // Build path: neck (1) -> lower funnel (4) -> lower reservoir (4)
        let mut neck_char = neck;
        if flow_active {
            let mut path: Vec<(u8, usize)> = Vec::with_capacity(1 + bot_fun.len() + bot_res.len());
            // 2 = neck, use index 0 placeholder
            path.push((2, 0));
            // 1 = bot_fun (top->bottom)
            for i in 0..bot_fun.len() {
                path.push((1, i));
            }
            // 0 = bot_res (top->bottom)
            for i in 0..bot_res.len() {
                path.push((0, i));
            }

            // Determine active path length (stop before first sand in center)
            let mut active_len = 0usize;
            for (sec, idx) in &path {
                let occupied = match *sec {
                    2 => neck_char == CH_SAND,
                    1 => {
                        let w = bot_fun[*idx].len();
                        bot_fun[*idx][w / 2] == CH_SAND
                    }
                    _ => bot_res[*idx][INNER_WIDTH / 2] == CH_SAND,
                };
                if occupied {
                    break;
                }
                active_len += 1;
            }

            if active_len > 0 {
                let step = ((elapsed.as_millis() / 500) as usize) % active_len;
                let main_ch = CH_FLOW_MAIN;
                let trail_ch = CH_FLOW_TRAIL;

                for (i, (sec, idx)) in path.iter().take(active_len).enumerate() {
                    let is_main = i == step;
                    match *sec {
                        2 => neck_char = if is_main { main_ch } else { trail_ch },
                        1 => {
                            let w = bot_fun[*idx].len();
                            let mid = w / 2;
                            bot_fun[*idx][mid] = if is_main { main_ch } else { trail_ch };
                        }
                        _ => {
                            let mid = INNER_WIDTH / 2;
                            bot_res[*idx][mid] = if is_main { main_ch } else { trail_ch };
                        }
                    }
                }
            }
        }

        // Compose output lines
        let mut lines: Vec<String> = Vec::with_capacity(20);
        lines.push(Self::top_border());
        // Top reservoir (display order)
        for r in 0..TOP_ROWS {
            lines.push(Self::boxed(top_res[r].iter().collect()));
        }
        // Top funnel join lines (widths 5, then 1)
        for (i, row) in top_fun.iter().enumerate() {
            lines.push(Self::join_line_top(row, i));
        }
        // Neck
        lines.push(Self::funnel_line(4, neck_char, 1));
        // Lower funnel join lines (widths 1, then 5)
        for (i, row) in bot_fun.iter().enumerate() {
            lines.push(Self::join_line_bottom(row, i));
        }
        // Lower reservoir (display order)
        for r in 0..BOTTOM_ROWS {
            lines.push(Self::boxed(bot_res[r].iter().collect()));
        }
        lines.push(Self::bottom_border());

        lines
    }

    // Draw a boxed line with side borders
    fn boxed(inner: String) -> String {
        format!("{}{}{}", CH_BORDER_V, inner, CH_BORDER_V)
    }

    // Upper/lower funnel line with fixed char repeated `width` and left indentation
    fn funnel_line(indent: usize, ch: char, width: usize) -> String {
        let inner: String = std::iter::repeat(ch).take(width).collect();
        format!(
            "{}{}{}{}",
            CH_SPACE.to_string().repeat(indent),
            CH_BORDER_V,
            inner,
            CH_BORDER_V
        )
    }

    // Join line for the top funnel section using corner connectors
    // i = 0 => width 5 with indent 0; i = 1 => width 1 with indent 2
    fn join_line_top(inner: &[char], i: usize) -> String {
        let w = inner.len();
        let total = INNER_WIDTH + 2; // full interior width of the hourglass area
        let left_indent = if i == 0 { 0 } else { 2 };
        let used = left_indent + 3 + w + 3; // left + "┗━┓" + inner + "┏━┛"
        let right_pad = total.saturating_sub(used);
        let inner_s: String = inner.iter().collect();
        format!(
            "{}┗━┓{}┏━┛{}",
            CH_SPACE.to_string().repeat(left_indent),
            inner_s,
            CH_SPACE.to_string().repeat(right_pad)
        )
    }

    // Join line for the bottom funnel section using corner connectors
    // i = 0 => width 1 with indent 2; i = 1 => width 5 with indent 0
    fn join_line_bottom(inner: &[char], i: usize) -> String {
        let w = inner.len();
        let total = INNER_WIDTH + 2;
        let left_indent = if i == 0 { 2 } else { 0 };
        let used = left_indent + 3 + w + 3; // left + "┏━┛" + inner + "┗━┓"
        let right_pad = total.saturating_sub(used);
        let inner_s: String = inner.iter().collect();
        format!(
            "{}┏━┛{}┗━┓{}",
            CH_SPACE.to_string().repeat(left_indent),
            inner_s,
            CH_SPACE.to_string().repeat(right_pad)
        )
    }

    fn top_border() -> String {
        format!(
            "{}{}{}",
            CH_BORDER_TL,
            CH_BORDER_H.to_string().repeat(INNER_WIDTH),
            CH_BORDER_TR
        )
    }

    fn bottom_border() -> String {
        format!(
            "{}{}{}",
            CH_BORDER_BL,
            CH_BORDER_H.to_string().repeat(INNER_WIDTH),
            CH_BORDER_BR
        )
    }
}
