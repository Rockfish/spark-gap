mod context;
mod draw;

use crate::context::Context;
use crate::draw::{create_render_pipeline, draw};
use spark_gap::frame_counter::FrameCounter;
use std::sync::Arc;
use winit::keyboard::NamedKey::Escape;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    keyboard,
    window::Window,
};

async fn run(event_loop: EventLoop<()>, window: Arc<Window>) {
    let mut context = Context::new(window).await;
    let mut frame_counter = FrameCounter::new();

    let render_pipeline = create_render_pipeline(&context);

    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent { window_id: _, event } = event {
                match event {
                    WindowEvent::Resized(new_size) => {
                        context.resize(new_size);
                    }
                    WindowEvent::RedrawRequested => {
                        frame_counter.update();

                        draw(&context, &render_pipeline);

                        context.window.request_redraw();
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        // if event.state == ElementState::Pressed {
                        if event.logical_key == keyboard::Key::Named(Escape) {
                            target.exit()
                        }
                        // }
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                };
            }
        })
        .unwrap();
}

pub fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    #[allow(unused_mut)]
    let mut builder = winit::window::WindowBuilder::new();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowBuilderExtWebSys;
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        builder = builder.with_canvas(Some(canvas));
    }

    let window = Arc::new(
        builder
            .with_title("A triangle.")
            .with_inner_size(winit::dpi::LogicalSize::new(400.0, 400.0))
            .build(&event_loop)
            .unwrap(),
    );

    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run(event_loop, window));
    }

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
