use crate::state::board::PieceKind;
use crate::state::game::{GamePhase, GameState, Interaction, PlayerColor};
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Instant;

mod state;
mod ui;

use crate::ui::board::BoardRenderer;
use crate::ui::promotion::PromotionRenderer;
use crate::ui::text::{FontRenderer, TextRenderer};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() -> Result<(), String> {
    println!("Instantiating Game!");

    let context = sdl2::init()?;
    let ttf_context = sdl2::ttf::init().expect("Failed to initialize SDL TTF Module!");
    let text_font = ttf_context.load_font("./assets/font/roboto/Roboto-Regular.ttf", 24)?;
    let chess_font = ttf_context.load_font("./assets/font/Chess/CHEQ_TT.TTF", 128)?;

    let video_subsystem = context.video()?;

    let mut window = video_subsystem
        .window("Voideds Chess?", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .expect("Failed to create the window!");

    window.set_icon(
        chess_font
            .render(PieceKind::King.to_ttf(PlayerColor::White))
            .blended(Color::BLACK)
            .expect("Failed to generate window icon!"),
    );

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("Failed to create the canvas!");

    let chess_font_renderer = FontRenderer::new(&chess_font);
    let text_font_renderer = FontRenderer::new(&text_font);

    let board_ui = BoardRenderer::new(
        Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT),
        &chess_font_renderer,
    );
    let promotion_ui = PromotionRenderer::new(
        Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT),
        &chess_font_renderer,
        &text_font_renderer,
    );

    let mut event_queue = context
        .event_pump()
        .expect("Failed to fetch the event queue!");

    let mut game_state = GameState::new();

    let fps_label = TextRenderer::new(
        &text_font_renderer,
        Rect::new(SCREEN_WIDTH as i32 - 100, 0, 100, 32),
    );
    let current_player = TextRenderer::new(
        &text_font_renderer,
        Rect::new(SCREEN_WIDTH as i32 - 100, 32, 100, 32),
    );

    let mut now = Instant::now();
    let mut fps = 0.0;
    let mut frames = 0;
    let mut mouse_position = (0, 0);

    'game_loop: loop {
        frames += 1;
        for event in event_queue.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'game_loop;
                }
                Event::MouseMotion { x, y, .. } => {
                    mouse_position = (x, y);
                }
                Event::MouseButtonDown {
                    x,
                    y,
                    mouse_btn: MouseButton::Left,
                    ..
                } => match game_state.interaction() {
                    Some(Interaction::PickingPromotion(_, _, choices)) => {
                        if let Some(piece) = promotion_ui.mouse_is_over(x, y, choices) {
                            game_state = game_state.interact(Interaction::PickedPromotion(piece));
                        }
                    }
                    _ => {
                        if let Some(position) = board_ui.mouse_is_over(x, y) {
                            game_state =
                                game_state.interact(Interaction::StartMovingPiece(position));
                        }
                    }
                },
                Event::MouseButtonUp {
                    x,
                    y,
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    if let Some(position) = board_ui.mouse_is_over(x, y) {
                        game_state = game_state.interact(Interaction::PlacedPiece(position));
                    }
                }
                _ => {}
            }
        }

        let elapsed = now.elapsed().as_secs_f64();

        if elapsed >= 0.1 {
            fps = frames as f64 / elapsed;
            now = Instant::now();
            frames = 0;
        }

        board_ui.render(&mut canvas, mouse_position, &game_state)?;
        if let Some(Interaction::PickingPromotion(_, _, choices)) = game_state.interaction() {
            promotion_ui.render(&mut canvas, choices, mouse_position)?;
        }

        fps_label.render(format!("FPS: {:.0}", fps).as_str(), &mut canvas)?;
        match game_state.phase() {
            GamePhase::Won(p) => {
                current_player.render(format!("Winner: {:?}", p).as_str(), &mut canvas)?
            }
            GamePhase::Draw(reason) => {
                current_player.render(format!("Draw: {:?}", reason).as_str(), &mut canvas)?
            }
            GamePhase::Turn(p) => {
                current_player.render(format!("Turn: {:?}", p).as_str(), &mut canvas)?
            }
        };

        canvas.present();
    }

    println!("Game Terminated!");
    Ok(())
}
