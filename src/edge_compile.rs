use std::sync::Arc;

use std::path::Path;
use std::collections::HashMap;
use rspack_ids::{NaturalChunkIdsPlugin, NamedModuleIdsPlugin};
use rspack_core::{
    ResolverFactory,
    CacheOptions, ChunkLoading, ChunkLoadingType, Compiler, CompilerOptions, Context,
    CrossOriginLoading, DevServerOptions, EntryOptions, Environment, Experiments, Filename,
    HashDigest, HashFunction, HashSalt, MangleExportsOption, Mode,
    ModuleOptions, Optimization, OutputOptions, PathInfo,
    Plugin, PublicPath, Resolve, SideEffectOption, SnapshotOptions,
    StatsOptions, Target, UsedExportsOption, WasmLoading,
    DynamicImportMode, DynamicImportFetchPriority, JavascriptParserOrder, JavascriptParserUrl,
    ParserOptionsMap, ModuleType, ParserOptions, JavascriptParserOptions,RspackFuture,Incremental
};
use rspack_plugin_entry::EntryPlugin;
use rspack_plugin_javascript::JsPlugin;
use rspack_plugin_schemes::{
    DataUriPlugin, HttpUriPlugin, HttpUriPluginOptions, HttpUriOptionsAllowedUris
};
use serde_json::{Map, Value};
use std::fs;
use crate::memory_fs::MockFileSystem;
use crate::system_fs::RealFileSystem;
use rspack_fs::AsyncFileSystem;
use crate::http_io::ReqwestHttpClient;
use rspack_paths::{Utf8PathBuf};
use rspack_fs::ReadableFileSystem;

pub async fn compile(network_entry: Option<String>) -> HashMap<String, Vec<u8>> {
    let mock_fs = MockFileSystem::new();
    let output_filesystem = mock_fs.clone();
    let root = env!("CARGO_MANIFEST_DIR");
    let context = Context::new(root.to_string().into());
    let dist_dir: Utf8PathBuf = Utf8PathBuf::from_path_buf(Path::new(root).join("./dist")).unwrap();
    if !dist_dir.exists() {
        fs::create_dir_all(&dist_dir).expect("Failed to create dist directory");
    }
    let dist_dir = Utf8PathBuf::from_path_buf(dist_dir.canonicalize().unwrap()).unwrap();
    let entry_file: String = network_entry
        .as_deref()
        .filter(|entry| !entry.is_empty())
        .map_or_else(
            || {
                Utf8PathBuf::from_path_buf(Path::new(root)
                    .join("./fixtures/index.js")
                    .canonicalize()
                    .unwrap()).unwrap()
                    .to_string()
            },
            |entry| entry.to_string(),
        );
    dbg!(network_entry.clone());

    let compiler_options = CompilerOptions {
        context: root.into(),
        dev_server: DevServerOptions::default(),
        output: OutputOptions {
            path: dist_dir,
            pathinfo: PathInfo::Bool(false),
            clean: false,
            public_path: PublicPath::Auto,
            asset_module_filename: Filename::from(String::from("asset-[name].js")),
            wasm_loading: WasmLoading::Disable,
            webassembly_module_filename: Filename::from(String::from("webassembly.js")),
            unique_name: "main".into(),
            chunk_loading: ChunkLoading::Enable(ChunkLoadingType::Jsonp),
            chunk_loading_global: String::new(),
            filename: Filename::from(String::from("[name].js")),
            chunk_filename: Filename::from(String::from("[id].js")),
            cross_origin_loading: CrossOriginLoading::Disable,
            css_filename: Filename::from(String::from("[name].css")),
            css_chunk_filename: Filename::from(String::from("[id].css")),
            hot_update_main_filename: Filename::from(String::from("[name].[hash].hot-update.js")),
            hot_update_chunk_filename: Filename::from(String::from("[id].[hash].hot-update.js")),
            hot_update_global: String::new(),
            library: None,
            enabled_library_types: None,
            strict_module_error_handling: false,
            global_object: String::from("window"),
            import_function_name: String::from("import"),
            import_meta_name: String::from("import.meta"),
            iife: false,
            module: false,
            trusted_types: None,
            source_map_filename: Filename::from(String::from("[file].map")),
            hash_function: HashFunction::MD4,
            hash_digest: HashDigest::Hex,
            hash_digest_length: 20,
            hash_salt: HashSalt::Salt(String::from("salt")),
            async_chunks: false,
            worker_chunk_loading: ChunkLoading::Disable,
            worker_wasm_loading: WasmLoading::Disable,
            worker_public_path: String::new(),
            script_type: String::from("javascript/module"),
            environment: Environment {
                r#const: Some(true),
                arrow_function: Some(true),
            },
            charset: false,
            chunk_load_timeout: 120000,
            css_head_data_compression: false,
        },
        target: Target::new(&vec!["es2022".to_string()]).unwrap(),
        mode: Mode::Development,
        resolve: Resolve {
            extensions: Some(vec![".js".to_string()]),
            ..Default::default()
        },
        resolve_loader: Resolve {
            extensions: Some(vec![".js".to_string()]),
            ..Default::default()
        },
        module: ModuleOptions {
            parser: Some(ParserOptionsMap::from_iter([(
                ModuleType::JsAuto.to_string(),
                ParserOptions::Javascript(JavascriptParserOptions {
                    dynamic_import_mode: Some(DynamicImportMode::Eager),
                    dynamic_import_prefetch: Some(JavascriptParserOrder::Order(1)),
                    import_meta: Some(false),
                    dynamic_import_fetch_priority: Some(DynamicImportFetchPriority::Auto),
                    url: Some(JavascriptParserUrl::Disable),
                    expr_context_critical: Some(false),
                    wrapped_context_critical: Some(false),
                    exports_presence: None,
                    import_exports_presence: None,
                    reexport_exports_presence: None,
                    strict_export_presence: Some(false),
                    worker: Some(vec![]),
                    dynamic_import_preload: Some(JavascriptParserOrder::Order(0)),
                    override_strict: None,
                }),
            )])),
            ..Default::default()
        },
        stats: StatsOptions::default(),
        snapshot: SnapshotOptions,
        cache: CacheOptions::default(),
        experiments: Experiments {
            layers: false,
            incremental: Incremental::Disabled,
            top_level_await: false,
            rspack_future: RspackFuture {},
        },
        optimization: Optimization {
            concatenate_modules: false,
            remove_available_modules: false,
            provided_exports: false,
            mangle_exports: MangleExportsOption::False,
            inner_graph: true,
            used_exports: UsedExportsOption::default(),
            side_effects: SideEffectOption::default(),
        },
        profile: false,
        bail: false,
        __references: Map::<String, Value>::new(),
        node: None,
    };
    let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();

    let entry_plugin_options = EntryOptions {
        name: Some("main".to_string()),
        runtime: None,
        layer: None,
        chunk_loading: None,
        async_chunks: None,
        public_path: None,
        base_uri: None,
        filename: None,
        library: None,
        depend_on: None,
    };
    let entry_plugin = Box::new(EntryPlugin::new(context, entry_file.clone(), entry_plugin_options));
    plugins.push(Box::new(JsPlugin::default()));
    plugins.push(entry_plugin);
    plugins.push(Box::new(NaturalChunkIdsPlugin::default()));
    plugins.push(Box::new(NamedModuleIdsPlugin::default()));
    plugins.push(Box::new(DataUriPlugin::default()));

    let real_fs = Arc::new(RealFileSystem::new());
    let native_fs_async: Arc<dyn AsyncFileSystem + Send + Sync> = real_fs.clone();
    let native_fs_read: Arc<dyn ReadableFileSystem + Send + Sync> = real_fs.clone();

    let cache_location = Some({
        let cwd = std::env::current_dir().unwrap();
        let mut dir = cwd.clone();
        loop {
            if let Ok(metadata) = std::fs::metadata(dir.join("package.json")) {
                if metadata.is_file() {
                    break;
                }
            }
            let parent = dir.parent();
            if parent.is_none() {
                dir = cwd.join(".cache/webpack");
                break;
            }
            dir = parent.unwrap().to_path_buf();
        }
        if std::env::var("pnp").unwrap_or_default() == "1" {
            dir.join(".pnp/.cache/webpack")
        } else if std::env::var("pnp").unwrap_or_default() == "3" {
            dir.join(".yarn/.cache/webpack")
        } else {
            dir.join("node_modules/.cache/webpack")
        }
        .to_string_lossy()
        .to_string()
    });

    let lockfile_location = cache_location.clone().map(|loc| format!("{}/lockfile.json", loc));

    let http_client = Arc::new(ReqwestHttpClient::new());

    let http_uri_options = HttpUriPluginOptions {
        allowed_uris: HttpUriOptionsAllowedUris,
        cache_location: cache_location.clone(),
        frozen: Some(true),
        lockfile_location, 
        proxy: Some("http://proxy.example.com".to_string()),
        upgrade: Some(true),
        filesystem: native_fs_async.clone(),
        http_client: Some(http_client)
    };
    plugins.push(Box::new(HttpUriPlugin::new(http_uri_options)));

    let resolver_factory = Arc::new(ResolverFactory::new(
        compiler_options.resolve.clone(),
        Arc::new(RealFileSystem::new()),
    ));
    let loader_resolver_factory = Arc::new(ResolverFactory::new(
        compiler_options.resolve_loader.clone(),
        Arc::new(RealFileSystem::new()),
    ));
    let mut compiler = Compiler::new(
        compiler_options,
        plugins,
        Some(Box::new(output_filesystem.clone())),
        Some(native_fs_read.clone()),
        Some(resolver_factory),
        Some(loader_resolver_factory),
    );
    println!("Compiling with entry: {}", entry_file);
    compiler.build().await.expect("build failed");

    let compiled_files = output_filesystem.files.read().await;

    compiled_files.iter()
        .map(|(path, content)| (path.to_string_lossy().to_string(), content.clone()))
        .collect()
}