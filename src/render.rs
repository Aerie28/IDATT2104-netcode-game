use minifb::{ Window, WindowOptions};


pub struct Renderer {
    pub window: Window,
    buffer: Vec<u32>,
    width: usize,
    height: usize,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            window: Window::new("Netcode Game", width, height, WindowOptions::default()).unwrap(),
            buffer: vec![0; width * height],
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.iter_mut().for_each(|p| *p = 0x000000);
    }

    pub fn draw_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        for dy in 0..h {
            for dx in 0..w {
                let px_x = x + dx;
                let px_y = y + dy;

                if px_x < self.width && px_y < self.height {
                    let px = px_y * self.width + px_x;
                    self.buffer[px] = color;
                }
            }
        }
    }

    pub fn update(&mut self) {
        self.window.update_with_buffer(&self.buffer, self.width, self.height).unwrap();
    }
}