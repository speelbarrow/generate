use futures_time::{future::FutureExt, time::Duration};
use log::{Level, error, warn};
use std::sync::{Arc, OnceLock};
use wgpu::{
    Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureUsages,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::OsError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy, OwnedDisplayHandle},
    window::{Window, WindowId},
};

#[derive(Debug)]
enum Event {
    Initialize(Result<State, Box<dyn std::error::Error + Send + Sync>>),
}

#[derive(Debug)]
struct State {
    configuration: SurfaceConfiguration,
    device: Device,
    surface: Surface<'static>,
    queue: Queue,
    window: Arc<Window>,
}
impl State {
    async fn initialize(
        handle: OwnedDisplayHandle,
        proxy: EventLoopProxy<Event>,
        window: Result<Window, OsError>,
    ) {
        if let Err(error) = proxy.send_event(Event::Initialize(
            async move {
                let window = Arc::new(window?);

                let instance = Instance::new(InstanceDescriptor::new_with_display_handle_from_env(
                    Box::new(handle),
                ));

                /*
                Sometimes the `surface` is not initially available when requested. This
                implementation leverages `futures_time` to give the surface a full second to be
                available, and errors out otherwise.
                */
                let Ok(surface) = async {
                    loop {
                        if let Ok(surface) = instance.create_surface(window.clone()) {
                            break surface;
                        }
                    }
                }
                .timeout(Duration::from_secs(1))
                .await
                else {
                    return Err("failed to create surface".into());
                };

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
                    surface,
                    queue,
                    window,
                })
            }
            .await,
        )) {
            error!(
                "failed to send user event to event loop with error:{}\n{:#?}",
                error, error.0
            );
        }
    }

    fn resize(&mut self, PhysicalSize { height, width }: PhysicalSize<u32>) {}
    fn render(&mut self) {
        self.window.request_redraw();
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

    fn initialize(&self, event_loop: &ActiveEventLoop) {
        let (handle, proxy, window) = (
            event_loop.owned_display_handle(),
            self.proxy.clone(),
            event_loop.create_window(Default::default()),
        );
        self.spawn(State::initialize(handle, proxy, window));
    }
}
impl ApplicationHandler<Event> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.get().is_none() {
            self.initialize(event_loop);
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Event) {
        if let Err(error) = match event {
            Event::Initialize(Ok(state)) => self
                .state
                .set(state)
                .map_err(|_| "cannot reinitialize application state".into()),
            Event::Initialize(Err(error)) => Err(error),
        } {
            error!(
                "failed to initialize application state with error: {}",
                error
            );
            event_loop.exit();
            return;
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let state = self.state.get_mut();
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) if let Some(state) = state => state.resize(size),
            WindowEvent::RedrawRequested if let Some(state) = state => state.render(),
            WindowEvent::Resized(..) | WindowEvent::RedrawRequested if state.is_none() => {
                warn!(
                    "cannot process window events before the application state has been initialized"
                )
            }
            _ => {}
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn main() {
    const LEVEL: Level = Level::Info;
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
