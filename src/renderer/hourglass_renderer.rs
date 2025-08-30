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

// UI symbols (non-structural)
const ICON_HOURGLASS: &str = "⏳";
const SEP_ARROW: &str = "→";
const INFO_DIVIDER: char = '|';

// Top reservoir rows and bottom reservoir rows (matches the sample frames)
const TOP_ROWS: usize = 5;
const BOTTOM_ROWS: usize = 4;

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

        let header = self.build_information();
        row = Self::render_content(w, &header, row)?;

        // Build footer first to compute alignment target (position of divider '|')
        let footer = self.build_footer();
        let divider_col = footer
            .chars()
            .enumerate()
            .find(|(_, c)| *c == INFO_DIVIDER)
            .map(|(i, _)| i)
            .unwrap_or(0);
        let base_center = 1 + (INNER_WIDTH / 2); // center index of full-width hourglass line
        let left_pad = divider_col.saturating_sub(base_center);
        let pad = CH_SPACE.to_string().repeat(left_pad);

        // Render the hourglass box
        for line in self.build_hourglass() {
            queue!(
                w,
                MoveTo(0, row),
                PrintStyledContent(format!("{}{}", pad, line).with(Color::Reset))
            )?;
            row += 1;
        }

        row = Self::render_content(w, &footer, row)?;
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


    // Build full box (top border, interior 18 lines, bottom border)
    fn build_hourglass(&self) -> Vec<String> {
        let start = START_INSTANT.get_or_init(Instant::now);
        let elapsed = start.elapsed();
        let flow_active = !self.progress.is_complete();

        // Total capacity per half (excluding borders): 61 cells
        // bottom: 1 (neck) + 4 lower funnel (3,5,7,9) + 4 reservoir (9x4) = 61
        // top:    4 upper funnel (1,3,5,7) + 5 reservoir (9x5) = 61
        let total_cells = 61usize;
        let filled = (self.progress.ratio.clamp(0.0, 1.0) * total_cells as f64).round() as usize;

        // Prepare structures
        // Top reservoir (display order: top->bottom)
        let mut top_res: Vec<Vec<char>> =
            (0..TOP_ROWS).map(|_| vec![CH_SAND; INNER_WIDTH]).collect();
        // Top funnel (display order: widths 7,5,3,1)
        let top_fun_widths = [7usize, 5, 3, 1];
        let mut top_fun: Vec<Vec<char>> =
            top_fun_widths.iter().map(|&w| vec![CH_SAND; w]).collect();

        // Bottom reservoir (display order: top->bottom)
        let mut bot_res: Vec<Vec<char>> = (0..BOTTOM_ROWS)
            .map(|_| vec![CH_EMPTY; INNER_WIDTH])
            .collect();
        // Bottom funnel (display order: widths 3,5,7,9)
        let bot_fun_widths = [3usize, 5, 7, 9];
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

        // Build bottom fill order (reservoir bottom->top, then funnel bottom->top, then neck)
        let mut bottom_coords: Vec<(u8, usize, usize)> = Vec::with_capacity(total_cells);
        // Reservoir rows: bottom-most first
        for r in (0..BOTTOM_ROWS).rev() {
            for c in center_order(INNER_WIDTH) {
                bottom_coords.push((0, r, c)); // 0 = bot_res
            }
        }
        // Funnel rows: bottom-most (width 9) to top-most (width 3)
        for (i, &w) in bot_fun_widths.iter().enumerate().rev() {
            for c in center_order(w) {
                bottom_coords.push((1, i, c)); // 1 = bot_fun
            }
        }
        // Neck
        bottom_coords.push((2, 0, 0)); // 2 = neck

        // Apply bottom filled cells according to progress
        for (k, &(section, i, j)) in bottom_coords.iter().enumerate() {
            if k >= filled {
                break;
            }
            match section {
                0 => bot_res[i][j] = CH_SAND,
                1 => bot_fun[i][j] = CH_SAND,
                _ => neck = CH_SAND,
            }
        }

        // Build top empty order (reservoir top->down first, then funnel top side)
        let mut top_coords: Vec<(u8, usize, usize)> = Vec::with_capacity(total_cells);
        // Reservoir rows: top-most first => 0..4
        for r in 0..TOP_ROWS {
            for c in center_order(INNER_WIDTH) {
                top_coords.push((4, r, c)); // 4 = top_res
            }
        }
        // Funnel rows: top side first => index 0 (width7), 1 (width5), 2 (width3), 3 (width1)
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
        // Top funnel (display order widths 7,5,3,1 with indents 1..4)
        for (i, row) in top_fun.iter().enumerate() {
            lines.push(Self::funnel_line_dyn(1 + i, row));
        }
        // Neck
        lines.push(Self::funnel_line(4, neck_char, 1));
        // Lower funnel (display order widths 3,5,7,9 with indents 3..0)
        for (i, row) in bot_fun.iter().enumerate() {
            let indent = 3usize.saturating_sub(i);
            lines.push(Self::funnel_line_dyn(indent, row));
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

    // Variant taking a prepared inner slice of chars
    fn funnel_line_dyn(indent: usize, inner: &[char]) -> String {
        let s: String = inner.iter().collect();
        format!(
            "{}{}{}{}",
            CH_SPACE.to_string().repeat(indent),
            CH_BORDER_V,
            s,
            CH_BORDER_V
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
