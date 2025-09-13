#![windows_subsystem = "windows"]
use gpui::prelude::*;
use gpui::{
    App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div, px, rgb, size
};

mod app;

struct HelloWorld {
    text: SharedString
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size(px(500.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().size_8().bg(gpui::red()))
                    .child(div().size_8().bg(gpui::green()))
                    .child(div().size_8().bg(gpui::blue()))
                    .child(div().size_8().bg(gpui::yellow()))
                    .child(div().size_8().bg(gpui::black()))
                    .child(div().size_8().bg(gpui::white()))
            )
    }
}

fn main() {
    let is_first_instance = {
        #[cfg(target_os = "windows")]
        {
            app::windows_only_instance::handle_single_instance()
        }
        #[cfg(not(target_os = "windows"))]
        {
            true
        }
    };
    if !is_first_instance {
        println!("Application is already running");
        return;
    }

    let app = Application::new();

    app.run(|cx: &mut App| {
        let window_bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        let opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(window_bounds)),
            ..Default::default()
        };

        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        // Open a window with default options
        cx.open_window(opts, |_, cx| {
            cx.activate(true);

            cx.new(|_| HelloWorld { text: "World".into() })
        })
        .unwrap();
    });
}
