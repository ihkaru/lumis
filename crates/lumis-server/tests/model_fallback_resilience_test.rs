use lumis_server::routes::chat::model_fallback::{
    get_model_display_name, get_ttft_duration, is_model_circuit_open, record_model_failure,
    record_model_success, resolve_model_cascade,
};
use std::time::Duration;

#[tokio::test]
async fn test_model_cascade_resolution_and_deduplication() {
    let cascade_free = resolve_model_cascade("my9model-free");
    assert_eq!(cascade_free[0], "my9model-free");
    assert!(cascade_free.contains(&"openrouter-free".to_string()));

    // Verify deduplication
    let mut unique = cascade_free.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), cascade_free.len());

    // Verify variant suffix stripping fallback (e.g. -smart falls back to base model)
    let cascade_smart = resolve_model_cascade("my9model-smart");
    assert_eq!(cascade_smart[0], "my9model-smart");
    assert!(cascade_smart.contains(&"my9model".to_string()) || cascade_smart.contains(&"my9model-free".to_string()));
    assert!(cascade_smart.contains(&"openrouter-free".to_string()));

    // Verify unknown model fallback
    let cascade_unknown = resolve_model_cascade("unknown-model-xyz");
    assert_eq!(cascade_unknown[0], "unknown-model-xyz");
    assert!(cascade_unknown.contains(&"my9model-free".to_string()));

    // Verify Muse reasoning model cascade (strips -contributor to fallback to muse/muse-spark-1.3)
    let cascade_muse = resolve_model_cascade("muse/muse-spark-1.3-contributor");
    assert_eq!(cascade_muse[0], "muse/muse-spark-1.3-contributor");
    assert!(cascade_muse.contains(&"muse/muse-spark-1.3".to_string()));
    assert!(cascade_muse.contains(&"openrouter-free".to_string()));
    assert!(cascade_muse.contains(&"my9model-free".to_string()));
}

#[tokio::test]
async fn test_model_circuit_breaker_trip_and_recovery() {
    let test_model = "test-flaky-model-circuit";

    // Initially circuit should be closed
    assert!(!is_model_circuit_open(test_model));

    // First failure: count = 1 (< threshold of 2), still closed
    record_model_failure(test_model);
    assert!(!is_model_circuit_open(test_model));

    // Second failure: count = 2 (>= threshold), circuit TRIPS to open!
    record_model_failure(test_model);
    assert!(is_model_circuit_open(test_model));

    // Successful completion resets the circuit back to closed immediately
    record_model_success(test_model);
    assert!(!is_model_circuit_open(test_model));
}

#[test]
fn test_model_display_names_and_ttft_durations() {
    assert_eq!(
        get_model_display_name("my9model-free"),
        "Nemotron 3 Ultra (Free Gateway)"
    );
    assert_eq!(
        get_model_display_name("muse/muse-spark-1.3-contributor"),
        "Muse Spark 1.3 Contributor (Deep Reasoning)"
    );
    // Arbitrary unknown model is dynamically formatted in Title Case without hardcoding
    assert_eq!(get_model_display_name("unknown-model"), "Unknown Model");
    assert_eq!(get_model_display_name("meta/llama-3.3-70b"), "Meta Llama 3.3 70b");

    // Deep reasoning models (detected dynamically via keywords) get 15s TTFT budget
    assert_eq!(get_ttft_duration("muse/muse-spark-1.3-contributor"), Duration::from_millis(15_000));
    assert_eq!(get_ttft_duration("deepseek-r1"), Duration::from_millis(15_000));
    assert_eq!(get_ttft_duration("openai/o3-mini"), Duration::from_millis(15_000));

    // Standard fast models get default 10s TTFT budget
    assert_eq!(get_ttft_duration("my9model-free"), Duration::from_millis(10_000));
    assert_eq!(get_ttft_duration("llama-3-8b"), Duration::from_millis(10_000));
}
