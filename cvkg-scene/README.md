# cvkg-scene

**cvkg-scene** manages 3D coordinate systems and scene graphs for hybrid 2D/3D CVKG applications.

## 🚀 Quick Start

```rust
use cvkg_scene::{Scene, SceneNode, Transform};
use cvkg_core::View;

// Create a 3D scene
let mut scene = Scene::new();

// Add a node with 3D transform
let node = SceneNode::new(