use pixels::{wgpu, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

struct World {
    // Source image
    img: Vec<u8>,
    width: usize,
    height: usize,

    // Position where it will be blitted to the pixel buffer.
    x: usize,
    y: usize,

    // Background animation state
    intensity: f64,
    darker: bool,
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Alpha")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);

        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::default();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => {
                pixels.resize_surface(size.width, size.height);
            }
            _ => {}
        },

        Event::RedrawRequested(_) => {
            // Draw the image to the pixel buffer.
            // Pixel animations typically happen here, but not in this demo!
            world.draw(pixels.get_frame());

            // Put the pixel buffer on the screen.
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
            }
        }

        Event::MainEventsCleared => {
            // Update the clear color, just to show that blending works.
            pixels.set_clear_color(world.update());

            window.request_redraw();
        }

        _ => {}
    });
}

impl Default for World {
    fn default() -> Self {
        let bytes = include_bytes!("hello.png");

        // This was just copied from the `png` docs...
        // Who cares how it works or why it's so verbose!
        let decoder = png::Decoder::new(&bytes[..]);
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let img = buf[..info.buffer_size()].to_vec();

        let width = reader.info().width as usize;
        let height = reader.info().height as usize;

        Self {
            img,
            width,
            height,
            x: (WIDTH as usize - width) / 2,
            y: (HEIGHT as usize - height) / 2,
            intensity: 0.0,
            darker: false,
        }
    }
}

impl World {
    fn update(&mut self) -> wgpu::Color {
        let incr = if self.darker { -1.0 } else { 1.0 };

        let mut intensity = self.intensity + incr * 0.005;
        if intensity < 0.0 {
            intensity = 0.0;

            self.darker = false;
        } else if intensity > 1.0 {
            intensity = 1.0;

            self.darker = true;
        }

        self.intensity = intensity;

        // Intensity is a linear value. `powf()` transforms it into the non-linear sRGB space.
        intensity = intensity.powf(2.2);

        wgpu::Color {
            r: intensity,
            g: intensity,
            b: intensity,
            a: 1.0,
        }
    }

    fn draw(&self, buffer: &mut [u8]) {
        // Clear the buffer to transparent black.
        buffer.fill(0);

        // Draw the image into the buffer.
        blit(buffer, self.x, self.y, &self.img, self.width, self.height);
    }
}

fn blit(buffer: &mut [u8], x: usize, y: usize, img: &[u8], width: usize, height: usize) {
    let dw = WIDTH as usize;
    let stride = width * 4;

    for sy in 0..height {
        let di = ((sy + y) * dw + x) * 4;
        let si = sy * width * 4;

        // Buffer and img have the same pixel format, so we can just copy without conversion.
        let dest = &mut buffer[di..di + stride];
        dest.copy_from_slice(&img[si..si + stride])
    }
}
