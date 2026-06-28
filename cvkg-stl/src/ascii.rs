// ASCII STL parser
use crate::error::{StlError, StlMesh};
use std::io::{BufRead, Read};

/// Parse an ASCII STL file.
pub fn parse<R: Read>(reader: R) -> Result<StlMesh, StlError> {
    let reader = std::io::BufReader::new(reader);
    let mut mesh = StlMesh::new();
    let mut current_normal: Option<[f32; 3]> = None;
    let mut current_vertices: Vec<[f32; 3]> = Vec::new();
    let mut in_facet = false;
    let mut in_loop = false;
    let mut found_facet = false;

    let mut found_endsolid = false;

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => return Err(StlError::NotAscii), // UTF-8 error means not ASCII STL
        };
        let line = line.trim();
        let tokens: Vec<&str> = line.split_whitespace().collect();

        if tokens.is_empty() {
            continue;
        }

        match tokens[0].to_ascii_lowercase().as_str() {
            "solid" => {
                // Start of file — optional name follows
                if tokens.len() > 1 {
                    let _name = &tokens[1..].join(" ");
                }
            }
            "facet" => {
                if tokens.len() >= 5 && tokens[1].eq_ignore_ascii_case("normal") {
                    in_facet = true;
                    found_facet = true;
                    current_normal = Some(parse_f33_triple(&tokens[2..], line_num)?);
                    current_vertices.clear();
                } else {
                    return Err(StlError::InvalidAscii(format!(
                        "line {}: expected 'facet normal ...'",
                        line_num + 1
                    )));
                }
            }
            "outer" => {
                if tokens.len() >= 2 && tokens[1].eq_ignore_ascii_case("loop") {
                    if !in_facet {
                        return Err(StlError::InvalidAscii(format!(
                            "line {}: 'outer loop' outside facet",
                            line_num + 1
                        )));
                    }
                    in_loop = true;
                }
            }
            "vertex" => {
                if !in_loop {
                    return Err(StlError::InvalidAscii(format!(
                        "line {}: 'vertex' outside loop",
                        line_num + 1
                    )));
                }
                if tokens.len() >= 4 {
                    let v = parse_f33_triple(&tokens[1..], line_num)?;
                    current_vertices.push(v);
                } else {
                    return Err(StlError::InvalidAscii(format!(
                        "line {}: vertex needs 3 coordinates",
                        line_num + 1
                    )));
                }
            }
            "endloop" => {
                in_loop = false;
            }
            "endfacet" => {
                if !in_facet {
                    return Err(StlError::InvalidAscii(format!(
                        "line {}: 'endfacet' outside facet",
                        line_num + 1
                    )));
                }
                // Emit the triangle(s) from collected vertices
                if current_vertices.len() != 3 {
                    return Err(StlError::NonTriangleFace);
                }
                let normal = current_normal.unwrap_or([0.0, 0.0, 0.0]);
                for &v in &current_vertices {
                    mesh.vertices.push(v);
                    mesh.normals.push(normal);
                    mesh.indices.push(mesh.vertices.len() as u32 - 1);
                }
                in_facet = false;
                current_normal = None;
                current_vertices.clear();
            }
            "endsolid" => {
                // End of file — optional name follows
                found_endsolid = true;
                break;
            }
            _ => {
                // Unknown token — if we haven't found any facet yet, this isn't ASCII
                if !found_facet {
                    return Err(StlError::NotAscii);
                }
                // Otherwise it's junk after facets — ignore
            }
        }
    }

    if mesh.vertices.is_empty() && !found_facet {
        return Err(StlError::InvalidAscii("no facets found".into()));
    }

    if !found_endsolid && found_facet {
        return Err(StlError::InvalidAscii("missing endsolid".into()));
    }

    Ok(mesh)
}

/// Parse 3 f32 values from string tokens.
fn parse_f33_triple(tokens: &[&str], line_num: usize) -> Result<[f32; 3], StlError> {
    if tokens.len() < 3 {
        return Err(StlError::InvalidAscii(format!(
            "line {}: expected 3 values, got {}",
            line_num + 1,
            tokens.len()
        )));
    }
    let x = tokens[0]
        .parse::<f32>()
        .map_err(|_| StlError::InvalidAscii(format!("line {}: invalid float '{}'", line_num + 1, tokens[0])))?;
    let y = tokens[1]
        .parse::<f32>()
        .map_err(|_| StlError::InvalidAscii(format!("line {}: invalid float '{}'", line_num + 1, tokens[1])))?;
    let z = tokens[2]
        .parse::<f32>()
        .map_err(|_| StlError::InvalidAscii(format!("line {}: invalid float '{}'", line_num + 1, tokens[2])))?;
    Ok([x, y, z])
}
