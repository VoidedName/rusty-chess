use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::ttf::Font;
use std::cmp::min;

pub struct FontRenderer<'ttf_module, 'rwops> {
    font: &'ttf_module Font<'ttf_module, 'rwops>,
}

pub struct TextRenderer<'ttf_module, 'rwops> {
    font_renderer: &'ttf_module FontRenderer<'ttf_module, 'rwops>,
    render_area: Rect,
}

impl<'ttf_module, 'rwops> FontRenderer<'ttf_module, 'rwops> {
    pub fn new(font: &'ttf_module Font<'ttf_module, 'rwops>) -> Self {
        Self { font }
    }
}

impl<'ttf_module, 'rwops> TextRenderer<'ttf_module, 'rwops> {
    pub fn new(
        font_renderer: &'ttf_module FontRenderer<'ttf_module, 'rwops>,
        render_area: Rect,
    ) -> Self {
        Self {
            font_renderer,
            render_area,
        }
    }
}

impl FontRenderer<'_, '_> {
    pub fn render_at(
        &self,
        text: &str,
        render_area: Rect,
        canvas: &mut WindowCanvas,
    ) -> Result<(), String> {
        let surface = self
            .font
            .render(text)
            .blended(Color::BLACK)
            .expect("Failed to render text!");

        let tc = canvas.texture_creator();
        let text = surface
            .as_texture(&tc)
            .expect("Failed to convert surface to texture!");

        canvas.copy(
            &text,
            Rect::new(0, 0, surface.width(), surface.height()),
            Rect::new(
                render_area.x,
                render_area.y,
                min(render_area.width(), surface.width()),
                min(render_area.height(), surface.height()),
            ),
        )?;

        Ok(())
    }
}

impl TextRenderer<'_, '_> {
    pub fn render(&self, text: &str, canvas: &mut WindowCanvas) -> Result<(), String> {
        self.font_renderer.render_at(text, self.render_area, canvas)
    }
}
