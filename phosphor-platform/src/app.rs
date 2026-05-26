use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use phosphor_core::event::{Event, LifecycleEvent};
use phosphor_core::handler::SyncOnlyExecutor;
use phosphor_core::style::ViewportSize;
use phosphor_core::theme::Theme;
use phosphor_core::tree::WidgetTree;
use phosphor_core::widget::Widget;
use phosphor_render::PhosphorRenderer;
use crate::event::translate_window_event;
use crate::{GpuBackend, GpuContext};

pub struct AppConfig {
    /// Window title.
    pub title: String,
    /// Initial size in logical pixels as (width, height)
    pub size: (u32, u32),
    /// Target frames per second. `0` means unlimited.
    pub fps: u32,
    /// Whether the window is resizable.
    pub resizable: bool,
    /// GPU backend to use. If `None`, the best available backend is selected
    /// automatically based on the platform and compiled features.
    pub backend: Option<GpuBackend>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "Phosphor".into(),
            size: (1280, 720),
            fps: 60,
            resizable: true,
            backend: None,
        }
    }
}

pub struct App {
    root: Box<dyn Widget>,
    theme: Theme,
    config: AppConfig,
}

impl App {
    pub fn new(root: impl Widget + 'static) -> Self {
        Self {
            root: Box::new(root),
            theme: Theme::new(),
            config: AppConfig::default(),
        }
    }

    /// Set the theme.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set the app configuration.
    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.config = config;
        self
    }

    pub fn run(self) {
        let event_loop = EventLoop::new().expect("failed to create event loop");
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut handler = AppHandler::new(self.root, self.theme, self.config);
        event_loop.run_app(&mut handler).expect("event loop error");
    }

    pub async fn run_async(self) {
        tokio::task::spawn_blocking(move || self.run())
            .await
            .expect("event loop panicked");
    }
}

struct AppHandler {
    root: Option<Box<dyn Widget>>,
    theme: Theme,
    config: AppConfig,
    // Set after window creation
    window: Option<Arc<Window>>,
    context: Option<Box<dyn GpuContext>>,
    tree: Option<WidgetTree>,
}

impl AppHandler {
    fn new(root: Box<dyn Widget>, theme: Theme, config: AppConfig) -> Self {
        Self {
            root: Some(root),
            theme,
            config,
            window: None,
            context: None,
            tree: None,
        }
    }

    fn viewport(&self) -> ViewportSize {
        self.window
            .as_ref()
            .map(|w| {
                let size = w.inner_size();
                ViewportSize {
                    width: size.width as f32,
                    height: size.height as f32,
                }
            })
            .unwrap_or(ViewportSize {
                width: self.config.size.0 as f32,
                height: self.config.size.1 as f32,
            })
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window
        let attrs = Window::default_attributes()
            .with_title(&self.config.title)
            .with_inner_size(LogicalSize::new(self.config.size.0, self.config.size.1))
            .with_resizable(self.config.resizable);

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );

        // Select and initialize GPU backend
        let backend = self
            .config
            .backend
            .unwrap_or_else(GpuBackend::best_for_platform);

        let context = backend
            .create_context(Arc::clone(&window))
            .expect("failed to create GPU context");

        // Build widget tree
        let viewport = ViewportSize {
            width: self.config.size.0 as f32,
            height: self.config.size.1 as f32,
        };
        let mut tree = WidgetTree::new(viewport, self.theme.clone());
        if let Some(root) = self.root.take() {
            tree.set_root_boxed(root);
        }

        self.window = Some(window);
        self.context = Some(context);
        self.tree = Some(tree);

        // Dispatch Resumed lifecycle event
        if let Some(tree) = &self.tree {
            let executor = SyncOnlyExecutor;
            tree.dispatch(
                &Event::<()>::Lifecycle(LifecycleEvent::Resumed),
                &executor,
            );
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match &event {
            WindowEvent::CloseRequested => {
                if let Some(tree) = &self.tree {
                    let executor = SyncOnlyExecutor;
                    tree.dispatch(
                        &Event::<()>::Lifecycle(LifecycleEvent::CloseRequested),
                        &executor,
                    );
                }
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                let viewport = ViewportSize {
                    width: size.width as f32,
                    height: size.height as f32,
                };
                if let Some(ctx) = &mut self.context {
                    ctx.resize(size.width as i32, size.height as i32);
                }
                if let Some(tree) = &mut self.tree {
                    tree.set_viewport(viewport);
                    let executor = SyncOnlyExecutor;
                    tree.dispatch(
                        &Event::<()>::Lifecycle(LifecycleEvent::Resized(viewport)),
                        &executor,
                    );
                }
            }

            WindowEvent::RedrawRequested => {
                let executor = SyncOnlyExecutor;
                if let (Some(tree), Some(ctx)) = (&mut self.tree, &mut self.context) {
                    tree.rebuild();
                    tree.layout();
                    let renderer = ctx.renderer();
                    renderer.begin_frame();
                    tree.paint(renderer);
                    renderer.end_frame();
                    ctx.present();
                }
            }

            //TODO: OS Text input, GamePad stuff

            other => {
                // Translate and dispatch to the widget tree
                if let Some(translated) = translate_window_event(other) {
                    if let Some(tree) = &self.tree {
                        let executor = SyncOnlyExecutor;
                        tree.dispatch(&translated, &executor);
                    }
                }
                // Request redraw after any input
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
        }
    }
}