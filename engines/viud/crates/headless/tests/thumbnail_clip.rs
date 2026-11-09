// SPDX-License-Identifier: MIT OR Apache-2.0
use smithay::backend::allocator::Fourcc;
use smithay::backend::egl::{EGLContext, EGLDisplay, native::EGLSurfacelessDisplay};
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture};
use smithay::backend::renderer::utils::draw_render_elements;
use smithay::backend::renderer::{Bind, Color32F, ExportMem, Frame, Offscreen, Renderer};
use smithay::utils::{Rectangle, Transform};

#[test]
#[ignore = "needs GLES"]
fn clipped_window_thumbnail_scales_the_visual_crop() {
    let mut h = testkit::Harness::new();
    let (surface, _, _) = h.mapped_toplevel(200, 160);
    let (buffer, file) = h.client.shm_buffer_with(
        200,
        160,
        800,
        wayland_client::protocol::wl_shm::Format::Argb8888,
    );
    h.client
        .write_buffer(file, &[0, 0, 255, 255].repeat(200 * 160));
    surface.attach(Some(&buffer), 0, 0);
    surface.damage_buffer(0, 0, 200, 160);
    surface.commit();
    h.roundtrip();
    let window = h.wire.inner.space.state.elements().next().unwrap().clone();
    window.user_data().insert_if_missing(|| {
        std::sync::Mutex::new(comp_model::request::VisualAttributes {
            clip: Some(comp_model::request::ClipRect {
                x: 20.0,
                y: 20.0,
                width: 40.0,
                height: 40.0,
            }),
            ..Default::default()
        })
    });
    let display = unsafe { EGLDisplay::new(EGLSurfacelessDisplay) }.unwrap();
    let context = EGLContext::new(&display).unwrap();
    let mut renderer = unsafe { GlesRenderer::new(context) }.unwrap();
    let rect = Rectangle::new((10, 10).into(), (100, 80).into());
    let (_, _, source_frame) =
        world::window::draw::frame::scene::capture_framed(&mut renderer, &window, 1.0).unwrap();
    let (scale, _, _) = world::window::draw::thumbnail::place(source_frame, rect).unwrap();
    assert!(scale < 1.0);
    let elements = world::window::draw::thumbnail::elements(&mut renderer, &window, 1, rect, 1.0);
    let full: Rectangle<i32, smithay::utils::Physical> = Rectangle::from_size((128, 128).into());
    let mut target =
        Offscreen::<GlesTexture>::create_buffer(&mut renderer, Fourcc::Abgr8888, (128, 128).into())
            .unwrap();
    {
        let mut fb = renderer.bind(&mut target).unwrap();
        let mut frame = renderer
            .render(&mut fb, full.size, Transform::Normal)
            .unwrap();
        frame
            .clear(Color32F::new(0.0, 0.0, 0.0, 1.0), &[full])
            .unwrap();
        draw_render_elements::<GlesRenderer, _, _>(&mut frame, 1.0, &elements, &[full]).unwrap();
        frame.finish().unwrap().wait().unwrap();
    }
    let fb = renderer.bind(&mut target).unwrap();
    let mapping = renderer
        .copy_framebuffer(&fb, Rectangle::from_size((128, 128).into()), Fourcc::Abgr8888)
        .unwrap();
    drop(fb);
    let pixels = renderer.map_texture(&mapping).unwrap();
    let red: Vec<_> = pixels
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, p)| p[0] > 240 && p[1] < 10 && p[2] < 10)
        .map(|(i, _)| (i % 128, i / 128))
        .collect();
    assert!(!red.is_empty(), "thumbnail must contain the clipped client");
    let width = red.iter().map(|p| p.0).max().unwrap() - red.iter().map(|p| p.0).min().unwrap() + 1;
    let height =
        red.iter().map(|p| p.1).max().unwrap() - red.iter().map(|p| p.1).min().unwrap() + 1;
    assert!(
        (width as f64 - 40.0 * scale).abs() <= 2.0,
        "scaled crop width {width}, scale {scale}"
    );
    assert!(
        (height as f64 - 40.0 * scale).abs() <= 2.0,
        "scaled crop height {height}, scale {scale}"
    );
}
