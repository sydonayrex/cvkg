use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::{WasiP1Ctx, add_to_linker_sync};

/// A session representing a loaded and running WASM module.
/// Holds the store and instance together to ensure they stay in sync.
pub struct WasmSession {
    pub store: Store<HostState>,
    pub instance: Instance,
}

/// Host state for WASI and CVKG logic.
pub struct HostState {
    pub wasi: WasiP1Ctx,
}

/// A server-side WASM host for executing CVKG logic natively.
/// Supporting Wasmtime v44.0.0 with hardened lifecycle and WASI wiring.
#[derive(Clone)]
pub struct NativeWasmServer {
    engine: Engine,
    session: Arc<Mutex<Option<WasmSession>>>,
}

impl NativeWasmServer {
    /// Create a new WASM server with a shared engine.
    pub fn new() -> anyhow::Result<Self> {
        let mut config = Config::new();
        config.consume_fuel(false);
        config.async_support(false);

        let engine = Engine::new(&config)?;
        Ok(Self {
            engine,
            session: Arc::new(Mutex::new(None)),
        })
    }

    /// Initialize or reload a WASM module into a persistent session.
    ///
    /// If `force_reload` is false and a session already exists, this call is a no-op,
    /// preserving the existing WASM state (memory, globals).
    /// If `force_reload` is true, the module is re-instantiated, resetting all state.
    pub fn load_module(&self, wasm_path: &Path, force_reload: bool) -> anyhow::Result<()> {
        if !force_reload {
            let guard = self.session.lock().unwrap();
            if guard.is_some() {
                debug!("[Native Wasm] Module already loaded, skipping reload to preserve state.");
                return Ok(());
            }
        }

        info!("[Native Wasm] Loading module: {:?}", wasm_path);

        let module = Module::from_file(&self.engine, wasm_path)?;
        let mut linker = Linker::new(&self.engine);

        // Link WASI Preview 1
        add_to_linker_sync(&mut linker, |s: &mut HostState| &mut s.wasi)?;

        // Build hardened WASI context for Preview 1
        let mut wasi_builder = WasiCtxBuilder::new();
        wasi_builder
            .inherit_stdout()
            .inherit_stderr()
            .inherit_stdin();

        // Hardened: Preopen only the current working directory as a safe root.
        // This prevents the WASM guest from accessing the entire host filesystem.
        let safe_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        info!(
            "[Native Wasm] Hardening WASI: Preopening safe root: {:?}",
            safe_root
        );

        wasi_builder
            .preopened_dir(
                &safe_root,
                ".",
                wasmtime_wasi::DirPerms::all(),
                wasmtime_wasi::FilePerms::all(),
            )
            .map_err(|e| anyhow::anyhow!("Failed to preopen directory: {:?}", e))?;

        let wasi = wasi_builder.build_p1();

        let mut store = Store::new(&self.engine, HostState { wasi });

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| anyhow::anyhow!("WASM instantiation failed: {:?}", e))?;

        // Run init if present
        if let Some(func) = instance.get_func(&mut store, "cvkg_init") {
            info!("[Native Wasm] Calling cvkg_init()...");
            let typed = func.typed::<(), ()>(&store)?;
            typed
                .call(&mut store, ())
                .map_err(|e| anyhow::anyhow!("cvkg_init failed: {:?}", e))?;
        }

        let mut session_guard = self.session.lock().unwrap();
        *session_guard = Some(WasmSession { store, instance });

        Ok(())
    }

    /// Execute a single 'tick' (update + render) of the loaded module.
    pub fn tick(&self) -> anyhow::Result<()> {
        let mut session = {
            let mut guard = self.session.lock().unwrap();
            guard.take()
        }
        .ok_or_else(|| anyhow::anyhow!("No active WASM session"))?;

        let result = self.execute_tick(&mut session);

        let mut guard = self.session.lock().unwrap();
        *guard = Some(session);

        result
    }

    fn execute_tick(&self, session: &mut WasmSession) -> anyhow::Result<()> {
        if let Some(func) = session.instance.get_func(&mut session.store, "cvkg_update") {
            let typed = func.typed::<(), ()>(&session.store)?;
            typed
                .call(&mut session.store, ())
                .map_err(|e| anyhow::anyhow!("cvkg_update failed: {:?}", e))?;
        }

        if let Some(func) = session.instance.get_func(&mut session.store, "cvkg_render") {
            let typed = func.typed::<(), ()>(&session.store)?;
            typed
                .call(&mut session.store, ())
                .map_err(|e| anyhow::anyhow!("cvkg_render failed: {:?}", e))?;
        }

        Ok(())
    }
}
