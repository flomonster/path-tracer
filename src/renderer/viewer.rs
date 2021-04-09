use crate::config::Resolution;
use sfml::graphics::*;
use sfml::window::*;
use std::sync::mpsc::{channel, Receiver, Sender};
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

    pub fn send_pixel_update(sender: &SenderPixel, x: u32, y: u32, color: [u8; 3]) {
        let color = Color::rgb(color[0], color[1], color[2]);
        sender
            .send((x, y, color))
            .expect("Couldn't send pixel update");
    }

    pub fn wait_for_close(self) {
        self.thread.join().expect("couldn't join thread");
    }
}

fn run(resolution: Resolution, recv: ReceiverPixel) {
    let mut window =
        RenderWindow::new((600, 600), "Path Tracer", Style::CLOSE, &Default::default());
    let mut image = Image::new(resolution.width, resolution.height);

    // The main loop - ends as soon as the window is closed
    while window.is_open() {
        // Event processing
        while let Some(event) = window.poll_event() {
            // Request closing for the window
            if event == Event::Closed {
                window.close();
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
        let texture = Texture::from_image(&image).unwrap();
        let sprite = Sprite::with_texture_and_rect(&texture, &Rect::new(0, 0, 600, 600));
        window.draw(&sprite);

        // End the current frame and display its contents on screen
        window.display();
    }
}
