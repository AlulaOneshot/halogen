use phosphor_core::color::Colors;
use phosphor_core::event::{Event, EventResult, Key};
use phosphor_core::style::{
    Background, BorderEdge, BorderStyle, Dimension, GradientStops, SizingAxes,
    Style, WidgetStyle,
};
use phosphor_core::widget::{BuildContext, EventContext, Widget, WidgetNode};
use phosphor_platform::app::{App, AppConfig};

struct Main {
    style: WidgetStyle,
}

impl Widget for Main {
    fn build(&self, cx: &mut BuildContext) -> Vec<WidgetNode> {
        vec![]
    }

    fn style(&self) -> &dyn Style {
        &self.style
    }

    fn on_event(&self, event: &Event, cx: &mut EventContext) -> EventResult {
        println!("on_event: {:#?}", event);
        EventResult::Bubble
    }
}

fn main() {
    let app = App::new(Main {
        style: WidgetStyle::new()
            .with_sizing(SizingAxes::new(
                (Some(Dimension::Percent(100.0)), None),
                (Some(Dimension::Percent(100.0)), None),
            ))
            .with_background(Background::conic(
                (0.5, 0.5),
                0.45,
                GradientStops::new((0.0, "#00FF00"), (1.0, "#0000FF"))
            ))
            .with_border(BorderStyle::new(
                BorderEdge::new(1.0, Colors::TRANSPARENT),
                45.0,
            )),
    })
    .with_config(AppConfig {
        title: "Phosphor".into(),
        size: (800, 600),
        fps: 60,
        resizable: false,
        backend: None,
    });

    app.run();
}
