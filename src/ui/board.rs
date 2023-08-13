use crate::state::board::PieceKind;
use crate::state::game::{GameState, Interaction, Move, PlayerColor, Position};
use crate::ui::text::FontRenderer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, WindowCanvas};
use std::cmp::min;

const BOARD_SIZE: i32 = 8;
pub const COLOR_WHITE: Color = Color::RGB(223, 158, 69);
pub const COLOR_BLACK: Color = Color::RGB(140, 36, 11);
pub const CLEAR_COLOR: Color = Color::RGB(192, 192, 192);
pub const SELECTED_HIGHLIGHT_COLOR: Color = Color::RGBA(255, 255, 255, 50);

pub struct BoardRenderer<'ttf_module, 'rwops> {
    screen_area: Rect,
    font_renderer: &'ttf_module FontRenderer<'ttf_module, 'rwops>,
}

impl<'ttf_module, 'rwops> BoardRenderer<'ttf_module, 'rwops> {
    pub fn new(
        screen_area: Rect,
        text_renderer: &'ttf_module FontRenderer<'ttf_module, 'rwops>,
    ) -> Self {
        Self {
            screen_area,
            font_renderer: text_renderer,
        }
    }
}

impl<'ttf_module, 'rwops> BoardRenderer<'ttf_module, 'rwops> {
    pub fn render(
        &self,
        canvas: &mut WindowCanvas,
        mouse_position: (i32, i32),
        state: &GameState,
    ) -> Result<(), String> {
        // clear board
        canvas.set_draw_color(CLEAR_COLOR);
        canvas.fill_rect(self.screen_area)?;

        let hovering = self.mouse_is_over(mouse_position.0, mouse_position.1);

        let screen_small = min(self.screen_area.width(), self.screen_area.height()) as i32;
        let cell_width = screen_small / BOARD_SIZE;
        let offset_x = (self.screen_area.width() as i32 - screen_small) / 2;
        let offset_y = (self.screen_area.height() as i32 - screen_small) / 2;

        let active_pos = if let Some(Interaction::StartMovingPiece(p)) = state.interaction() {
            Some(p)
        } else {
            None
        };
        let active_piece = active_pos.map(|x| state.piece_at(*x)).flatten();

        // draw board
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let rect = Rect::new(
                    offset_x + x * cell_width,
                    offset_y + y * cell_width,
                    cell_width as u32,
                    cell_width as u32,
                );

                if x % 2 == y % 2 {
                    canvas.set_draw_color(COLOR_WHITE);
                } else {
                    canvas.set_draw_color(COLOR_BLACK);
                }

                canvas.fill_rect(rect)?;

                if let Some(active_pos) = active_pos {
                    if let Some(active_piece) = active_piece {
                        for valid_move in active_piece.moves(*active_pos, state) {
                            match valid_move {
                                Move::Move(to) => {
                                    Self::highlight_if_position_match(canvas, to, x, y, rect)?;
                                }
                                Move::Take(to, _) => {
                                    Self::highlight_if_position_match(canvas, to, x, y, rect)?;
                                }
                                Move::Promote(to, _) => {
                                    Self::highlight_if_position_match(canvas, to, x, y, rect)?;
                                }
                                Move::Castle(side) => {
                                    Self::highlight_if_position_match(
                                        canvas,
                                        side.positions(active_piece.color).king_end,
                                        x,
                                        y,
                                        rect,
                                    )?;
                                }
                            }
                        }
                    }
                }

                if let Some(position) = hovering {
                    Self::highlight_if_position_match(canvas, position, x, y, rect)?;
                }

                if let Some(piece) = state.piece_at(Position(x, y)) {
                    // drawing pieces using ttf
                    self.font_renderer
                        .render_at(piece.kind.to_ttf(piece.color), rect, canvas)?;
                }
            }
        }

        Ok(())
    }

    fn highlight_if_position_match(
        canvas: &mut WindowCanvas,
        Position(s_x, s_y): Position,
        x: i32,
        y: i32,
        rect: Rect,
    ) -> Result<(), String> {
        if s_x == x && s_y == y {
            let blend_mode = canvas.blend_mode();
            canvas.set_blend_mode(BlendMode::Mul);
            canvas.set_draw_color(SELECTED_HIGHLIGHT_COLOR);
            canvas.fill_rect(rect)?;
            canvas.set_blend_mode(blend_mode);
        }
        Ok(())
    }

    pub fn mouse_is_over(&self, mouse_x: i32, mouse_y: i32) -> Option<Position> {
        let screen_small = min(self.screen_area.width(), self.screen_area.height()) as i32;
        let cell_width = screen_small / BOARD_SIZE;
        let offset_x = (self.screen_area.width() as i32 - screen_small) / 2;
        let offset_y = (self.screen_area.height() as i32 - screen_small) / 2;

        if mouse_x < offset_x || mouse_y < offset_y {
            return None;
        }

        let x = (mouse_x - offset_x) / cell_width;
        let y = (mouse_y - offset_y) / cell_width;
        if x >= BOARD_SIZE || y >= BOARD_SIZE {
            None
        } else {
            Some(Position(x, y))
        }
    }
}

impl PieceKind {
    pub fn to_ttf(&self, color: PlayerColor) -> &'static str {
        match self {
            PieceKind::Pawn if color == PlayerColor::White => "p",
            PieceKind::Rook if color == PlayerColor::White => "r",
            PieceKind::Knight if color == PlayerColor::White => "h",
            PieceKind::Bishop if color == PlayerColor::White => "b",
            PieceKind::King if color == PlayerColor::White => "k",
            PieceKind::Queen if color == PlayerColor::White => "q",
            PieceKind::Pawn => "o",
            PieceKind::Rook => "t",
            PieceKind::Knight => "j",
            PieceKind::Bishop => "n",
            PieceKind::King => "l",
            PieceKind::Queen => "w",
        }
    }
}
