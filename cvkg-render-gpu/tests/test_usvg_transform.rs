#[test]
fn test_usvg() {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(b"<svg><g transform=\"scale(2)\"><path d=\"M0,0 L1,1\"/></g></svg>", &opt).unwrap();
    for node in tree.root().children() {
        if let usvg::Node::Group(g) = node {
            let abs_transform = g.abs_transform();
            println!("{:?}", abs_transform);
        }
    }
}
