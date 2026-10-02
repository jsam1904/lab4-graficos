mod framebuffer;
mod obj;

use framebuffer::Framebuffer;
use obj::Model;
use raylib::prelude::*;

const WIDTH: i32 = 900;
const HEIGHT: i32 = 700;
const MARGIN: f32 = 0.88; // el modelo ocupa ~88% de la ventana
const BG: [u8; 4] = [14, 16, 24, 255];
const FG: [u8; 4] = [120, 220, 255, 255];

/// Dibuja todos los triángulos del modelo usando solo x,y (sin proyección real).
fn render(model: &Model, fb: &mut Framebuffer) {
    fb.clear(BG);

    // Escala y traslación para centrar la caja envolvente en la ventana.
    let (min_x, max_x, min_y, max_y) = model.bounds_xy();
    let scale = (fb.width as f32 / (max_x - min_x)).min(fb.height as f32 / (max_y - min_y)) * MARGIN;
    let (cx, cy) = ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);

    let to_screen = |v: &[f32; 3]| -> (i32, i32) {
        let x = (v[0] - cx) * scale + fb.width as f32 / 2.0;
        let y = fb.height as f32 / 2.0 - (v[1] - cy) * scale; // y de pantalla crece hacia abajo
        (x.round() as i32, y.round() as i32)
    };

    // Recorrer triángulos: 3 índices -> 3 vértices -> triangle()
    let mut tris = Vec::new();
    for t in model.indices.chunks_exact(3) {
        let a = to_screen(&model.vertices[t[0]]);
        let b = to_screen(&model.vertices[t[1]]);
        let c = to_screen(&model.vertices[t[2]]);
        tris.push((a, b, c));
    }
    for (a, b, c) in tris {
        fb.triangle(a, b, c, FG);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.iter().skip(1).find(|a| !a.starts_with("--")).cloned()
        .unwrap_or_else(|| "assets/Nave_Javier_Alvarado.obj".to_string());
    let screenshot = args.iter().any(|a| a == "--screenshot");

    let model = Model::load(&path).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    println!("OBJ cargado: {} vértices, {} triángulos", model.vertices.len(), model.indices.len() / 3);

    let mut fb = Framebuffer::new(WIDTH, HEIGHT);
    render(&model, &mut fb);

    // Modo captura: exporta el framebuffer a PNG sin abrir ventana.
    if screenshot {
        let mut img = Image::gen_image_color(WIDTH, HEIGHT, Color::BLACK);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let i = ((y * WIDTH + x) * 4) as usize;
                let p = &fb.pixels[i..i + 4];
                img.draw_pixel(x, y, Color::new(p[0], p[1], p[2], p[3]));
            }
        }
        img.export_image("screenshot.png");
        println!("Captura guardada en screenshot.png");
        return;
    }

    let (mut rl, thread) = raylib::init().size(WIDTH, HEIGHT).title("Proyecto 1 - OBJ").build();
    rl.set_target_fps(60);

    // Raylib solo se usa para mostrar el framebuffer propio como textura.
    let img = Image::gen_image_color(WIDTH, HEIGHT, Color::BLACK);
    let mut tex = rl.load_texture_from_image(&thread, &img).expect("textura");
    tex.update_texture(&fb.pixels).expect("update");

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex, 0, 0, Color::WHITE);
    }
}