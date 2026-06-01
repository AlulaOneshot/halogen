use std::sync::Arc;
use halogen_core::color::{Color, ColorStop, ColorStops, ConicGradient, LinearGradient, RadialGradient};
use halogen_core::geometry::{Axes, Length, Percentage, Pixels, Point, ScaleUnits, Sizing};
use halogen_core::style::{AlignContent, AlignItems, DisplayFlexStyles, DisplayStyles, FlexDirection, JustifyContent, Paint, Style, ViewportSize};
use halogen_core::theme::Theme;
use halogen_core::tree::{Scheduler, WidgetTree};
use halogen_core::widget::{BuildContext, Widget, WidgetId, WidgetNode};
use halogen_render::backend::tiny_skia::SkiaBackend;
use halogen_render::executor::Executor;

struct DumbassScheduler;

impl Scheduler for DumbassScheduler {
    fn spawn(&self, _: Box<dyn std::future::Future<Output = ()> + Send + 'static>) {}
    fn mark_dirty(&self, _: WidgetId) {}
}

fn build_tree(root: impl Widget, viewport: ViewportSize) -> WidgetTree {
    let theme = Theme::default();
    let scheduler = Arc::new(DumbassScheduler);
    let mut tree = WidgetTree::new(viewport, theme, scheduler);

    tree.set_root(WidgetNode::new(root));

    tree
}

struct Root {
    style: Style,
    children: Vec<Box<dyn Widget>>,
}

impl Clone for Root {
    fn clone(&self) -> Self {
        let mut children = vec![];

        for child in self.children.iter() {
            children.push(child.clone_box());
        }

        Self {
            style: self.style.clone(),
            children,
        }
    }
}

impl Root {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Self {
        let mut style = Style::default();

        style.box_model.content = Axes::both(Sizing::Exact(Length::ViewportPercent(Percentage(100.0))));
        style.paint.background = Paint::RadialGradient(RadialGradient::new(Point::new(0.5, 0.5), ColorStops::new(ColorStop::new(0.0, Color::rgb8(46, 16, 101)), ColorStop::new(1.0, Color::rgb8(15, 23, 42)))));
        style.display = DisplayStyles::Flex(DisplayFlexStyles {
            direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        });
        style.gap = Axes::new(Length::Pixels(Pixels(5.0)), Length::Pixels(Pixels(5.0)));

        Root { style, children }
    }
}

impl Widget for Root {
    fn build(&self, cx: &mut BuildContext) -> Vec<WidgetNode> {
        let mut nodes = vec![];

        for child in self.children.iter() {
            nodes.push(WidgetNode::from_box(child.clone_box()))
        }

        nodes
    }

    fn style(&self) -> &Style {
        &self.style
    }
}

#[derive(Clone)]
struct Element {
    style: Style,
}

impl Element {
    pub fn new() -> Self {
        let mut style = Style::default();

        style.box_model.content = Axes::new(
            Sizing::Exact(Length::Pixels(Pixels(25.0))),
            Sizing::Exact(Length::Pixels(Pixels(25.0)))
        );
        style.paint.background = Paint::LinearGradient(LinearGradient::new(45.0, ColorStops::new(
            ColorStop::new(0.0, Color::rgb8(0, 255, 0)),
            ColorStop::new(1.0, Color::rgb8(0, 0, 255))
        )));

        Element { style }
    }
}

impl Widget for Element {
    fn build(&self, cx: &mut BuildContext) -> Vec<WidgetNode> {
        vec![]
    }

    fn style(&self) -> &Style {
        &self.style
    }
}

struct App {
    window:   Option<Arc<winit::window::Window>>,
    executor: Option<Executor<SkiaBackend<Arc<winit::window::Window>, Arc<winit::window::Window>>>>,
    tree:     Option<WidgetTree>,
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_title("Halogen Demo")
                        .with_inner_size(winit::dpi::LogicalSize::new(250, 250)),
                )
                .unwrap(),
        );

        let scale = window.scale_factor();
        let size = window.inner_size().to_logical::<f32>(scale);

        let viewport = ViewportSize {
            width:  Pixels(size.width),
            height: Pixels(size.height),
        };
        let backend = SkiaBackend::new(
            window.clone(),
            window.clone(),
            window.inner_size().width,
            window.inner_size().height,
        );

        self.executor = Some(Executor::new(backend));
        self.tree     = Some(build_tree(Root::new(vec![
            Element::new().into(),
            Element::new().into(),
            Element::new().into(),
        ]), viewport));
        self.window   = Some(window);
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _id: winit::window::WindowId, event: winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            winit::event::WindowEvent::Resized(physical_size) => {
                if let Some(executor) = &mut self.executor {
                    executor.backend_mut().resize(physical_size.width, physical_size.height);
                }
                if let Some(tree) = &mut self.tree {
                    let logical = physical_size.to_logical::<f32>(self.window.as_ref().unwrap().scale_factor());
                    tree.set_viewport(ViewportSize {
                        width:  Pixels(logical.width),
                        height: Pixels(logical.height),
                    });
                }
            }

            winit::event::WindowEvent::RedrawRequested => {
                if let (Some(executor), Some(tree)) =
                    (&mut self.executor, &mut self.tree)
                {
                    tree.update();
                    tree.paint(executor);
                    executor.flush();
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let mut app = App {
        window:   None,
        executor: None,
        tree:     None,
    };
    event_loop.run_app(&mut app).unwrap();
}