/*
 * Copyright (c) 2026 Mohamad Al-Zawahreh (dba Sovereign Systems).
 *
 * End-to-End Integration Tests — Phases 6-9 Cross-Module Verification.
 *
 * These tests verify that all sovereign integration modules work together
 * through the cognitive intrinsics layer, ensuring singletons are stable
 * and cross-module data flows are correct.
 */

use ark_0_zheng::cognitive_intrinsics::{all_cognitive_names, resolve_cognitive};
use ark_0_zheng::runtime::Value;

// ===========================================================================
// Test 1: All scope names resolve to actual functions
// ===========================================================================

#[test]
fn e2e_all_scopes_resolve() {
    let scopes = all_cognitive_names();
    assert!(
        scopes.len() >= 20,
        "Expected at least 20 cognitive names, got {}",
        scopes.len()
    );

    let mut missing = Vec::new();
    for scope in &scopes {
        if resolve_cognitive(scope).is_none() {
            missing.push(scope.to_string());
        }
    }

    assert!(missing.is_empty(), "Unresolved scopes: {:?}", missing);
}

// ===========================================================================
// Test 2: Yggdrasil lifecycle through intrinsics
// ===========================================================================

#[test]
fn e2e_yggdrasil_lifecycle() {
    let seed_fn = resolve_cognitive("sys.yggdrasil.seed").expect("seed should resolve");
    let result = seed_fn(vec![Value::Integer(42)]);
    assert!(result.is_ok(), "seed should succeed: {:?}", result.err());

    let cycle_fn = resolve_cognitive("sys.yggdrasil.cycle").expect("cycle should resolve");
    let result = cycle_fn(vec![]);
    assert!(result.is_ok(), "cycle should succeed");

    let harvest_fn = resolve_cognitive("sys.yggdrasil.harvest").expect("harvest should resolve");
    let result = harvest_fn(vec![]);
    assert!(result.is_ok(), "harvest should succeed");

    let metrics_fn = resolve_cognitive("sys.yggdrasil.metrics").expect("metrics should resolve");
    let result = metrics_fn(vec![]).expect("metrics should succeed");
    if let Value::Struct(fields) = result {
        assert!(
            fields.contains_key("tree_count"),
            "metrics should contain tree_count"
        );
    }
}

// ===========================================================================
// Test 3: QDMA store-query roundtrip through intrinsics
// ===========================================================================

#[test]
fn e2e_qdma_roundtrip() {
    // Store an entity as Struct with id, embedding, shards (actual intrinsic API)
    let store_fn = resolve_cognitive("sys.qdma.store").expect("store should resolve");
    let mut entity = std::collections::HashMap::new();
    entity.insert("id".to_string(), Value::String("e2e_test_key".to_string()));
    entity.insert(
        "embedding".to_string(),
        Value::List(vec![
            Value::Integer(100),
            Value::Integer(0),
            Value::Integer(0),
        ]),
    );
    entity.insert(
        "shards".to_string(),
        Value::List(vec![Value::String("test_shard".to_string())]),
    );
    let result = store_fn(vec![Value::Struct(entity)]);
    assert!(result.is_ok(), "store should succeed: {:?}", result.err());

    // Query with similar vector
    let query_fn = resolve_cognitive("sys.qdma.query").expect("query should resolve");
    let query_vec = vec![Value::Integer(90), Value::Integer(10), Value::Integer(0)];
    let result = query_fn(vec![Value::List(query_vec), Value::Integer(3)]);
    assert!(result.is_ok(), "query should succeed");

    // Stats
    let stats_fn = resolve_cognitive("sys.qdma.stats").expect("stats should resolve");
    let result = stats_fn(vec![]).expect("stats should succeed");
    if let Value::Struct(fields) = result {
        assert!(
            fields.contains_key("entity_count"),
            "stats should contain entity_count"
        );
    }
}

// ===========================================================================
// Test 4: Research pipeline through intrinsics
// ===========================================================================

#[test]
fn e2e_research_detect_and_plan() {
    let detect_fn = resolve_cognitive("sys.research.detect").expect("detect should resolve");
    let result = detect_fn(vec![Value::String("search for AI safety".to_string())])
        .expect("detect should succeed");
    if let Value::List(intents) = result {
        assert!(!intents.is_empty(), "should detect at least one intent");
        let has_search = intents.iter().any(|v| {
            if let Value::String(s) = v {
                s == "SEARCH"
            } else {
                false
            }
        });
        assert!(has_search, "should detect SEARCH intent");
    }

    let plan_fn = resolve_cognitive("sys.research.plan").expect("plan should resolve");
    let result = plan_fn(vec![Value::String(
        "search for data and draw a chart".to_string(),
    )])
    .expect("plan should succeed");
    if let Value::Struct(fields) = result {
        assert!(
            fields.contains_key("synthesis"),
            "plan should have synthesis"
        );
        assert!(fields.contains_key("steps"), "plan should have steps");
    }

    let tools_fn = resolve_cognitive("sys.research.tools").expect("tools should resolve");
    let result = tools_fn(vec![]).expect("tools should succeed");
    if let Value::List(tools) = result {
        assert!(tools.len() >= 4, "should have at least 4 tools");
    }
}

// ===========================================================================
// Test 5: Engine model registry through intrinsics
// ===========================================================================

#[test]
fn e2e_engine_models_and_select() {
    let models_fn = resolve_cognitive("sys.engine.models").expect("models should resolve");
    let result = models_fn(vec![]).expect("models should succeed");
    if let Value::List(models) = &result {
        assert!(models.len() >= 3, "should have at least 3 models");
    }

    let select_fn = resolve_cognitive("sys.engine.select").expect("select should resolve");
    let result =
        select_fn(vec![Value::String("small".to_string())]).expect("select should succeed");
    if let Value::String(name) = result {
        assert!(name.contains("Qwen"), "should select Qwen model");
    }

    let prompt_fn = resolve_cognitive("sys.engine.prompt").expect("prompt should resolve");
    let result = prompt_fn(vec![
        Value::String("Hello, world!".to_string()),
        Value::String("Previous context goes here".to_string()),
    ])
    .expect("prompt should succeed");
    if let Value::List(messages) = result {
        assert_eq!(messages.len(), 2, "should have system + user messages");
    }

    let stats_fn = resolve_cognitive("sys.engine.stats").expect("stats should resolve");
    let result = stats_fn(vec![]).expect("stats should succeed");
    if let Value::Struct(fields) = result {
        assert!(
            fields.contains_key("total_queries"),
            "stats should have total_queries"
        );
    }
}

// ===========================================================================
// Test 6: Cross-module: research intent → engine prompt
// ===========================================================================

#[test]
fn e2e_cross_module_research_to_engine() {
    let detect_fn = resolve_cognitive("sys.research.detect").unwrap();
    let intents = detect_fn(vec![Value::String(
        "search for quantum physics".to_string(),
    )])
    .expect("detect should work");

    let prompt_fn = resolve_cognitive("sys.engine.prompt").unwrap();
    let intent_str = if let Value::List(ref list) = intents {
        if let Some(Value::String(s)) = list.first() {
            s.clone()
        } else {
            "GENERAL".to_string()
        }
    } else {
        "GENERAL".to_string()
    };

    let result = prompt_fn(vec![
        Value::String(format!(
            "[INTENT: {}] search for quantum physics",
            intent_str
        )),
        Value::String("Context from QDMA memory.".to_string()),
    ])
    .expect("prompt should work");

    if let Value::List(messages) = result {
        assert_eq!(messages.len(), 2);
        if let Value::Struct(fields) = &messages[1] {
            if let Some(Value::String(content)) = fields.get("content") {
                assert!(content.contains("INTENT"), "should contain intent marker");
                assert!(
                    content.contains("RELEVANT LONG-TERM MEMORY"),
                    "should inject context"
                );
            }
        }
    }
}

// ===========================================================================
// Test 7: Error handling — type mismatches
// ===========================================================================

#[test]
fn e2e_type_mismatch_errors() {
    let store_fn = resolve_cognitive("sys.qdma.store").unwrap();
    let result = store_fn(vec![Value::Integer(42)]); // should want String
    assert!(result.is_err(), "should fail on type mismatch");

    let detect_fn = resolve_cognitive("sys.research.detect").unwrap();
    let result = detect_fn(vec![Value::Integer(42)]); // should want String
    assert!(result.is_err(), "should fail on type mismatch");

    let select_fn = resolve_cognitive("sys.engine.select").unwrap();
    let result = select_fn(vec![Value::Integer(42)]); // should want String
    assert!(result.is_err(), "should fail on type mismatch");
}
