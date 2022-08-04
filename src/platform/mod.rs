use std::{
    path::Path,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use deno_core::{error::AnyError, FsModuleLoader};
use deno_runtime::{
    deno_broadcast_channel::InMemoryBroadcastChannel,
    deno_web::BlobStore,
    permissions::Permissions,
    worker::{MainWorker, WorkerOptions},
    BootstrapOptions,
};
use tokio::runtime::{self, Runtime};

pub fn create_runtime() -> runtime::Runtime {
    runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(12)
        .build()
        .unwrap()
}

fn get_error_class_name(e: &AnyError) -> &'static str {
    deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
}

pub fn create_worker_options() -> WorkerOptions {
    let module_loader = Rc::new(FsModuleLoader);
    let create_web_worker_cb = Arc::new(|_| {
        todo!("Web workers are not supported in the example");
    });
    let web_worker_preload_module_cb = Arc::new(|_| {
        todo!("Web workers are not supported in the example");
    });

    WorkerOptions {
        bootstrap: BootstrapOptions {
            args: vec![],
            cpu_count: 1,
            debug_flag: false,
            enable_testing_features: false,
            location: None,
            no_color: false,
            is_tty: false,
            runtime_version: "x".to_string(),
            ts_version: "x".to_string(),
            unstable: false,
            user_agent: "hello_runtime".to_string(),
        },
        extensions: vec![],
        unsafely_ignore_certificate_errors: None,
        root_cert_store: None,
        seed: None,
        source_map_getter: None,
        format_js_error_fn: None,
        web_worker_preload_module_cb,
        create_web_worker_cb,
        maybe_inspector_server: None,
        should_break_on_first_statement: false,
        module_loader,
        get_error_class_fn: Some(&get_error_class_name),
        origin_storage_dir: None,
        blob_store: BlobStore::default(),
        broadcast_channel: InMemoryBroadcastChannel::default(),
        shared_array_buffer_store: None,
        compiled_wasm_module_store: None,
        stdio: Default::default(),
    }
}

pub struct Worker {
    options: WorkerOptions,
    runtime: Runtime,
    timeout: Duration,
}

impl Worker {
    pub fn new(options: WorkerOptions, runtime: Runtime, timeout: Duration) -> Self {
        Self {
            options,
            runtime,
            timeout,
        }
    }

    pub fn execute(self, path: String) {
        let now = Instant::now();
        self.runtime.block_on(async move {
            let js_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples")
                .join(path.clone());
            let main_module = deno_core::resolve_path(&js_path.to_string_lossy()).unwrap();
            let permissions = Permissions::allow_all();
            let mut worker =
                MainWorker::bootstrap_from_options(main_module.clone(), permissions, self.options);
            worker.execute_main_module(&main_module).await.unwrap();
            println!("Start event loop");
            if tokio::time::timeout(self.timeout, worker.run_event_loop(false))
                .await
                .is_err()
            {
                println!("Runtime: Timeout event loop {}", path);
            } else {
                println!(
                    "Runtime: Executing {} took {}ms",
                    path,
                    now.elapsed().as_millis()
                );
            };
        });
    }
}

impl Default for Worker {
    fn default() -> Self {
        Self::new(
            create_worker_options(),
            create_runtime(),
            Duration::from_millis(5000),
        )
    }
}
