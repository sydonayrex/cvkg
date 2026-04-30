use wasmtime_wasi::{WasiView, ResourceTable};
use wasmtime_wasi::p1::WasiP1Ctx;

struct Dummy {
    wasi: WasiP1Ctx,
    table: ResourceTable,
}

impl WasiView for Dummy {
    fn ctx(&mut self) -> &mut WasiP1Ctx { &mut self.wasi }
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}

fn main() {}
