#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

glium::implement_vertex!(Vertex, position, tex_coords);

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ObjData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}


pub fn parse_obj(data: &str, material_name: Option<&str>) -> ObjData {
    let mut vertices = Vec::new();
    let mut tex_coords = Vec::new();
    let mut indices = Vec::new();
    
    let mut current_material = "";

    for line in data.split('\n') {
        let parts: Vec<&str> = line.split_whitespace().collect();

        match parts.as_slice() {
            ["v", x, y, z] => {
                let vertex = Vertex {
                    position: [x.parse().unwrap(), y.parse().unwrap(), z.parse().unwrap()],
                    tex_coords: [0.0, 0.0], // Default texture coordinates
                };
                vertices.push(vertex);
            }
            ["vt", u, v] => {
                // Store texture coordinates
                tex_coords.push([u.parse().unwrap(), v.parse().unwrap()]);
            }
            ["f", ..] => {
                if material_name.is_none() || material_name.unwrap_or("") == current_material {
                    for &face in &parts[1..] {
                        let indices_parts: Vec<&str> = face.split('/').collect();
                        let vertex_index: usize = indices_parts[0].parse().unwrap();
                        let tex_coord_index: usize = indices_parts.get(1).and_then(|&s| s.parse().ok()).unwrap_or(0); // Get texture coordinate index if available
                        indices.push((vertex_index - 1) as u32); // OBJ indices are 1-based
    
                        // Update the vertex with the corresponding texture coordinate
                        if let Some(tex_coord) = tex_coords.get(tex_coord_index - 1) { // OBJ indices are 1-based
                            vertices[vertex_index - 1].tex_coords = *tex_coord;
                        }
                    }
                }

            }
            ["usemtl", name] => {
                current_material = name;
            }
            _ => {}
        }
    }

    ObjData { vertices, indices }
}
