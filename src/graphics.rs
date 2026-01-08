use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Rectangle,
    text::Text,
};
use log::info;

// Background / base tones
pub const BASE03: Rgb565 = Rgb565::new(0, 11, 7); // #002b36
pub const BASE02: Rgb565 = Rgb565::new(1, 13, 8); // #073642
pub const BASE01: Rgb565 = Rgb565::new(11, 27, 14); // #586e75
pub const BASE00: Rgb565 = Rgb565::new(12, 30, 16); // #657b83
pub const BASE0: Rgb565 = Rgb565::new(16, 37, 18); // #839496
pub const BASE1: Rgb565 = Rgb565::new(18, 40, 20); // #93a1a1
pub const BASE2: Rgb565 = Rgb565::new(29, 57, 26); // #eee8d5
pub const BASE3: Rgb565 = Rgb565::new(31, 61, 28); // #fdf6e3

// Accent colors
pub const YELLOW: Rgb565 = Rgb565::new(22, 34, 0); // #b58900
pub const ORANGE: Rgb565 = Rgb565::new(25, 19, 3); // #cb4b16
pub const RED: Rgb565 = Rgb565::new(27, 12, 6); // #dc322f
pub const MAGENTA: Rgb565 = Rgb565::new(26, 13, 16); // #d33682
pub const VIOLET: Rgb565 = Rgb565::new(13, 28, 24); // #6c71c4
pub const BLUE: Rgb565 = Rgb565::new(5, 34, 26); // #268bd2
pub const CYAN: Rgb565 = Rgb565::new(5, 40, 18); // #2aa198
pub const GREEN: Rgb565 = Rgb565::new(16, 38, 0); // #859900

pub const CELL_W: u16 = 6;
pub const CELL_H: u16 = 10;

#[derive(Copy, Clone)]
pub struct Cell {
    pub ch: char,
    pub fg: Rgb565,
    pub bg: Rgb565,
    pub dirty: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Rgb565::BLACK,
            bg: Rgb565::BLACK,
            dirty: true,
        }
    }
}

pub struct ScreenGrid<'a> {
    pub cols: u16,
    pub rows: u16,
    pub cells: &'a mut [Cell],
}

impl<'a> ScreenGrid<'a> {
    pub fn new(cols: u16, rows: u16, cells: &'a mut [Cell]) -> Self {
        // caller ensures cells.len() == cols as usize * rows as usize
        Self { cols, rows, cells }
    }

    // Get cell index based on grid position
    fn idx(&self, x: u16, y: u16) -> usize {
        (y as usize) * (self.cols as usize) + (x as usize)
    }

    pub fn clear(&mut self, ch: char, fg: Rgb565, bg: Rgb565) {
        for cell in self.cells.iter_mut() {
            *cell = Cell {
                ch,
                fg,
                bg,
                dirty: cell.ch != ch || cell.fg != fg || cell.bg != bg,
            };
        }
    }

    pub fn put_char(&mut self, x: u16, y: u16, ch: char, fg: Rgb565, bg: Rgb565) {
        if x < self.cols && y < self.rows {
            let cell = &mut self.cells[self.idx(x, y)];
            if cell.ch != ch || cell.fg != fg || cell.bg != bg {
                *cell = Cell {
                    ch,
                    fg,
                    bg,
                    dirty: true,
                };
            }
        }
    }

    pub fn write_str(&mut self, x: u16, y: u16, s: &str, fg: Rgb565, bg: Rgb565) {
        for (i, ch) in s.chars().enumerate() {
            let xi = x + i as u16;
            if xi >= self.cols {
                break;
            }
            self.put_char(xi, y, ch, fg, bg);
        }
    }

    pub fn draw_box(&mut self, x: u16, y: u16, width: u16, height: u16, bg: Rgb565) {
        for x in x..x + width {
            for y in y..y + height {
                self.put_char(x, y, ' ', bg, bg);
            }
        }
    }
}

pub fn screen_pos_to_grid_pos(x: u16, y: u16) -> (u16, u16) {
    (
        if x > 0 { x / CELL_W } else { 0 },
        if y > 0 { y / CELL_H } else { 0 },
    )
}

pub fn draw_status_bars(grid: &mut ScreenGrid, app_name: &str, render_time: u64) {
    // Title bar
    grid.draw_box(0, 0, 40, 2, BASE01);
    grid.write_str((40 - app_name.len()) as u16 / 2, 0, app_name, BASE3, BASE01);

    // Status bar
    grid.draw_box(0, 31, 40, 1, CYAN);
    grid.write_str(
        0,
        31,
        &heapless::format!(12; "Render: {}ms", render_time).unwrap_or_default(),
        BASE2,
        CYAN,
    );
}

pub fn render_grid<D: DrawTarget<Color = Rgb565>>(
    display: &mut D,
    grid: &mut ScreenGrid,
) -> Result<(), D::Error> {
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            let cell = grid.cells[grid.idx(x, y)];

            if cell.dirty {
                let x_px = (x * CELL_W) as i32;
                let y_px = (y * CELL_H) as i32;

                // Draw background
                Rectangle::new(
                    Point::new(x_px, y_px),
                    Size::new(CELL_W as u32, CELL_H as u32),
                )
                .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_fill(
                    cell.bg,
                ))
                .draw(display)?;

                // Draw character
                if cell.ch != ' ' {
                    let style = MonoTextStyle::new(&FONT_6X10, cell.fg);

                    let mut buf = [0u8; 4]; // a char can be up to 4 UTF-8 bytes
                    let s = cell.ch.encode_utf8(&mut buf);

                    Text::new(s, Point::new(x_px, y_px + FONT_6X10.baseline as i32), style)
                        .draw(display)?;
                }

                grid.cells[grid.idx(x, y)].dirty = false;
            }
        }
    }
    Ok(())
}
