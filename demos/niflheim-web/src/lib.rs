#![cfg(target_arch = "wasm32")]
use cvkg_components::niflheim_demo;
use cvkg_core::{FrameRenderer, View};
use cvkg_render_web::WebRenderer;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

pub use cvkg_render_web::get_render_tier_name;

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("error initializing log");

    log::info!("Niflheim Web Demo Initializing...");

    // Create the WebRenderer
    let mut renderer = WebRenderer::new();
    if let Err(e) = renderer.forge().await {
        log::error!("FORGE FAILURE: {:?}", e);
        return Err(e);
    }

    log::info!("Forge returned successfully. Tier: {:?}", renderer.tier());

    // Apply the Niflheim Default Tokens
    let tokens = cvkg_core::default_tokens();
    cvkg_core::env::insert::<cvkg_core::YggdrasilKey>(tokens);
    cvkg_core::env::insert::<cvkg_core::AppearanceKey>(cvkg_core::Appearance::Dark);

    // Initial VDOM update
    if let Err(e) = renderer.update_vdom(niflheim_demo()) {
        log::error!("VDOM UPDATE FAILURE: {:?}", e);
        return Err(e);
    }

    log::info!("Niflheim Web Demo Ready. Starting render loop...");

    // Wrap renderer for the loop
    let renderer = Rc::new(RefCell::new(renderer));

    // Simple render loop using request_animation_frame
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    let window = web_sys::window().unwrap();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut r = renderer.borrow_mut();

        // Begin frame (clears canvas, updates delta_time)
        r.begin_frame();

        // Get viewport size from canvas
        let (width, height) = if let Some(canvas) = r.canvas() {
            (canvas.width() as f32, canvas.height() as f32)
        } else {
            (800.0, 600.0)
        };

        let demo_width = 600.0;
        let demo_height = 400.0;
        let rect = cvkg_core::Rect {
            x: (width - demo_width) / 2.0,
            y: (height - demo_height) / 2.0,
            width: demo_width,
            height: demo_height,
        };

        // Render the demo view directly
        niflheim_demo().render(&mut *r, rect);

        // End frame (flushes to GPU if needed)
        r.end_frame(());

        // Schedule next frame
        web_sys::window()
            .unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
    }) as Box<dyn FnMut()>));

    window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;

    Ok(())
}
