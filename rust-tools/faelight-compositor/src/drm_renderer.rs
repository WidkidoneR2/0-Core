// faelight-compositor — GBM/EGL render loop
// Replaces dumb buffer path with proper GPU compositing

use smithay::{
    utils::Transform,
    backend::{
        allocator::gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
        drm::{
            compositor::{DrmCompositor, FrameFlags},
            exporter::gbm::{GbmFramebufferExporter, NodeFilter},
            DrmDevice, DrmDeviceFd,
        },
        egl::{EGLContext, EGLDisplay},
        renderer::{
            element::{
                surface::{render_elements_from_surface_tree, WaylandSurfaceRenderElement},
                Kind,
            },
            gles::GlesRenderer,
            Color32F,
        },
    },
    output::{Output, PhysicalProperties, Subpixel},
    reexports::drm::control::ModeTypeFlags,
    utils::{Point, Scale, Size},
};
use smithay_drm_extras::drm_scanner::{DrmScanEvent, DrmScanner, SimpleCrtcMapper};
use drm_fourcc::{DrmFormat, DrmFourcc};
use std::collections::HashSet;

use crate::FaelightCompositor;

const CLEAR_COLOR: Color32F = Color32F::new(
    0x11 as f32 / 255.0,
    0x14 as f32 / 255.0,
    0x0f as f32 / 255.0,
    1.0,
);

type FaelightDrmCompositor = DrmCompositor<GbmAllocator<DrmDeviceFd>, GbmFramebufferExporter<DrmDeviceFd>, (), DrmDeviceFd>;

#[allow(dead_code)]
pub struct GbmRenderPipeline {
    pub compositor: FaelightDrmCompositor,
    pub renderer: GlesRenderer,
    pub output: Output,
}

pub fn init_gbm_pipeline(
    drm: &mut DrmDevice,
    drm_fd: DrmDeviceFd,
) -> Result<GbmRenderPipeline, Box<dyn std::error::Error>> {
    let gbm: GbmDevice<DrmDeviceFd> = GbmDevice::new(drm_fd.clone())
        .map_err(|e| format!("GBM device: {e}"))?;
    tracing::info!("GBM device created");

    let egl_display = unsafe {
        EGLDisplay::new(gbm.clone()).map_err(|e| format!("EGL display: {e}"))?
    };
    tracing::info!("EGL display created");

    let egl_context = EGLContext::new(&egl_display)
        .map_err(|e| format!("EGL context: {e}"))?;

    let renderer = unsafe {
        GlesRenderer::new(egl_context).map_err(|e| format!("GLES renderer: {e}"))?
    };
    tracing::info!("GLES renderer created");

    // Scan connectors
    let mut scanner: DrmScanner<SimpleCrtcMapper> = DrmScanner::new();
    let scan_events = scanner.scan_connectors(drm).unwrap_or_default();

    let mut found_connector = None;
    for event in scan_events.iter() {
        if let DrmScanEvent::Connected { connector, crtc: Some(crtc) } = event {
            let mode = connector.modes()
                .iter()
                .find(|m| m.mode_type().contains(ModeTypeFlags::PREFERRED))
                .or_else(|| connector.modes().first())
                .cloned();
            if let Some(mode) = mode {
                found_connector = Some((connector.clone(), crtc, mode));
                break;
            }
        }
    }

    let (connector, crtc, mode) = found_connector.ok_or("No connected display")?;
    let (w, h) = (mode.size().0 as i32, mode.size().1 as i32);
    tracing::info!(
        w, h,
        refresh = mode.vrefresh(),
        connector = connector.interface().as_str(),
        "Display found"
    );

    let surface = drm
        .create_surface(crtc, mode, &[connector.handle()])
        .map_err(|e| format!("DRM surface: {e}"))?;
    tracing::info!("DRM surface created");

    let output = Output::new(
        connector.interface().as_str().to_string(),
        PhysicalProperties {
            size: Size::from((w, h)),
            make: "faelight".to_string(),
            model: connector.interface().as_str().to_string(),
            subpixel: Subpixel::Unknown,
            serial_number: "forest-001".to_string(),
        },
    );
    // Set the active mode on the output -- required by DrmCompositor::render_frame
    let output_mode = smithay::output::Mode {
        size: Size::from((w, h)),
        refresh: mode.vrefresh() as i32 * 1000,
    };
    output.change_current_state(Some(output_mode), Some(Transform::Normal), None, Some(Point::from((0, 0))));
    output.set_preferred(output_mode);

    let allocator = GbmAllocator::new(
        gbm.clone(),
        GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
    );

    let exporter = GbmFramebufferExporter::new(gbm.clone(), NodeFilter::None);

    let render_formats: HashSet<DrmFormat> = renderer
        .egl_context()
        .dmabuf_render_formats()
        .iter()
        .copied()
        .collect();

    let color_formats = vec![DrmFourcc::Argb8888, DrmFourcc::Xrgb8888];

    let compositor = DrmCompositor::new(
        &output,
        surface,
        None,
        allocator,
        exporter,
        color_formats,
        render_formats,
        drm.cursor_size(),
        Some(gbm),
    )
    .map_err(|e| format!("DRM compositor: {e:?}"))?;

    tracing::info!("DRM compositor created — GPU render loop ready");

    Ok(GbmRenderPipeline { compositor, renderer, output })
}

pub fn add_output_to_space(pipeline: &GbmRenderPipeline, state: &mut FaelightCompositor) {
    // Register the output with the space so window placement works
    if state.space.outputs().count() == 0 {
        state.space.map_output(&pipeline.output, (0, 0));
        tracing::info!("Output added to space");
    }
}

pub fn render_frame(pipeline: &mut GbmRenderPipeline, state: &mut FaelightCompositor) {
    let elements: Vec<WaylandSurfaceRenderElement<GlesRenderer>> = state
        .space
        .elements()
        .flat_map(|window| {
            let surface = match window.toplevel() {
                Some(t) => t.wl_surface().clone(),
                None => return vec![],
            };
            render_elements_from_surface_tree(
                &mut pipeline.renderer,
                &surface,
                Point::from((0, 0)),
                Scale::from(1.0),
                1.0,
                Kind::Unspecified,
            )
        })
        .collect();

    match pipeline.compositor.render_frame::<GlesRenderer, WaylandSurfaceRenderElement<GlesRenderer>>(
        &mut pipeline.renderer,
        &elements,
        CLEAR_COLOR,
        FrameFlags::DEFAULT,
    ) {
        Ok(result) => {
            if !result.is_empty {
                if let Err(e) = pipeline.compositor.queue_frame(()) {
                    tracing::error!("queue_frame failed: {e}");
                }
            }
        }
        Err(e) => tracing::error!("render_frame failed: {e}"),
    }
}
