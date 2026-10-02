/// Framebuffer propio: un arreglo de píxeles RGBA en CPU.
pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u8>, // RGBA, 4 bytes por píxel
}

impl Framebuffer {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height, pixels: vec![0; (width * height * 4) as usize] }
    }

    pub fn clear(&mut self, c: [u8; 4]) {
        for px in self.pixels.chunks_exact_mut(4) {
            px.copy_from_slice(&c);
        }
    }

    pub fn put_pixel(&mut self, x: i32, y: i32, c: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let i = ((y * self.width + x) * 4) as usize;
        self.pixels[i..i + 4].copy_from_slice(&c);
    }

    /// Línea de Bresenham.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: [u8; 4]) {
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.put_pixel(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Triángulo (contorno): tres líneas entre sus vértices en pantalla.
    pub fn triangle(&mut self, a: (i32, i32), b: (i32, i32), c: (i32, i32), color: [u8; 4]) {
        self.line(a.0, a.1, b.0, b.1, color);
        self.line(b.0, b.1, c.0, c.1, color);
        self.line(c.0, c.1, a.0, a.1, color);
    }
}