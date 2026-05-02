use wgpu;
use pollster;

fn main() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: wgpu::InstanceFlags::default(),
        backend_options: wgpu::BackendOptions::default(),
        display: None,
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
    });
    
    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    println!("Found {} adapters:", adapters.len());
    for adapter in adapters {
        let info = adapter.get_info();
        println!(" - {:?}: {:?} ({:?})", info.backend, info.name, info.device_type);
    }
}
