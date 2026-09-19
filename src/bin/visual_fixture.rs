use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env,
    error::Error,
    fs::File,
    path::Path,
    sync::mpsc,
    time::Duration,
};

use eframe::egui;

use sublens::{
    app::configure_style,
    dedup::deduplicate,
    model::{Protocol, Security, Transport},
    protocols::parse_uri,
    resolver::{
        stage::{PipelineStage, StageKind},
        AnalysisReport, ContentKind,
    },
    ui::main_view::{self, CompactPage, ConnectivityFilter, DuplicateFilter, LteFilter},
};

struct FixtureApp {
    input: String,
    report: AnalysisReport,
    selected: Option<String>,
    protocol_filters: HashSet<Protocol>,
    selected_configs: HashSet<String>,
    search: String,
    security_filter: Option<Security>,
    transport_filter: Option<Transport>,
    connectivity_filter: ConnectivityFilter,
    duplicate_filter: DuplicateFilter,
    lte_filter: LteFilter,
    selection_mode: bool,
    compact_page: CompactPage,
    qr: Option<sublens::qr::QrMatrix>,
    diagnostics: HashMap<usize, sublens::diagnostics::DiagnosticResult>,
    copied_until: Option<std::time::Instant>,
}

impl FixtureApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::with_context(&cc.egui_ctx)
    }

    fn with_context(ctx: &egui::Context) -> Self {
        configure_style(ctx);
        let report = fixture_report();
        let selected = configs_first_id(&report);
        let compact_page = if env::var("SUBLENS_FIXTURE_PAGE").as_deref() == Ok("details") {
            CompactPage::ConfigDetails
        } else {
            CompactPage::ConfigList
        };
        Self {
            input: "https://fixture.local/subscription.txt".to_owned(),
            report,
            // Keep the visual fixture focused on the protocol that previously
            // regressed in the real subscription response.
            selected,
            protocol_filters: HashSet::new(),
            selected_configs: HashSet::new(),
            search: String::new(),
            security_filter: None,
            transport_filter: None,
            connectivity_filter: ConnectivityFilter::All,
            duplicate_filter: DuplicateFilter::All,
            lte_filter: LteFilter::All,
            selection_mode: env::var("SUBLENS_FIXTURE_SELECTION").as_deref() == Ok("1"),
            compact_page,
            qr: None,
            diagnostics: HashMap::new(),
            copied_until: None,
        }
    }

    fn render(&mut self, ctx: &egui::Context) {
        let result = main_view::show(
            ctx,
            &mut self.input,
            &[],
            Some(&self.report),
            &mut self.selected,
            &mut self.protocol_filters,
            &mut self.selected_configs,
            &mut self.search,
            &mut self.security_filter,
            &mut self.transport_filter,
            &mut self.connectivity_filter,
            &mut self.duplicate_filter,
            &mut self.lte_filter,
            &mut self.selection_mode,
            &mut self.compact_page,
            false,
            "Fixture loaded",
            &mut self.qr,
            &self.diagnostics,
            &mut self.copied_until,
        );
        if let Some(id) = result.selected {
            self.selected = Some(id);
        }
    }
}

fn configs_first_id(report: &AnalysisReport) -> Option<String> {
    report.configs.first().map(|config| config.id.clone())
}

impl eframe::App for FixtureApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render(ctx);
    }
}

fn fixture_report() -> AnalysisReport {
    let source = include_str!("../../tests/fixtures/visual_sanitized.txt");
    let extracted = sublens::resolver::extract::extract_items(source, ContentKind::ProxyList);
    let mut configs = extracted
        .proxy_uris
        .iter()
        .filter_map(|uri| parse_uri(uri, None, 0).ok())
        .collect::<Vec<_>>();
    if let Some(hysteria_index) = configs
        .iter()
        .position(|config| config.protocol == Protocol::Hysteria2)
    {
        configs.swap(0, hysteria_index);
    }
    let duplicates = deduplicate(&mut configs);
    let mut protocol_counts = BTreeMap::new();
    for config in &configs {
        *protocol_counts
            .entry(config.protocol.as_str().to_owned())
            .or_insert(0) += 1;
    }
    let count = configs.len();
    AnalysisReport {
        source: "https://fixture.local/subscription.txt".to_owned(),
        stages: vec![
            PipelineStage::success(StageKind::Http, "HTTP 200 · 2.4 KB", 2400, Duration::ZERO),
            PipelineStage::success(StageKind::Detection, "ProxyList", 1, Duration::ZERO),
            PipelineStage::success(
                StageKind::Extract,
                format!("{count} proxy URIs · 0 nested URLs"),
                count,
                Duration::ZERO,
            ),
            PipelineStage::success(
                StageKind::Parse,
                format!("{count} configurations parsed"),
                count,
                Duration::ZERO,
            ),
            PipelineStage::success(
                StageKind::Deduplication,
                format!("{count} configurations normalized"),
                count,
                Duration::ZERO,
            ),
        ],
        configs,
        duplicates,
        protocol_counts,
        candidate_count: count,
        ..AnalysisReport::default()
    }
}

fn main() -> eframe::Result {
    let width = env::var("SUBLENS_FIXTURE_WIDTH")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(1280.0);
    let height = env::var("SUBLENS_FIXTURE_HEIGHT")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(720.0);
    if let Some(path) = env::var_os("SUBLENS_FIXTURE_OFFSCREEN") {
        return render_offscreen(width as u32, height as u32, Path::new(&path))
            .map_err(eframe::Error::AppCreation);
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_min_inner_size([width, height]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "SubLens visual fixture",
        options,
        Box::new(|cc| Ok(Box::new(FixtureApp::new(cc)))),
    )
}

fn render_offscreen(
    width: u32,
    height: u32,
    path: &Path,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let context = egui::Context::default();
    let mut app = FixtureApp::with_context(&context);
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width as f32, height as f32),
        )),
        ..Default::default()
    };
    let full_output = context.run(raw_input, |ctx| app.render(ctx));
    let paint_jobs = context.tessellate(full_output.shapes, full_output.pixels_per_point);
    let screen_descriptor = eframe::egui_wgpu::ScreenDescriptor {
        size_in_pixels: [width, height],
        pixels_per_point: full_output.pixels_per_point,
    };
    let runtime = tokio::runtime::Runtime::new()?;
    let instance = eframe::wgpu::Instance::new(eframe::wgpu::InstanceDescriptor::default());
    let adapter = runtime
        .block_on(
            instance.request_adapter(&eframe::wgpu::RequestAdapterOptions {
                power_preference: eframe::wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }),
        )
        .ok_or_else(|| std::io::Error::other("no compatible WGPU adapter"))?;
    let (device, queue) = runtime
        .block_on(adapter.request_device(&eframe::wgpu::DeviceDescriptor::default(), None))?;
    let format = eframe::wgpu::TextureFormat::Rgba8UnormSrgb;
    let target = device.create_texture(&eframe::wgpu::TextureDescriptor {
        label: Some("sublens-offscreen-target"),
        size: eframe::wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: eframe::wgpu::TextureDimension::D2,
        format,
        usage: eframe::wgpu::TextureUsages::RENDER_ATTACHMENT
            | eframe::wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&eframe::wgpu::TextureViewDescriptor::default());
    let mut renderer = eframe::egui_wgpu::Renderer::new(&device, format, None, 1, false);
    for (id, image_delta) in &full_output.textures_delta.set {
        renderer.update_texture(&device, &queue, *id, image_delta);
    }
    let mut encoder = device.create_command_encoder(&eframe::wgpu::CommandEncoderDescriptor {
        label: Some("sublens-offscreen-encoder"),
    });
    let user_command_buffers = renderer.update_buffers(
        &device,
        &queue,
        &mut encoder,
        &paint_jobs,
        &screen_descriptor,
    );
    {
        let render_pass = encoder.begin_render_pass(&eframe::wgpu::RenderPassDescriptor {
            label: Some("sublens-offscreen-pass"),
            color_attachments: &[Some(eframe::wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: eframe::wgpu::Operations {
                    load: eframe::wgpu::LoadOp::Clear(eframe::wgpu::Color {
                        r: 11.0 / 255.0,
                        g: 17.0 / 255.0,
                        b: 28.0 / 255.0,
                        a: 1.0,
                    }),
                    store: eframe::wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        renderer.render(
            &mut render_pass.forget_lifetime(),
            &paint_jobs,
            &screen_descriptor,
        );
    }
    for id in &full_output.textures_delta.free {
        renderer.free_texture(id);
    }
    let unpadded_bytes_per_row = width * 4;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(256) * 256;
    let readback = device.create_buffer(&eframe::wgpu::BufferDescriptor {
        label: Some("sublens-offscreen-readback"),
        size: padded_bytes_per_row as u64 * height as u64,
        usage: eframe::wgpu::BufferUsages::COPY_DST | eframe::wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        eframe::wgpu::ImageCopyTexture {
            texture: &target,
            mip_level: 0,
            origin: eframe::wgpu::Origin3d::ZERO,
            aspect: eframe::wgpu::TextureAspect::All,
        },
        eframe::wgpu::ImageCopyBuffer {
            buffer: &readback,
            layout: eframe::wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        eframe::wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    let encoded = encoder.finish();
    queue.submit(user_command_buffers.into_iter().chain([encoded]));
    device.poll(eframe::wgpu::Maintain::Wait);
    let slice = readback.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(eframe::wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device.poll(eframe::wgpu::Maintain::Wait);
    receiver.recv()??;
    let data = slice.get_mapped_range();
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for row in data
        .chunks(padded_bytes_per_row as usize)
        .take(height as usize)
    {
        pixels.extend_from_slice(&row[..unpadded_bytes_per_row as usize]);
    }
    let file = File::create(path)?;
    let mut png = png::Encoder::new(file, width, height);
    png.set_color(png::ColorType::Rgba);
    png.set_depth(png::BitDepth::Eight);
    png.write_header()?.write_image_data(&pixels)?;
    drop(data);
    readback.unmap();
    Ok(())
}
