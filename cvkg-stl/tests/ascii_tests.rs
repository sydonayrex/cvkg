//! ASCII STL parser tests — P1

#[cfg(test)]
mod tests {
    /// Generate a minimal ASCII STL with 2 triangles forming a quad.
    fn generate_simple_quad_ascii() -> Vec<u8> {
        b"solid simple_quad
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 1 1 0
    endloop
  endfacet
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 1 0
      vertex 0 1 0
    endloop
  endfacet
endsolid simple_quad
".to_vec()
    }

    #[test]
    fn test_ascii_simple_quad() {
        let data = generate_simple_quad_ascii();
        let mesh = cvkg_stl::parse_bytes(&data).expect("ASCII parse should succeed");
        assert_eq!(mesh.indices.len(), 6, "2 triangles × 3 indices");
        assert!(mesh.vertices.len() >= 4, "at least 4 unique corners");
    }

    #[test]
    fn test_ascii_normals_preserved() {
        let data = generate_simple_quad_ascii();
        let mesh = cvkg_stl::parse_bytes(&data).expect("ASCII parse should succeed");
        // All normals should be [0, 0, 1]
        for n in &mesh.normals {
            assert!(
                (n[0].abs() < 0.001 && n[1].abs() < 0.001 && (n[2] - 1.0).abs() < 0.001),
                "expected normal ~[0,0,1] got {:?}",
                n
            );
        }
    }

    #[test]
    fn test_ascii_extra_whitespace() {
        let data = b"solid spacey
  facet   normal   0   0   1
    outer  loop
      vertex   0   0   0
      vertex   1   0   0
      vertex   1   1   0
    endloop
  endfacet
endsolid spacey
".to_vec();
        let mesh = cvkg_stl::parse_bytes(&data).expect("extra whitespace should parse");
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn test_ascii_missing_endsolid_errors() {
        let data = b"solid incomplete
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 1 1 0
    endloop
  endfacet
".to_vec();
        let result = cvkg_stl::parse_bytes(&data);
        assert!(result.is_err(), "missing endsolid should error");
    }

    #[test]
    fn test_ascii_case_insensitive() {
        let data = b"SOLID Cased
  FACET NORMAL 0 0 1
    OUTER LOOP
      VERTEX 0 0 0
      VERTEX 1 0 0
      VERTEX 1 1 0
    ENDLOOP
  ENDFACET
ENDSOLID Cased
".to_vec();
        let mesh = cvkg_stl::parse_bytes(&data).expect("case insensitive parse should work");
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn test_ascii_auto_detected() {
        let data = generate_simple_quad_ascii();
        // Should auto-detect as ASCII without format hint
        let mesh = cvkg_stl::parse_bytes(&data).expect("auto-detect ASCII");
        assert!(!mesh.vertices.is_empty());
    }
}
