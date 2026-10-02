mod framebuffer;
mod obj;

use framebuffer::Framebuffer;
use obj::Model;
use raylib::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

const WIDTH: i32 = 900;
const HEIGHT: i32 = 700;
const MARGIN: f32 = 0.88; // el modelo ocupa ~88% de su área
const BG: [u8; 4] = [14, 16, 24, 255];
const FG: [u8; 4] = [120, 220, 255, 255];
const GRID: [u8; 4] = [45, 52, 70, 255];
const TITLE: [u8; 4] = [235, 240, 250, 255];
const ROT_SPEED: f32 = 1.5; // radianes por segundo con las flechas

/// Orientación de la cámara: giro en y (yaw) y luego inclinación en x (pitch).
#[derive(Clone, Copy)]
struct View {
    name: &'static str,
    yaw: f32,
    pitch: f32,
}

/// Vistas fijas. La cámara mira hacia -z después de rotar; con yaw = 0 se ve el frente
/// (la nave apunta hacia +z y su lado derecho queda en -x).
const VIEWS: [View; 7] = [
    View { name: "Frente", yaw: 0.0, pitch: 0.0 },
    View { name: "Atrás", yaw: PI, pitch: 0.0 },
    View { name: "Izquierda", yaw: -FRAC_PI_2, pitch: 0.0 },
    View { name: "Derecha", yaw: FRAC_PI_2, pitch: 0.0 },
    View { name: "Arriba", yaw: PI, pitch: FRAC_PI_2 },
    View { name: "Abajo", yaw: PI, pitch: -FRAC_PI_2 },
    View { name: "Isométrica", yaw: FRAC_PI_4, pitch: 0.6 },
];

/// Vistas que aparecen en la cuadrícula de 3x2 (índices de VIEWS).
const GRID_VIEWS: [usize; 6] = [0, 1, 3, 2, 4, 6];

/// Área de pantalla donde se dibuja una vista: (x, y, ancho, alto).
type Rect = (i32, i32, i32, i32);

/// Rota un vértice según la vista: primero en y, luego en x.
fn rotate(v: &[f32; 3], view: &View) -> [f32; 3] {
    let (sy, cy) = view.yaw.sin_cos();
    let (sp, cp) = view.pitch.sin_cos();
    let x = v[0] * cy + v[2] * sy;
    let z = -v[0] * sy + v[2] * cy;
    let y = v[1] * cp - z * sp;
    let z = v[1] * sp + z * cp;
    [x, y, z]
}

/// Dibuja el modelo visto desde `view` dentro de `rect` (proyección ortográfica).
/// Si `fixed_scale` es true, la escala sale de la esfera envolvente para que no cambie al rotar.
fn render_view(model: &Model, fb: &mut Framebuffer, view: &View, rect: Rect, fixed_scale: bool) {
    let (min, max) = model.bounds();
    let center = [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0, (min[2] + max[2]) / 2.0];

    // Vértices centrados en el origen y rotados.
    let pts: Vec<[f32; 3]> = model
        .vertices
        .iter()
        .map(|v| rotate(&[v[0] - center[0], v[1] - center[1], v[2] - center[2]], view))
        .collect();

    // Caja envolvente en x,y de la vista rotada, y rango de z para el sombreado.
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in &pts {
        for i in 0..3 {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    }

    let (rx, ry, rw, rh) = rect;
    let (scale, cx, cy) = if fixed_scale {
        let radius = pts.iter().map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()).fold(0.0, f32::max);
        (rw.min(rh) as f32 / (2.0 * radius) * MARGIN, 0.0, 0.0)
    } else {
        let s = (rw as f32 / (hi[0] - lo[0])).min(rh as f32 / (hi[1] - lo[1])) * MARGIN;
        (s, (lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0)
    };

    let to_screen = |p: &[f32; 3]| -> (i32, i32) {
        let x = (p[0] - cx) * scale + rx as f32 + rw as f32 / 2.0;
        let y = ry as f32 + rh as f32 / 2.0 - (p[1] - cy) * scale; // y de pantalla crece hacia abajo
        (x.round() as i32, y.round() as i32)
    };

    // Recorrer triángulos: 3 índices -> 3 vértices; se guarda su profundidad media.
    let mut tris = Vec::new();
    for t in model.indices.chunks_exact(3) {
        let (a, b, c) = (&pts[t[0]], &pts[t[1]], &pts[t[2]]);
        let depth = (a[2] + b[2] + c[2]) / 3.0;
        tris.push((depth, to_screen(a), to_screen(b), to_screen(c)));
    }

    // Lo más lejano primero, y más oscuro, para que se note qué está adelante.
    tris.sort_by(|x, y| x.0.total_cmp(&y.0));
    let z_range = (hi[2] - lo[2]).max(f32::EPSILON);
    for (depth, a, b, c) in tris {
        let k = 0.3 + 0.7 * (depth - lo[2]) / z_range;
        let color = [
            (FG[0] as f32 * k) as u8,
            (FG[1] as f32 * k) as u8,
            (FG[2] as f32 * k) as u8,
            255,
        ];
        fb.triangle(a, b, c, color);
    }
}

/// Celda `i` de la cuadrícula de 3 columnas x 2 filas.
fn grid_cell(i: usize) -> Rect {
    let (w, h) = (WIDTH / 3, HEIGHT / 2);
    ((i as i32 % 3) * w, (i as i32 / 3) * h, w, h)
}

/// Escribe el nombre de la vista centrado arriba de `rect` y dibuja el modelo en el espacio de abajo.
fn render_titled(model: &Model, fb: &mut Framebuffer, view: &View, rect: Rect, fixed_scale: bool, text_scale: i32) {
    let (rx, ry, rw, rh) = rect;
    let pad = 4 * text_scale;
    let band = 7 * text_scale + 2 * pad; // alto de la franja del título
    let name = without_accents(view.name);
    let tx = rx + (rw - Framebuffer::text_width(&name, text_scale)) / 2;
    fb.text(tx, ry + pad, &name, text_scale, TITLE);
    render_view(model, fb, view, (rx, ry + band, rw, rh - band), fixed_scale);
}

/// La fuente de píxeles solo tiene letras sin tilde.
fn without_accents(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'Á' => 'A',
            'é' | 'É' => 'E',
            'í' | 'Í' => 'I',
            'ó' | 'Ó' => 'O',
            'ú' | 'Ú' => 'U',
            _ => c,
        })
        .collect()
}

/// Dibuja las vistas de la cuadrícula, cada una con su título, y los separadores.
fn render_grid(model: &Model, fb: &mut Framebuffer) {
    fb.clear(BG);
    for (i, &v) in GRID_VIEWS.iter().enumerate() {
        render_titled(model, fb, &VIEWS[v], grid_cell(i), false, 3);
    }
    let (w, h) = (WIDTH / 3, HEIGHT / 2);
    fb.line(w, 0, w, HEIGHT - 1, GRID);
    fb.line(2 * w, 0, 2 * w, HEIGHT - 1, GRID);
    fb.line(0, h, WIDTH - 1, h, GRID);
}

/// Qué se muestra en la ventana.
enum Mode {
    Grid,
    Single(View),
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

    // Modo captura: exporta la cuadrícula con todas las vistas a PNG sin abrir ventana.
    if screenshot {
        render_grid(&model, &mut fb);
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

    let number_keys = [
        KeyboardKey::KEY_ONE,
        KeyboardKey::KEY_TWO,
        KeyboardKey::KEY_THREE,
        KeyboardKey::KEY_FOUR,
        KeyboardKey::KEY_FIVE,
        KeyboardKey::KEY_SIX,
        KeyboardKey::KEY_SEVEN,
    ];
    let mut mode = Mode::Grid;

    while !rl.window_should_close() {
        // Entrada: 0/Tab = todas las vistas, 1-7 = una vista, flechas = rotar.
        if rl.is_key_pressed(KeyboardKey::KEY_ZERO) || rl.is_key_pressed(KeyboardKey::KEY_TAB) {
            mode = Mode::Grid;
        }
        for (i, &k) in number_keys.iter().enumerate() {
            if rl.is_key_pressed(k) {
                mode = Mode::Single(VIEWS[i]);
            }
        }
        let dt = rl.get_frame_time();
        let mut d_yaw = 0.0;
        let mut d_pitch = 0.0;
        if rl.is_key_down(KeyboardKey::KEY_LEFT) { d_yaw -= ROT_SPEED * dt; }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { d_yaw += ROT_SPEED * dt; }
        if rl.is_key_down(KeyboardKey::KEY_UP) { d_pitch += ROT_SPEED * dt; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) { d_pitch -= ROT_SPEED * dt; }
        if d_yaw != 0.0 || d_pitch != 0.0 {
            // Desde la cuadrícula, rotar arranca en la vista isométrica.
            let base = match mode {
                Mode::Grid => VIEWS[6],
                Mode::Single(v) => v,
            };
            mode = Mode::Single(View {
                name: "Libre",
                yaw: base.yaw + d_yaw,
                pitch: (base.pitch + d_pitch).clamp(-FRAC_PI_2, FRAC_PI_2),
            });
        }

        // Render en CPU y subida del framebuffer a la textura.
        match &mode {
            Mode::Grid => render_grid(&model, &mut fb),
            Mode::Single(v) => {
                fb.clear(BG);
                render_titled(&model, &mut fb, v, (0, 0, WIDTH, HEIGHT - 30), v.name == "Libre", 4);
            }
        }
        tex.update_texture(&fb.pixels).expect("update");

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex, 0, 0, Color::WHITE);
        d.draw_text("0/Tab: todas   1-7: una vista   Flechas: rotar", 10, HEIGHT - 24, 16, Color::GRAY);
    }
}
