//! Explicit allowlists keep local-filesystem and global administrative tools out of tenant requests.
use super::{Error, Grant, Result};
use serde_json::Value;
pub fn writer(g: &Grant, tool: &str, a: &Value) -> Result<()> {
    let action = a["action"].as_str().unwrap_or("");
    let read = match tool {
        "writer_role" => matches!(action, "list" | "get" | "versions"),
        "writer_source" => matches!(action, "list" | "get" | "segments" | "search"),
        "writer_task" => matches!(action, "list" | "get"),
        "writer_context" => action == "prepare",
        "writer_project" => matches!(action, "list" | "get"),
        "writer_draft" => matches!(action, "list" | "get" | "history"),
        "writer_review" => matches!(action, "list" | "get"),
        _ => false,
    };
    if read {
        return Ok(());
    }
    if g.role == "viewer" {
        return Err(Error::forbidden());
    }
    if tool == "writer_role"
        && matches!(action, "publish" | "rollback" | "delete")
        && g.role != "owner"
    {
        return Err(Error::forbidden());
    }
    Ok(())
}
pub fn tool(g: &Grant, name: &str, a: &Value) -> Result<()> {
    if crate::tools::writer::TOOLS
        .iter()
        .any(|(n, _, _, _)| *n == name)
    {
        return writer(g, name, a);
    }
    let read = match name {
        "recall" | "memory_status" | "graph" => true,
        "memory" => matches!(a["action"].as_str(), Some("get" | "get_batch" | "state")),
        _ => false,
    };
    if read {
        return Ok(());
    }
    if g.role == "viewer" {
        return Err(Error::forbidden());
    }
    match name {
        "smart_ingest" | "memory" | "suppress" | "backfill" | "deep_reference"
        | "cross_reference" | "dedup" => Ok(()),
        "maintain"
            if matches!(
                a["action"].as_str(),
                Some("consolidate" | "dream" | "gc" | "importance_score")
            ) =>
        {
            Ok(())
        }
        _ => Err(Error::forbidden()),
    }
}
pub fn http(g: &Grant, method: &str, path: &str) -> Result<()> {
    // Explicitly exclude global verifier telemetry, local model operations and any future route.
    let safe = matches!(
        path,
        "/api/memories"
            | "/api/search"
            | "/api/stats"
            | "/api/health"
            | "/api/timeline"
            | "/api/changelog"
            | "/api/graph"
            | "/api/dream"
            | "/api/consolidate"
            | "/api/explore"
            | "/api/predict"
            | "/api/importance"
            | "/api/retention-distribution"
            | "/api/embeddings/profiles"
            | "/api/duplicates"
            | "/api/duplicates/plan"
            | "/api/duplicates/apply"
            | "/api/intentions"
            | "/api/contradictions"
            | "/api/patterns/cross-project"
            | "/api/deep_reference"
            | "/api/backfill"
            | "/api/receipts"
            | "/api/traces"
            | "/api/memory-prs"
    ) || [
        "/api/memories/",
        "/api/receipts/",
        "/api/traces/",
        "/api/memory-prs/",
    ]
    .iter()
    .any(|p| path.starts_with(p));
    if !safe {
        return Err(Error::forbidden());
    }
    if method != "GET" && g.role == "viewer" {
        return Err(Error::forbidden());
    }
    Ok(())
}
