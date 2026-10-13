//! Background-only configuration-read benchmark. Never initializes the GUI.
use pp_switch::{
    client::Paths,
    provider::{Client, Models},
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Instant,
};

struct Allocator;
static COUNTING: AtomicBool = AtomicBool::new(false);
static COUNT: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            COUNT.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            COUNT.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(size, Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

fn main() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::from_home(dir.path().to_owned());
    std::fs::create_dir_all(&paths.claude).unwrap();
    let mut results = Vec::new();
    for properties in [16, 128, 512] {
        let mut config = serde_json::json!({"env": {
            "ANTHROPIC_BASE_URL": "https://fixture.example/v1",
            "ANTHROPIC_AUTH_TOKEN": "example-key",
            "ANTHROPIC_MODEL": "fixture-model",
            "ANTHROPIC_DEFAULT_HAIKU_MODEL": "fixture-model",
            "ANTHROPIC_DEFAULT_SONNET_MODEL": "fixture-model",
            "ANTHROPIC_DEFAULT_OPUS_MODEL": "fixture-model"
        }, "model": "fixture-model"});
        for index in 0..properties {
            config[format!("fixture_{index}")] = serde_json::json!({"keep": [1, 2, 3]});
            config["env"][format!("KEEP_{index}")] = serde_json::json!("fixture-value");
        }
        let text = config.to_string();
        std::fs::write(paths.settings(Client::Claude), &text).unwrap();
        let expected = Some(("https://fixture.example/v1".into(), "example-key".into()));
        let models = Models::from_fields("fixture-model", "", "", "");
        assert!(paths.current(Client::Claude).unwrap() == expected);
        let mut rounds = Vec::new();
        // Alternate order to reduce systematic cache/thermal ordering bias.
        for round in 0..10 {
            for combined in if round % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                COUNT.store(0, Ordering::Relaxed);
                BYTES.store(0, Ordering::Relaxed);
                COUNTING.store(true, Ordering::Relaxed);
                let start = Instant::now();
                for _ in 0..100 {
                    let (connection, model_state) = if combined {
                        let status = paths.status(Client::Claude, true).unwrap();
                        (status.connection, status.models.unwrap())
                    } else {
                        (
                            paths.current(Client::Claude).unwrap(),
                            paths.current_models(Client::Claude).unwrap(),
                        )
                    };
                    assert!(black_box(connection) == expected);
                    assert!(black_box(model_state.matches(Client::Claude, &models)));
                }
                let elapsed = start.elapsed();
                COUNTING.store(false, Ordering::Relaxed);
                rounds.push(serde_json::json!({
                "round": round,
                "reader": if combined { "single_snapshot" } else { "separate_connection_and_models" },
                "microseconds_per_read": elapsed.as_secs_f64() * 10_000.,
                "allocation_calls_per_read": COUNT.load(Ordering::Relaxed) / 100,
                "requested_allocation_bytes_per_read": BYTES.load(Ordering::Relaxed) / 100
            }));
            }
        }
        results.push(serde_json::json!({"properties_per_object": properties,
            "file_bytes": text.len(), "rounds": rounds}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "scenario": "isolated Claude connection + model badge read; warm OS file cache; same binary and fixtures",
            "gui_initialized": false,
            "allocation_metric": "cumulative requested bytes, not live heap or process RSS",
            "results": results
        }))
        .unwrap()
    );
}
