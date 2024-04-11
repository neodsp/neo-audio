use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::{Color, Length, Size};

pub struct LevelMeter {
    level_db: f32,
}

impl LevelMeter {
    pub fn new(level_db: f32) -> Self {
        Self { level_db }
    }
}

pub fn level_meter(level_db: f32) -> LevelMeter {
    LevelMeter::new(level_db)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for LevelMeter
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> iced::Size<iced::Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(2.0, 20.0))
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                ..renderer::Quad::default()
            },
            Color::BLACK,
        )
    }
}
