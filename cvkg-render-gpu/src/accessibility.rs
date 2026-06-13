use crate::color_blindness::ColorBlindMode;
use crate::kvasir::node::KvasirNode;
use crate::kvasir::resource::ResourceId;

#[derive(Debug, Clone)]
pub enum AccessibilityTransform {
    ColorBlind(ColorBlindMode),
    HighContrast(f32),
    MotionReduction,
    Magnification { region: [f32; 4], scale: f32 },
    FocusEnhancement { target: ResourceId },
}

pub struct AccessibilityService {
    pub active_transforms: Vec<AccessibilityTransform>,
}

impl Default for AccessibilityService {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessibilityService {
    pub fn new() -> Self {
        Self {
            active_transforms: Vec::new(),
        }
    }

    pub fn graph_nodes(&self) -> Vec<Box<dyn KvasirNode>> {
        let mut nodes: Vec<Box<dyn KvasirNode>> = Vec::new();
        for transform in &self.active_transforms {
            if let AccessibilityTransform::ColorBlind(_) = transform {
                nodes.push(Box::new(
                    crate::passes::accessibility::AccessibilityNode::new(),
                ));
            }
        }
        nodes
    }
}
