use log::{Level, error, trace, warn};
use std::sync::{Arc, OnceLock};
use wgpu::{
    Color, CurrentSurfaceTexture, Device, Instance, InstanceDescriptor, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration, TextureUsages,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowAttributes, WindowId},
};

#[derive(Debug)]
enum Event {
    Initialize(Result<State, Box<dyn std::error::Error + Send + Sync>>),
}
#[derive(Debug)]
struct State {
    configuration: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    window: Arc<Window>,
}
impl State {
    async fn initialize(
        proxy: EventLoopProxy<Event>,
        surface: Surface<'static>,
        window: Arc<Window>,
        instance: Instance,
    ) {
        let event = async move {
            let adapter = instance
                .request_adapter(&RequestAdapterOptions {
                    compatible_surface: Some(&surface),
                    ..Default::default()
                })
                .await?;
            let (capabilities, (device, queue), PhysicalSize { height, width }) = (
                surface.get_capabilities(&adapter),
                adapter.request_device(&Default::default()).await?,
                window.inner_size(),
            );
            let configuration = SurfaceConfiguration {
                alpha_mode: capabilities.alpha_modes[0],
                desired_maximum_frame_latency: 2,
                format: capabilities.formats[0],
                present_mode: capabilities.present_modes[0],
                usage: TextureUsages::RENDER_ATTACHMENT,
                view_formats: vec![],

                height,
                width,
            };
            surface.configure(&device, &configuration);

            Ok(Self {
                configuration,
                device,
                queue,
                surface,
                window,
            })
        }
        .await;

        let trace = format!("{:#?}", event);
        if let Err(error) = proxy.send_event(Event::Initialize(event)) {
            error!(
                "failed to send user event to event loop with error:{}\n{:#?}",
                error, error.0
            );
        } else {
            trace!("sent user event to event loop\n{}", trace);
        }
    }

    fn resize(&mut self, PhysicalSize { height, width }: PhysicalSize<u32>) {
        if height > 0 && width > 0 {
            self.configuration.height = height;
            self.configuration.width = width;
            self.surface.configure(&self.device, &self.configuration);
        }
    }
    fn render(&mut self) -> Result<(), ()> {
        self.window.request_redraw();

        let frame = {
            let current = self.surface.get_current_texture();
            if let CurrentSurfaceTexture::Suboptimal(_) | CurrentSurfaceTexture::Outdated = current
            {
                self.surface.configure(&self.device, &self.configuration);
            }
            match current {
                CurrentSurfaceTexture::Success(texture)
                | CurrentSurfaceTexture::Suboptimal(texture) => texture,
                current @ (CurrentSurfaceTexture::Outdated
                | CurrentSurfaceTexture::Timeout
                | CurrentSurfaceTexture::Occluded
                | CurrentSurfaceTexture::Validation) => {
                    trace!("bad frame: {:#?}", current);
                    return Ok(());
                }
                CurrentSurfaceTexture::Lost => {
                    return Err(());
                }
            }
        };
        let (mut encoder, view) = (
            self.device.create_command_encoder(&Default::default()),
            frame.texture.create_view(&Default::default()),
        );
        {
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.,
                        }),
                        store: StoreOp::Store,
                    },
                    view: &view,

                    depth_slice: None,
                    resolve_target: None,
                })],
                ..Default::default()
            });
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }
}
struct App {
    #[cfg(not(target_arch = "wasm32"))]
    pool: futures::executor::ThreadPool,
    proxy: EventLoopProxy<Event>,
    state: OnceLock<State>,
}
impl App {
    pub fn new(proxy: EventLoopProxy<Event>) -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            pool: futures::executor::ThreadPool::new().expect("failed to create thread pool"),
            proxy,
            state: Default::default(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(future);
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.pool.spawn_ok(future);
    }

    fn initialize(&self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn std::error::Error>> {
        let (handle, proxy, window) = (
            event_loop.owned_display_handle(),
            self.proxy.clone(),
            Arc::new(event_loop.create_window({
                #[cfg_attr(not(target_arch = "wasm32"), expect(unused_mut))]
                let mut attributes = WindowAttributes::default();
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen::JsCast;
                    use winit::platform::web::WindowAttributesExtWebSys;
                    if let Some(canvas) = wgpu::web_sys::window()
                        .and_then(|window| window.document())
                        .and_then(|document| document.query_selector("canvas").unwrap_or(None))
                        .and_then(|canvas| {
                            canvas.dyn_into::<wgpu::web_sys::HtmlCanvasElement>().ok()
                        })
                    {
                        attributes = attributes.with_canvas(Some(canvas));
                    } else {
                        return Err("no canvas".into());
                    }
                }
                attributes
            })?),
        );
        let instance = Instance::new(InstanceDescriptor::new_with_display_handle_from_env(
            Box::new(handle),
        ));
        self.spawn(State::initialize(
            proxy,
            instance.create_surface(window.clone())?,
            window,
            instance,
        ));
        Ok(())
    }
}
impl ApplicationHandler<Event> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.get().is_none() {
            if let Err(error) = self.initialize(event_loop) {
                error!("failed to initialize with error: {}", error);
                event_loop.exit();
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Event) {
        match match event {
            Event::Initialize(Ok(state)) => {
                let window = state.window.clone();
                self.state
                    .set(state)
                    .map(|_| window)
                    .map_err(|_| "cannot reinitialize application state".into())
            }
            Event::Initialize(Err(error)) => Err(error),
        } {
            Ok(window) => window.request_redraw(),
            Err(error) => {
                error!(
                    "failed to initialize application state with error: {}",
                    error
                );
                event_loop.exit();
                return;
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let state = self.state.get_mut();
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) if let Some(state) = state => state.resize(size),
            WindowEvent::RedrawRequested if let Some(state) = state => {
                if let Err(_) = state.render() {
                    error!("lost device");
                    event_loop.exit();
                    return;
                }
            }
            WindowEvent::Resized(..) | WindowEvent::RedrawRequested if state.is_none() => {
                warn!(
                    concat!(
                        "cannot process window events before the application state has been ",
                        "initialized\n{:#?}"
                    ),
                    event
                );
                return;
            }
            _ => return,
        }
        trace!("handled {:#?}", event);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn main() {
    const LEVEL: Level = if cfg!(debug_assertions) {
        Level::Debug
    } else {
        Level::Info
    };
    cfg_select! {
        target_arch = "wasm32" => {
            console_error_panic_hook::set_once();
            console_log::init_with_level(LEVEL).unwrap();
        }
        _ => {
            env_logger::builder()
                .filter_level(LEVEL.to_level_filter())
                .parse_default_env()
                .init();
        }
    }

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("failed to create event loop");
    let proxy = event_loop.create_proxy();

    #[cfg_attr(target_arch = "wasm32", expect(unused_mut))]
    let mut app = App::new(proxy);

    if let Err(error) = cfg_select! {
        target_arch = "wasm32" => { {
            use winit::{error::EventLoopError, platform::web::EventLoopExtWebSys};
            event_loop.spawn_app(app);
            Ok::<_, EventLoopError>(())
        } }
        _ => {
            event_loop.run_app(&mut app)
        }
    } {
        error!("{}", error);
    }
}
