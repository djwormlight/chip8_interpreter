mod chip8;

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::num::NonZeroU32;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::{env, thread};

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::DisplayHandle;
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::{Window, WindowId};

use softbuffer::{Context, Surface};

use chip8::Interpreter;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_rom> [--debug]", args[0]);

        std::process::exit(1);
    }

    let rom_path = &args[1];
    let debug: bool = args.contains(&"--debug".to_string());

    println!("Rom file: {}", rom_path);
    println!("Debug mode: {}", debug);

    let rom_data: Vec<u8> = fs::read(rom_path).unwrap();

    let (sender, receiver) = channel();

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Application::new(&event_loop, receiver);

    let event_loop_proxy = event_loop.create_proxy();

    let _interpreter_thread = thread::spawn(move || {
        let mut interpreter = Interpreter::new();

        interpreter.load_program(&rom_data).unwrap();

        loop {
            interpreter.execute_cycle();

            sender.send(interpreter.memory).unwrap();

            event_loop_proxy.send_event(UserEvent::RedrawScreen).expect("Failed to send event");

            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    event_loop.run_app(&mut app).map_err(Into::into)
}

#[derive(Debug)]
enum UserEvent {
    RedrawScreen,
}

struct WindowState {
    surface: Surface<DisplayHandle<'static>, Arc<Window>>,
    window: Arc<Window>,
}

impl WindowState {
    fn new(app: &Application, window: Window) -> Result<Self, Box<dyn Error>> {
        let window = Arc::new(window);

        let surface = Surface::new(app.context.as_ref().unwrap(), Arc::clone(&window))?;

        // TODO: add a cursor?

        // TODO: Do I need to handle IME?

        let size = window.inner_size();

        let mut state = Self { surface, window };

        state.resize(size);

        Ok(state)
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        let (width, height) = match (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
            (Some(width), Some(height)) => (width, height),
            _ => return,
        };

        println!("Resize {width} {height}");

        self.surface
            .resize(width, height)
            .expect("failed to resize inner buffer");

        self.window.request_redraw();
    }

    fn draw(&mut self, memory: &[u8; 4096]) -> Result<(), Box<dyn Error>> {
        let size = self.window.inner_size();

        let scale_x = size.width / 64;
        let scale_y = size.height / 32;

        let mut buffer = self.surface.buffer_mut()?;

        buffer.fill(0xff000000);

        let offset = 0xF00;

        for index in offset..offset + (64 * 32 / 8) {
            let y = (index - offset) / (64 / 8);
            let x = (index - offset) % (64 / 8) * 8;

            let byte = memory[index as usize];

            for bit in 0..8 {
                let pixel = (byte >> (7 - bit)) & 0x1;

                if pixel == 1 {
                    let start_x = (x + bit) * scale_x;
                    let start_y = y * scale_y;

                    // Draw the scaled pixel
                    for draw_y in start_y..start_y + scale_y {
                        for draw_x in start_x..start_x + scale_x {
                            let index =
                                (draw_y as usize) * (size.width as usize) + (draw_x as usize);

                            buffer[index] = 0xFFFFFFFF;
                        }
                    }
                }
            }
        }

        self.window.pre_present_notify();

        buffer.present()?;

        Ok(())
    }
}

struct Application {
    windows: HashMap<WindowId, WindowState>,
    context: Option<Context<DisplayHandle<'static>>>,
    receiver: Receiver<[u8; 4096]>,
}

impl Application {
    fn new<T>(event_loop: &EventLoop<T>, receiver: Receiver<[u8; 4096]>) -> Self {
        let context = Some(
            Context::new(unsafe {
                std::mem::transmute::<DisplayHandle<'_>, DisplayHandle<'static>>(
                    event_loop.display_handle().unwrap(),
                )
            })
            .unwrap(),
        );

        Self {
            context,
            windows: Default::default(),
            receiver,
        }
    }

    fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        _tab_id: Option<String>,
    ) -> Result<WindowId, Box<dyn Error>> {
        let scaling_factor = 20;

        let window_attributes = Window::default_attributes()
            .with_title("Chip8 Interpreter")
            .with_inner_size(LogicalSize::new(64 * scaling_factor, 32 * scaling_factor))
            .with_transparent(true);

        let window = event_loop.create_window(window_attributes)?;

        let window_state = WindowState::new(self, window)?;

        let window_id = window_state.window.id();

        self.windows.insert(window_id, window_state);

        Ok(window_id)
    }
}

impl ApplicationHandler<UserEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop, None)
            .expect("failed to create the initial window");
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, user_event: UserEvent) {
        match user_event {
            UserEvent::RedrawScreen => {
                for window_state in self.windows.values_mut() {
                    window_state.window.request_redraw();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let window_state = match self.windows.get_mut(&window_id) {
            Some(window_state) => window_state,
            None => return,
        };

        match event {
            WindowEvent::Resized(size) => {
                window_state.resize(size);
            }

            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");

                self.windows.remove(&window_id);
            }

            WindowEvent::RedrawRequested => {
                println!("Redraw requested");

                if let Ok(memory) = self.receiver.try_recv() {
                    // TODO: Handle error here correctly
                    window_state.draw(&memory).unwrap();
                }
            }
            _ => (),
        }
    }
}
