use wgpu;

#[tokio::main]
async fn main() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapters = instance.enumerate_adapters(wgpu::Backends::all());
    println!("Found {} adapters:", adapters.len());
    for adapter in adapters {
        let info = adapter.get_info();
        println!(" - {:?}: {:?} ({:?})", info.backend, info.name, info.device_type);
    }
}
