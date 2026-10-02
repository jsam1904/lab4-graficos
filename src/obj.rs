use std::fs;

/// Modelo OBJ cargado: vértices y triángulos (tres índices por triángulo).
pub struct Model {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<usize>, // de 3 en 3: cada grupo es un triángulo
}

impl Model {
    pub fn load(path: &str) -> Result<Model, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("No se pudo leer {path}: {e}"))?;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for line in text.lines() {
            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("v") => {
                    let c: Vec<f32> = parts.take(3).filter_map(|s| s.parse().ok()).collect();
                    if c.len() == 3 {
                        vertices.push([c[0], c[1], c[2]]);
                    }
                }
                Some("f") => {
                    // Formato "v/vt/vn": solo nos interesa el índice de vértice (1-based).
                    let face: Vec<usize> = parts
                        .filter_map(|tok| tok.split('/').next()?.parse::<usize>().ok())
                        .map(|i| i - 1)
                        .collect();
                    // Triangulación en abanico: un quad produce 2 triángulos.
                    for i in 1..face.len().saturating_sub(1) {
                        indices.push(face[0]);
                        indices.push(face[i]);
                        indices.push(face[i + 1]);
                    }
                }
                _ => {}
            }
        }

        if vertices.is_empty() {
            return Err("El OBJ no contiene vértices".into());
        }
        Ok(Model { vertices, indices })
    }

    /// Caja envolvente en x,y: (min_x, max_x, min_y, max_y)
    pub fn bounds_xy(&self) -> (f32, f32, f32, f32) {
        let mut b = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for v in &self.vertices {
            b.0 = b.0.min(v[0]);
            b.1 = b.1.max(v[0]);
            b.2 = b.2.min(v[1]);
            b.3 = b.3.max(v[1]);
        }
        b
    }
}