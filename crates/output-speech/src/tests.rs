use super::*;

#[test]
fn test_synthesize_fails_gracefully_when_piper_missing() {
    // Verify we get a clean error rather than a panic when the command isn't found.
    // In CI / containers without piper this exercises the error path.
    let result = std::process::Command::new("piper").arg("--version").output();

    match result {
        Err(_) => {
            // piper not installed — confirm our synthesize() returns Err
            let err = synthesize("hello", "/nonexistent/model.onnx", None).unwrap_err();
            assert!(
                err.to_string().contains("piper"),
                "error should mention piper: {err}"
            );
        }
        Ok(_) => {
            // piper is installed but we have no model — should return Err cleanly
            let err = synthesize("hello", "/nonexistent/model.onnx", None).unwrap_err();
            assert!(!err.to_string().is_empty(), "error should be non-empty: {err}");
        }
    }
}
