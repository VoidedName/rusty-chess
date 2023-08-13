use crate::state::game::Piece;
use crate::ui::board::{CLEAR_COLOR, SELECTED_HIGHLIGHT_COLOR};
use crate::ui::text::FontRenderer;
use crate::SCREEN_WIDTH;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, WindowCanvas};
use std::cmp::min;

pub struct PromotionRenderer<'ttf_module, 'rwops> {
    render_area: Rect,
    chess_renderer: &'ttf_module FontRenderer<'ttf_module, 'rwops>,
    text_renderer: &'ttf_module FontRenderer<'ttf_module, 'rwops>,
}

impl<'ttf_module, 'rwops> PromotionRenderer<'ttf_module, 'rwops> {
    pub fn new(
        render_area: Rect,
        chess_renderer: &'ttf_module FontRenderer<'ttf_module, 'rwops>,
        text_renderer: &'ttf_module FontRenderer<'ttf_module, 'rwops>,
    ) -> Self {
        Self {
            render_area,
            chess_renderer,
            text_renderer,
        }
    }
}

impl<'ttf_module, 'rwops> PromotionRenderer<'ttf_module, 'rwops> {
    pub fn render(
        &self,
        canvas: &mut WindowCanvas,
        options: &Vec<Piece>,
        (mouse_x, mouse_y): (i32, i32),
    ) -> Result<(), String> {
        canvas.set_draw_color(CLEAR_COLOR);
        canvas.fill_rect(self.render_area)?;

        canvas.set_draw_color(Color::BLACK);

        for (tile, p) in self.tiles(options) {
            canvas.draw_rect(tile)?;

            if Self::is_point_in_rect(mouse_x, mouse_y, tile) {
                let mode = canvas.blend_mode();
                canvas.set_blend_mode(BlendMode::Mul);
                canvas.set_draw_color(SELECTED_HIGHLIGHT_COLOR);
                canvas.fill_rect(tile)?;
                canvas.set_blend_mode(mode);
                canvas.set_draw_color(Color::BLACK);
            }

            self.chess_renderer
                .render_at(p.kind.to_ttf(p.color), tile, canvas)?;
        }

        self.text_renderer.render_at(
            "Promote Pawn",
            Rect::new((SCREEN_WIDTH - 200) as i32 / 2, 100, 200, 100),
            canvas,
        )?;
        Ok(())
    }

    fn tiles(&self, options: &Vec<Piece>) -> Vec<(Rect, Piece)> {
        let screen_small = min(self.render_area.width(), self.render_area.height()) as i32;
        let width = min(screen_small / options.len() as i32, 128);

        let offset_x = (self.render_area.width() as i32 - width * options.len() as i32) / 2;
        let offset_y = (self.render_area.height() as i32 - width) / 2;

        options
            .iter()
            .enumerate()
            .map(|(idx, piece)| {
                (
                    Rect::new(
                        offset_x + (idx as i32 * width),
                        offset_y,
                        width as u32,
                        width as u32,
                    ),
                    *piece,
                )
            })
            .collect()
    }

    pub fn mouse_is_over(&self, x: i32, y: i32, choices: &Vec<Piece>) -> Option<Piece> {
        self.tiles(choices)
            .into_iter()
            .find(|(tile, _)| Self::is_point_in_rect(x, y, *tile))
            .map(|(_, p)| p)
    }

    fn is_point_in_rect(mouse_x: i32, mouse_y: i32, tile: Rect) -> bool {
        mouse_x > tile.x
            && mouse_x < tile.x + tile.w
            && mouse_y > tile.y
            && mouse_y < tile.y + tile.h
    }
}
