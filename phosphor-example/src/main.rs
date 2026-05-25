//! Layout demo: a row of colored boxes rendered to a PNG via the Skia backend.
//!
//! Run with: `cargo run --example row_of_boxes`
//! Output: `row_of_boxes.png` in the crate root.

use phosphor_core::color::Color;
use phosphor_core::style::{BorderEdge, BorderStyle, ColorStop, Corners, Edges, GapStyles, GradientStops, SizingAxes, Texture};
use phosphor_core::{
    style::{Background, Dimension, Display, FlexDirection, Sizing, ViewportSize, WidgetStyle},
    theme::Theme,
    tree::WidgetTree,
    widget::{BuildContext, Widget, WidgetNode},
};
use phosphor_render::{skia, PhosphorRenderer, SkiaRenderer};

const VIEWPORT_W: i32 = 1280;
const VIEWPORT_H: i32 = 720;

// =============
// ColorBox
// =============

/// A fixed-size leaf widget with a solid background color.
struct ColorBox {
    texture: Texture,
    size: f32,
}

impl ColorBox {
    fn new(texture: Texture, size: f32) -> Self {
        Self { texture, size }
    }
}

impl Widget for ColorBox {
    fn build(&self, _cx: &mut BuildContext) -> Vec<WidgetNode> {
        vec![] // leaf
    }

    fn style(&self) -> &dyn phosphor_core::style::Style {
        // We can't return a reference to a local here, so we store the style.
        // For a quick demo, leak it — this is not production code.
        let style = Box::new(
            WidgetStyle::new()
                .with_sizing(SizingAxes::new(
                    Sizing::Exact(Dimension::Px(self.size)),
                    Sizing::Exact(Dimension::Px(self.size)),
                ))
                .with_border(BorderStyle::new(Edges::all(BorderEdge {
                    width: 0.0,
                    texture: self.texture.clone(),
                })).radius(Corners::all(90.0)))
                .with_background(Background::from(self.texture.clone())),
        );
        Box::leak(style)
    }
}

// =============
// Row
// =============

/// A flex-row container that renders a fixed set of colored boxes.
struct Row {
    boxes: Vec<(Texture, f32)>,
    gap: f32,
}

impl Row {
    fn new(boxes: Vec<(Texture, f32)>, gap: f32) -> Self {
        Self { boxes, gap }
    }
}

impl Widget for Row {
    fn build(&self, _cx: &mut BuildContext) -> Vec<WidgetNode> {
        self.boxes
            .iter()
            .enumerate()
            .map(|(i, (texture, size))| {
                WidgetNode::new(ColorBox::new(texture.clone(), *size)).with_key(i.to_string())
            })
            .collect()
    }

    fn style(&self) -> &dyn phosphor_core::style::Style {
        let style = Box::new(
            WidgetStyle::new()
                .with_sizing(SizingAxes::new(
                    Sizing::Exact(Dimension::Px(VIEWPORT_W as f32)),
                    Sizing::Exact(Dimension::Px(VIEWPORT_H as f32)),
                ))
                .with_display(Display::Flex {
                    direction: FlexDirection::Column,
                    align_items: phosphor_core::style::Align::Center,
                    justify_content: phosphor_core::style::Justify::Center,
                })
                .with_gap(GapStyles {
                    row_gap: Dimension::Px(self.gap),
                    column_gap: Dimension::Px(self.gap)
                })
                .with_background(Background::color(Color::hex("#1a1a2e"))),
        );
        Box::leak(style)
    }
}

// =============
// Main
// =============

fn main() {
    let viewport = ViewportSize {
        width: VIEWPORT_W as f32,
        height: VIEWPORT_H as f32,
    };

    let root = Row::new(
        vec![
            (
                Texture::radial(
                    (0.5, 0.5),
                    GradientStops::new(
                        ColorStop {
                            position: 0.0,
                            color: Color::hex("#0000ff"),
                        },
                        ColorStop {
                            position: 1.0,
                            color: Color::hex("#00ff00"),
                        },
                    ),
                ),
                100.0,
            ),
            (
                Texture::linear(
                    45.0,
                    GradientStops::new(
                        ColorStop {
                            position: 0.0,
                            color: Color::hex("#0000ff"),
                        },
                        ColorStop {
                            position: 1.0,
                            color: Color::hex("#00ff00"),
                        },
                    ),
                ),
                100.0,
            ),
            (
                Texture::conic(
                    (0.5, 0.5),
                    90.0,
                    GradientStops::new(
                        ColorStop {
                            position: 0.0,
                            color: Color::hex("#0000ff"),
                        },
                        ColorStop {
                            position: 1.0,
                            color: Color::hex("#00ff00"),
                        },
                    ),
                ),
                100.0,
            ),
        ],
        24.0,
    );

    // Build the tree once, and layout
    let mut tree = WidgetTree::new(viewport, Theme::new());
    tree.set_root(root);
    tree.rebuild();
    tree.layout();

    // Skia Renderer
    let mut renderer = SkiaRenderer::new_offscreen(VIEWPORT_W, VIEWPORT_H)
        .expect("failed to create offscreen surface");

    // Tree frame.
    renderer.begin_frame();
    tree.paint(&mut renderer);
    renderer.end_frame();

    // Use skia to save to png.
    let image = renderer.snapshot();
    let data = image
        .encode(None, skia::EncodedImageFormat::WEBP, None)
        .expect("failed to encode WEBP");
    std::fs::write("circles.webp", data.as_bytes()).expect("failed to write WEBP");

    println!("wrote row_of_boxes.webp");
}