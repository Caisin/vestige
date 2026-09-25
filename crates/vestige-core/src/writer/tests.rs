use super::*;
use serde_json::{Value, json};

fn setup() -> (Storage, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(Some(dir.path().join("test.db"))).unwrap();
    (storage, dir)
}
fn call(s: &Storage, tool: &str, args: Value) -> Value {
    s.writer_execute(tool, args).unwrap()
}
fn new_role(s: &Storage, name: &str) -> String {
    call(s, "writer_role", json!({"action":"create","name":name}))["role"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}
fn learn(s: &Storage, r: &str) -> (Value, Value) {
    let upload = s
        .writer_upload(
            r,
            "故事.txt",
            "测试作品",
            "第1场\n林夏：如果说出真相，我就会失去姐姐。\n她把录音交给警察。".as_bytes(),
        )
        .unwrap();
    let source = upload["source"]["id"].as_str().unwrap();
    let segment = call(
        s,
        "writer_source",
        json!({"action":"get","role_id":r,"source_id":source}),
    )["segments"][0]["id"]
        .clone();
    let task = call(
        s,
        "writer_task",
        json!({"action":"create","role_id":r,"kind":"extract"}),
    )["task"]
        .clone();
    let lease = call(
        s,
        "writer_task",
        json!({"action":"claim","task_id":task["id"],"agent_id":"test-agent"}),
    );
    let result = json!({"summary":"价值冲突","rules":[{"id":"choice","category":"conflict","title":"让选择有代价","instruction":"让角色在两种重要价值之间作出选择。","applies_to":"关系高潮","exceptions":"不强制用于所有场景","evidence":[{"source_id":source,"segment_id":segment,"quote":"如果说出真相，我就会失去姐姐。"}]}]});
    (lease, result)
}

#[test]
fn writer_extraction_is_evidence_checked_atomic_and_versioned() {
    let (s, _dir) = setup();
    let r = new_role(&s, "悬疑编剧");
    let (lease, result) = learn(&s, &r);
    let id = lease["task"]["id"].clone();
    let token = lease["lease_token"].clone();
    assert_eq!(
        s.writer_execute(
            "writer_task",
            json!({"action":"claim","task_id":id,"agent_id":"other"})
        )
        .unwrap_err()
        .code,
        "TASK_UNAVAILABLE"
    );
    let mut invalid = result.clone();
    invalid["rules"][0]["evidence"][0]["quote"] = json!("原文没有这句话");
    assert_eq!(
        s.writer_execute(
            "writer_task",
            json!({"action":"complete","task_id":id,"lease_token":token,"result":invalid})
        )
        .unwrap_err()
        .code,
        "INVALID_EVIDENCE"
    );
    assert_eq!(
        call(&s, "writer_role", json!({"action":"versions","role_id":r}))["versions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let complete = call(
        &s,
        "writer_task",
        json!({"action":"complete","task_id":id,"lease_token":token,"result":result}),
    );
    assert_eq!(complete["task"]["result"]["output"]["candidate_version"], 1);
    assert_eq!(
        call(&s, "writer_role", json!({"action":"get","role_id":r}))["role"]["active_version"],
        0
    );
    assert_eq!(
        call(
            &s,
            "writer_task",
            json!({"action":"complete","task_id":id,"lease_token":token,"result":result})
        )["idempotent"],
        true
    );
    call(
        &s,
        "writer_role",
        json!({"action":"publish","role_id":r,"version":1,"expected_version":0}),
    );
    assert_eq!(
        s.writer_execute(
            "writer_role",
            json!({"action":"publish","role_id":r,"version":1,"expected_version":0})
        )
        .unwrap_err()
        .code,
        "VERSION_CONFLICT"
    );
}

#[test]
fn writer_cross_role_evidence_and_project_context_are_rejected() {
    let (s, _dir) = setup();
    let r = new_role(&s, "甲");
    let other = new_role(&s, "乙");
    let (_, result) = learn(&s, &r);
    s.writer_upload(&other, "乙.txt", "乙作品", "另一部作品".as_bytes())
        .unwrap();
    let task = call(
        &s,
        "writer_task",
        json!({"action":"create","role_id":other,"kind":"extract"}),
    )["task"]["id"]
        .clone();
    let lease = call(
        &s,
        "writer_task",
        json!({"action":"claim","task_id":task,"agent_id":"agent"}),
    );
    assert_eq!(s.writer_execute("writer_task",json!({"action":"complete","task_id":task,"lease_token":lease["lease_token"],"result":result})).unwrap_err().code,"INVALID_EVIDENCE");
    let p=call(&s,"writer_project",json!({"action":"create","role_id":r,"name":"新剧","format":"short_drama","brief":"一场选择"}))["project"]["id"].clone();
    assert_eq!(
        s.writer_execute(
            "writer_context",
            json!({"action":"prepare","role_id":other,"project_id":p})
        )
        .unwrap_err()
        .code,
        "OWNERSHIP_MISMATCH"
    );
}

#[test]
fn writer_feedback_creates_candidate_and_expired_leases_can_be_reclaimed() {
    let (s, dir) = setup();
    let r = new_role(&s, "编剧");
    let task=call(&s,"writer_task",json!({"action":"create","role_id":r,"kind":"role_chat","input":{"message":"减少旁白，每集结尾保留悬念。"}}))["task"].clone();
    let lease = call(
        &s,
        "writer_task",
        json!({"action":"claim","task_id":task["id"],"agent_id":"first"}),
    );
    s.writer
        .lock()
        .unwrap()
        .execute(
            "UPDATE writer_tasks SET lease_until=0 WHERE id=?1",
            [task["id"].as_str().unwrap()],
        )
        .unwrap();
    let fresh = call(
        &s,
        "writer_task",
        json!({"action":"claim","task_id":task["id"],"agent_id":"second"}),
    );
    assert_ne!(fresh["lease_token"], lease["lease_token"]);
    let result = json!({"summary":"采用用户偏好","reply":"建议减少旁白，以行动推进。","rules":[{"id":"less-voiceover","title":"减少旁白","category":"preference","instruction":"优先动作与对话，减少解释性旁白。","evidence":[{"message_id":task["input"]["message_id"],"quote":"减少旁白"}]}]});
    assert_eq!(s.writer_execute("writer_task",json!({"action":"complete","task_id":task["id"],"lease_token":lease["lease_token"],"result":result})).unwrap_err().code,"LEASE_INVALID");
    call(
        &s,
        "writer_task",
        json!({"action":"complete","task_id":task["id"],"lease_token":fresh["lease_token"],"result":result}),
    );
    drop(s);
    let s = Storage::new(Some(dir.path().join("test.db"))).unwrap();
    assert_eq!(
        call(&s, "writer_role", json!({"action":"get","role_id":r}))["messages"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        call(&s, "writer_role", json!({"action":"get","role_id":r}))["role"]["active_version"],
        0
    );
    assert!(
        call(&s, "writer_task", json!({"action":"list"}))["tasks"][0]
            .get("lease_token")
            .is_none()
    );
}

#[test]
fn writer_drafts_reviews_and_delete_preserve_ownership_and_revisions() {
    let (s, _dir) = setup();
    let r = new_role(&s, "角色");
    let p=call(&s,"writer_project",json!({"action":"create","role_id":r,"name":"原创短剧","format":"short_drama","brief":"发生在午夜车站的选择"}))["project"]["id"].clone();
    let saved = call(
        &s,
        "writer_draft",
        json!({"action":"save","project_id":p,"role_version":0,"kind":"scene","title":"夜站","content":"她攥紧车票，没有上车。"}),
    );
    let d = saved["draft"]["id"].clone();
    let good = json!({"action":"save","draft_id":d,"draft_revision":1,"summary":"动机可以更明确","findings":[{"severity":"warning","quote":"没有上车","issue":"缺少选择的代价","suggestion":"补充留下将失去什么。"}]});
    call(&s, "writer_review", good);
    assert_eq!(s.writer_execute("writer_review",json!({"action":"save","draft_id":d,"draft_revision":1,"summary":"问题","findings":[{"severity":"error","quote":"伪造原文","issue":"问题","suggestion":"建议"}]})).unwrap_err().code,"INVALID_EVIDENCE");
    assert_eq!(s.writer_execute("writer_draft",json!({"action":"save","draft_id":d,"expected_revision":0,"project_id":p,"role_version":0,"kind":"scene","title":"夜站","content":"新正文"})).unwrap_err().code,"VERSION_CONFLICT");
    assert_eq!(
        s.writer_execute(
            "writer_role",
            json!({"action":"delete","role_id":r,"confirm":true})
        )
        .unwrap_err()
        .code,
        "ROLE_IN_USE"
    );
    call(
        &s,
        "writer_project",
        json!({"action":"delete","project_id":p,"confirm":true}),
    );
    call(
        &s,
        "writer_role",
        json!({"action":"delete","role_id":r,"confirm":true}),
    );
    for table in [
        "writer_roles",
        "writer_projects",
        "writer_drafts",
        "writer_reviews",
    ] {
        assert_eq!(
            s.reader
                .lock()
                .unwrap()
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}

#[test]
fn writer_binary_and_mcp_text_uploads_deduplicate() {
    let (s, _dir) = setup();
    let r = new_role(&s, "角色");
    let a = s
        .writer_upload(&r, "作品.txt", "作品", "完整正文".as_bytes())
        .unwrap();
    let b = call(
        &s,
        "writer_source",
        json!({"action":"attach_text","role_id":r,"title":"作品副本","content":"完整正文"}),
    );
    assert_eq!(a["source"]["id"], b["source"]["id"]);
    assert_eq!(b["duplicate"], true);
}

#[test]
fn writer_conversation_can_reply_without_inventing_rules() {
    let (s, _dir) = setup();
    let r = new_role(&s, "编剧");
    let task = call(&s,"writer_task",json!({"action":"create","role_id":r,"kind":"role_chat","input":{"message":"你好，我们从哪里开始？"}}))["task"].clone();
    let lease = call(
        &s,
        "writer_task",
        json!({"action":"claim","task_id":task["id"],"agent_id":"test"}),
    );
    let result = call(
        &s,
        "writer_task",
        json!({"action":"complete","task_id":task["id"],"lease_token":lease["lease_token"],"result":{"reply":"先上传作品，再提炼方法。当前没有新规则需要采纳。"}}),
    );
    assert!(result["task"]["result"]["output"]["candidate_version"].is_null());
    assert_eq!(
        call(&s, "writer_role", json!({"action":"versions","role_id":r}))["versions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn writer_attribution_and_late_evidence_are_preserved() {
    let (s, _dir) = setup(); let r = new_role(&s, "来源校验");
    let content = "连续原文。".repeat(1500);
    let source = s.writer_upload_with_metadata(&r, "长剧本.txt", "长剧本", content.as_bytes(), &json!({"author":"测试署名","episode":"第1集"})).unwrap()["source"].clone();
    assert_eq!(source["metadata"]["author"], "测试署名");
    assert_eq!(source["metadata"]["parser_version"], "writer-import-v1");
    let page = call(&s,"writer_source",json!({"action":"get","role_id":r,"source_id":source["id"],"offset":2,"limit":1}));
    let segment = page["segments"][0].clone();
    let direct = call(&s,"writer_source",json!({"action":"get","role_id":r,"source_id":source["id"],"segment_id":segment["id"],"limit":1}));
    assert_eq!(direct["segments"][0], segment);
    assert_eq!(s.writer_upload_with_metadata(&r,"wrong.txt","错误署名",b"text",&json!({"parser_version":"pretend"})).unwrap_err().code,"INVALID_METADATA");
}
