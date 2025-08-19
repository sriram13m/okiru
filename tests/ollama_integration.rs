use okiru::{AppInfo, OllamaClient};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct TestCase {
    name: String,
    description: String,
    intention: String,
    app: TestAppInfo,
    expected_score_range: [f32; 2],
}

#[derive(Debug, Deserialize, Clone)]
struct TestAppInfo {
    app_name: String,
    window_title: String,
    bundle_id: String,
    process_id: i32,
}

#[derive(Debug, Deserialize)]
struct TestSuite {
    test_cases: Vec<TestCase>,
}

impl From<TestAppInfo> for AppInfo {
    fn from(test_app: TestAppInfo) -> Self {
        AppInfo {
            app_name: test_app.app_name,
            window_title: test_app.window_title,
            bundle_id: test_app.bundle_id,
            process_id: test_app.process_id,
        }
    }
}

async fn load_test_cases() -> Vec<TestCase> {
    let test_data = fs::read_to_string("test_cases.json")
        .expect("Failed to read test_cases.json - make sure it exists in project root");
    
    let test_suite: TestSuite = serde_json::from_str(&test_data)
        .expect("Failed to parse test_cases.json");
    
    test_suite.test_cases
}

#[tokio::test]
#[ignore] 
async fn test_ollama_alignment_evaluation() {
    let client = OllamaClient::new(
        "http://localhost:11434".to_string(),
        "gemma3:4b".to_string(),
    );

    let test_cases = load_test_cases().await;
    println!("🧪 Running {} Ollama alignment tests...\n", test_cases.len());

    let mut passed = 0;
    let mut failed = 0;

    for case in test_cases {
        println!("Testing: {}", case.name);
        println!("  Description: {}", case.description);
        println!("  Intention: \"{}\"", case.intention);
        println!("  Activity: {} - {}", case.app.app_name, case.app.window_title);

        match client.evaluate_alignment(&case.intention, &case.app.clone().into(), "").await {
            Ok(result) => {
                let in_range = result.alignment_score >= case.expected_score_range[0] 
                    && result.alignment_score <= case.expected_score_range[1];

                if in_range {
                    println!("  ✅ PASS: Score {:.2} (expected {:.2}-{:.2})", 
                        result.alignment_score, case.expected_score_range[0], case.expected_score_range[1]);
                    println!("  💭 Reasoning: {}", result.reasoning);
                    if let Some(suggestion) = result.suggestion {
                        println!("  💡 Suggestion: {}", suggestion);
                    }
                    passed += 1;
                } else {
                    println!("  ❌ FAIL: Score {:.2} outside expected range [{:.2}, {:.2}]", 
                        result.alignment_score, case.expected_score_range[0], case.expected_score_range[1]);
                    failed += 1;
                }
            },
            Err(e) => {
                println!("  💥 ERROR: {}", e);
                failed += 1;
            }
        }
        println!(); 
    }

    println!("📊 Test Results: {} passed, {} failed", passed, failed);
    
    if failed > 0 {
        panic!("Some alignment tests failed! Check Ollama connection and model availability.");
    }
}

#[tokio::test]
async fn test_ollama_connection() {
    let client = OllamaClient::new(
        "http://localhost:11434".to_string(),
        "gemma3:4b".to_string(),
    );

    let test_app = AppInfo {
        app_name: "Test App".to_string(),
        window_title: "Test Window".to_string(),
        bundle_id: "com.test.app".to_string(),
        process_id: 12345,
    };

    match client.evaluate_alignment("Test intention", &test_app, "").await {
        Ok(_) => println!("✅ Ollama connection successful"),
        Err(e) => {
            println!("❌ Ollama connection failed: {}", e);
            println!("Make sure Ollama is running with: ollama run gemma3:4b");
            panic!("Ollama connection test failed");
        }
    }
}
