use cvkg_render_gpu::SurtrRenderer;
use cvkg_core::Renderer;

fn main() {
    let mut renderer: Option<SurtrRenderer> = None;
    if let Some(ref mut r) = renderer {
        let _dyn_renderer: &mut dyn Renderer = r;
    }
}
