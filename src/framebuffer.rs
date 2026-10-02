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
/// Fuente de 5x7 píxeles: cada fila es un número de 5 bits (el bit más alto es la columna izquierda).
fn glyph(ch: char) -> Option<[u8; 7]> {
    Some(match ch {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        _ => return None,
    })
}

impl Framebuffer {
    /// Ancho en píxeles de `text` escrito con `text()` a escala `scale`.
    pub fn text_width(text: &str, scale: i32) -> i32 {
        (text.chars().count() as i32 * 6 - 1).max(0) * scale
    }

    /// Escribe texto en mayúsculas con la fuente de 5x7; cada píxel de la fuente es un cuadro de `scale`.
    /// Los caracteres que no están en la fuente se dejan como espacio.
    pub fn text(&mut self, x: i32, y: i32, text: &str, scale: i32, c: [u8; 4]) {
        for (n, ch) in text.chars().enumerate() {
            let Some(rows) = glyph(ch.to_ascii_uppercase()) else { continue };
            let ox = x + n as i32 * 6 * scale;
            for (row, bits) in rows.iter().enumerate() {
                for col in 0..5 {
                    if bits & (0b10000 >> col) != 0 {
                        for dy in 0..scale {
                            for dx in 0..scale {
                                self.put_pixel(ox + col * scale + dx, y + row as i32 * scale + dy, c);
                            }
                        }
                    }
                }
            }
        }
    }
}
