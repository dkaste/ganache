use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, Rect};

use ganache::Gui;

enum GuiDrawCommand {
    Rect(Rect, Color),
}

enum GuiInputEvent {}

impl ganache::InputEvent for GuiInputEvent {
    fn dirty(&self) -> bool {
        false
    }

    fn offset_coordinates(&mut self, _x: ganache::Scalar, _y: ganache::Scalar) {}
}

enum GuiContext {}

impl ganache::Context for GuiContext {
    type ThemeResources = ();
    type StyleFieldValue = ();
    type DrawCommand = GuiDrawCommand;
    type DrawContext = ();
    type InputEvent = GuiInputEvent;
}

struct Panel(ganache::default_layout::Settings);

impl ganache::Widget<GuiContext> for Panel {
    fn kind_id(&self) -> &'static str {
        "Panel"
    }

    fn takes_focus(&self) -> bool {
        false
    }

    fn minimum_size(&self, args: ganache::MinimumSizeArgs<'_, GuiContext>) -> ganache::Dimensions {
        ganache::default_layout::minimum_size(&args, &self.0)
    }

    fn layout_children(&self, mut args: ganache::LayoutChildrenArgs<'_, GuiContext>) {
        ganache::default_layout::layout_children(&mut args, &self.0)
    }

    fn process_event(
        &mut self,
        _args: ganache::ProcessEventArgs<'_, GuiContext>,
    ) -> ganache::ProcessEventResult {
        ganache::ProcessEventResult::default()
    }

    fn draw(&self, _args: ganache::DrawArgs<'_, GuiContext>) {}
}

struct Button;

impl ganache::Widget<GuiContext> for Button {
    fn kind_id(&self) -> &'static str {
        "Button"
    }

    fn takes_focus(&self) -> bool {
        true
    }

    fn minimum_size(&self, _args: ganache::MinimumSizeArgs<'_, GuiContext>) -> ganache::Dimensions {
        ganache::Dimensions::zero()
    }

    fn layout_children(&self, _args: ganache::LayoutChildrenArgs<'_, GuiContext>) {
        panic!("Button widget slots should not have children!")
    }

    fn process_event(
        &mut self,
        _args: ganache::ProcessEventArgs<'_, GuiContext>,
    ) -> ganache::ProcessEventResult {
        ganache::ProcessEventResult::default()
    }

    fn draw(&self, args: ganache::DrawArgs<'_, GuiContext>) {
        args.commands.push(GuiDrawCommand::Rect(
            Rect::new(
                args.bounds.x as f32,
                args.bounds.y as f32,
                args.bounds.size.width as f32,
                args.bounds.size.height as f32,
            ),
            Color::new(1.0, 0.0, 1.0, 1.0),
        ));
    }
}

struct Example {
    gui: Gui<GuiContext>,
    gui_theme: ganache::Theme<(), ()>,
}

impl EventHandler for Example {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let (width, height) = graphics::size(ctx);
        let gui_size = ganache::Dimensions::new(width as f64, height as f64);
        self.gui.slots.set_size(self.gui.root_slot_id(), gui_size);
        self.gui.layout_if_needed(&self.gui_theme);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::new(0.0, 0.33, 0.66, 1.0));
        for cmd in self.gui.draw(&self.gui_theme, ()) {
            match cmd {
                GuiDrawCommand::Rect(rect, color) => {
                    let mesh = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        rect,
                        color,
                    ).unwrap();
                    graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
                }
            }
        }
        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult<()> {
    let (mut ctx, mut event_loop) = ContextBuilder::new("ganache_ggez_example", "ganache")
        .build()?;
    
    let mut gui = Gui::new(ganache::Bounds::zero());

    let (panel_slot_id, _) = gui.add_slot_with_widget(
        gui.root_slot_id(),
        ganache::SlotInfo {
            minimum_size: ganache::Dimensions::new(400.0, 300.0),
            ..Default::default()
        },
        Panel(ganache::default_layout::Settings {
            padding: 5.0,
            child_spacing: 2.0,
            axis: ganache::default_layout::Axis::Horizontal,
        }),
    );

    gui.add_slot_with_widget(
        panel_slot_id,
        ganache::SlotInfo {
            expand_x: true,
            expand_y: true,
            ..Default::default()
        },
        Button,
    );
    gui.add_slot_with_widget(
        panel_slot_id,
        ganache::SlotInfo {
            expand_x: true,
            expand_y: true,
            ..Default::default()
        },
        Button,
    );

    let gui_theme = ganache::Theme::new(());
    let mut example = Example {
        gui,
        gui_theme,
    };
    event::run(&mut ctx, &mut event_loop, &mut example)
}