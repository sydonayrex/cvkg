//! P1-26: GPU capability detection.
//!
//! Detects GPU vendor and capabilities at startup. Used by the
//! renderer to log GPU info and (in future) select shader
//! variants appropriate for the available hardware.

use std::fmt;

/// Detected GPU vendor. This is a coarse classification derived
/// from the adapter name string. It's used to enable
/// vendor-specific workarounds in shaders or pipeline creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Qualcomm,
    Arm,
    ImgTec,
    Microsoft,
    Mesa,
    Broadcom,
    Unknown,
}

impl fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GpuVendor::Nvidia => "NVIDIA",
            GpuVendor::Amd => "AMD",
            GpuVendor::Intel => "Intel",
            GpuVendor::Apple => "Apple",
            GpuVendor::Qualcomm => "Qualcomm",
            GpuVendor::Arm => "ARM",
            GpuVendor::ImgTec => "Imagination",
            GpuVendor::Microsoft => "Microsoft",
            GpuVendor::Mesa => "Mesa",
            GpuVendor::Broadcom => "Broadcom",
            GpuVendor::Unknown => "Unknown",
        };
        f.write_str(s)
    }
}

/// Detect GPU vendor from an adapter name string. The name
/// comes from `wgpu::Adapter::get_info().name`.
///
/// This is a best-effort heuristic. Vendors may be misidentified
/// when the adapter name is unusual or contains a substring
/// from another vendor (e.g., an AMD card with "NVIDIA" in
/// the user-customized name).
pub fn detect_gpu_vendor(adapter_name: &str) -> GpuVendor {
    let name = adapter_name.to_lowercase();
    if name.contains("nvidia") || name.contains("geforce") || name.contains("quadro") || name.contains("tesla") {
        GpuVendor::Nvidia
    } else if name.contains("amd") || name.contains("radeon") || name.contains("rx ") || name.contains("firepro") {
        GpuVendor::Amd
    } else if name.contains("intel") || name.contains("uhd") || name.contains("iris") || name.contains("hd graphics") {
        GpuVendor::Intel
    } else if name.contains("apple") || name.contains("m1") || name.contains("m2") || name.contains("m3") {
        GpuVendor::Apple
    } else if name.contains("qualcomm") || name.contains("adreno") {
        GpuVendor::Qualcomm
    } else if name.contains("arm") || name.contains("mali") {
        GpuVendor::Arm
    } else if name.contains("imgtec") || name.contains("powervr") {
        GpuVendor::ImgTec
    } else if name.contains("microsoft") || name.contains("direct3d12") {
        GpuVendor::Microsoft
    } else if name.contains("mesa") || name.contains("llvmpipe") || name.contains("swiftshader") {
        GpuVendor::Mesa
    } else if name.contains("broadcom") || name.contains("videocore") {
        GpuVendor::Broadcom
    } else {
        GpuVendor::Unknown
    }
}

/// GPU capability summary. Currently just includes the vendor;
/// future versions will add feature flags (e.g., bindless
/// textures, advanced compute features).
#[derive(Debug, Clone)]
pub struct GpuCapabilities {
    /// Detected vendor.
    pub vendor: GpuVendor,
    /// The original adapter name string.
    pub adapter_name: String,
    /// The wgpu backend in use (Vulkan, Metal, DX12, etc.).
    pub backend: String,
}

impl GpuCapabilities {
    /// Detect capabilities from an adapter name and backend.
    pub fn detect(adapter_name: &str, backend: impl Into<String>) -> Self {
        Self {
            vendor: detect_gpu_vendor(adapter_name),
            adapter_name: adapter_name.to_string(),
            backend: backend.into(),
        }
    }
}

impl fmt::Display for GpuCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} GPU '{}' on {} backend",
            self.vendor, self.adapter_name, self.backend
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_nvidia() {
        assert_eq!(detect_gpu_vendor("NVIDIA GeForce RTX 3080"), GpuVendor::Nvidia);
        assert_eq!(detect_gpu_vendor("Quadro P4000"), GpuVendor::Nvidia);
    }

    #[test]
    fn detects_amd() {
        assert_eq!(detect_gpu_vendor("AMD Radeon RX 6800 XT"), GpuVendor::Amd);
        assert_eq!(detect_gpu_vendor("Radeon Pro 5500M"), GpuVendor::Amd);
    }

    #[test]
    fn detects_intel() {
        assert_eq!(detect_gpu_vendor("Intel UHD Graphics 630"), GpuVendor::Intel);
        assert_eq!(detect_gpu_vendor("Intel Iris Xe"), GpuVendor::Intel);
    }

    #[test]
    fn detects_apple() {
        assert_eq!(detect_gpu_vendor("Apple M1 Pro"), GpuVendor::Apple);
        assert_eq!(detect_gpu_vendor("Apple M2 GPU"), GpuVendor::Apple);
    }

    #[test]
    fn detects_qualcomm() {
        assert_eq!(detect_gpu_vendor("Qualcomm Adreno 660"), GpuVendor::Qualcomm);
    }

    #[test]
    fn detects_arm_mali() {
        assert_eq!(detect_gpu_vendor("ARM Mali-G78"), GpuVendor::Arm);
    }

    #[test]
    fn detects_mesa_software() {
        assert_eq!(detect_gpu_vendor("Mesa llvmpipe"), GpuVendor::Mesa);
        assert_eq!(detect_gpu_vendor("Google SwiftShader"), GpuVendor::Mesa);
    }

    #[test]
    fn unknown_for_garbage() {
        assert_eq!(detect_gpu_vendor("Custom XYZ GPU 9000"), GpuVendor::Unknown);
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(detect_gpu_vendor("nvidia geforce gtx 1080"), GpuVendor::Nvidia);
        assert_eq!(detect_gpu_vendor("AMD RADEON VII"), GpuVendor::Amd);
    }

    #[test]
    fn capabilities_display() {
        let caps = GpuCapabilities::detect("NVIDIA GeForce RTX 3080", "Vulkan");
        let s = format!("{}", caps);
        assert!(s.contains("NVIDIA"));
        assert!(s.contains("GeForce"));
        assert!(s.contains("Vulkan"));
    }
}
