use crate::config::Resolution;
use sfml::graphics::*;
use sfml::window::*;
use std::sync::mpsc::{channel, Receiver, SendError, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

type ReceiverPixel = Receiver<(u32, u32, Color)>;
pub type SenderPixel = Sender<(u32, u32, Color)>;

pub struct Viewer {
    thread: JoinHandle<()>,
    pub sender: SenderPixel,
}

impl Viewer {
    pub fn create(resolution: Resolution) -> Self {
        let (sender, recv) = channel();
        let thread = thread::spawn(move || run(resolution, recv));
        Viewer { thread, sender }
    }

    pub fn send_pixel_update(
        sender: &SenderPixel,
        x: u32,
        y: u32,
        color: [u8; 3],
    ) -> Result<(), SendError<(u32, u32, Color)>> {
        let color = Color::rgb(color[0], color[1], color[2]);
        sender.send((x, y, color))
    }

    pub fn wait_for_close(self) {
        self.thread.join().expect("couldn't join thread");
    }
}

fn run(resolution: Resolution, recv: ReceiverPixel) {
    let mut window = RenderWindow::new(
        (resolution.width, resolution.height),
        "Path Tracer",
        Style::CLOSE | Style::RESIZE | Style::TITLEBAR,
        &Default::default(),
    );
    let mut image = Image::new(resolution.width, resolution.height);
    let mut texture = Texture::new(resolution.width, resolution.height).unwrap();
    let mut view = View::from_rect(&FloatRect::new(
        0.,
        0.,
        resolution.width as f32,
        resolution.height as f32,
    ));
    let mut view_center = (resolution.width as f32 / 2., resolution.height as f32 / 2.);
    let mut zoom = 1.;

    // The main loop - ends as soon as the window is closed
    while window.is_open() {
        // Event processing
        while let Some(event) = window.poll_event() {
            // Request closing for the window
            match event {
                Event::Closed => window.close(),
                Event::Resized { width, height } => {
                    view.set_size((width as f32, height as f32));
                    view.zoom(zoom);
                    view.set_center(view_center);
                }
                Event::MouseWheelScrolled { delta, x, y, .. } => {
                    let zoom_factor = -delta / 20.;
                    zoom *= 1. + zoom_factor;
                    view.zoom(1. + zoom_factor);
                    view_center = (
                        view_center.0 + (x as f32 - view_center.0) * f32::abs(delta) / 20.,
                        view_center.1 + (y as f32 - view_center.1) * f32::abs(delta) / 20.,
                    );
                    view.set_center(view_center);
                }
                _ => {}
            }
        }

        let start = Instant::now();
        for (pos_x, pos_y, color) in recv.try_iter() {
            image.set_pixel(pos_x, pos_y, color);
            if start.elapsed() > Duration::from_secs(1) / 60 {
                break;
            }
        }

        // Draw the image
        window.clear(Color::BLACK);
        window.set_view(&view);
        texture.update_from_image(&image, 0, 0);
        let sprite = Sprite::with_texture(&texture);
        window.draw(&sprite);

        // End the current frame and display its contents on screen
        window.display();
    }
}
