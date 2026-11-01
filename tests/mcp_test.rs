use std::io::Cursor;
use std::sync::Arc;
use serde_json::{json, Value};
use tokio::io::BufReader;

use rcal::mcp::run_mcp_server;
use rcal::storage::Database;

async fn send_mcp_messages(db: Arc<Database>, input_lines: &[Value]) -> Vec<Value> {
    let mut input_bytes = Vec::new();
    for msg in input_lines {
        let s = serde_json::to_string(msg).unwrap();
        input_bytes.extend_from_slice(s.as_bytes());
        input_bytes.push(b'\n');
    }

    let reader = BufReader::new(Cursor::new(input_bytes));
    let mut output_bytes = Vec::new();

    run_mcp_server(reader, &mut output_bytes, db)
        .await
        .expect("Server run error");

    let output_str = String::from_utf8(output_bytes).expect("Invalid UTF-8");
    let mut responses = Vec::new();
    for line in output_str.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let val: Value = serde_json::from_str(trimmed).expect("Failed to parse response JSON");
            responses.push(val);
        }
    }
    responses
}

#[tokio::test]
async fn test_mcp_initialize_and_ping() {
    let db = Arc::new(Database::in_memory().unwrap());

    let messages = vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test-client", "version": "1.0.0" }
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "ping"
        }),
    ];

    let responses = send_mcp_messages(db, &messages).await;

    // Only 2 responses because notification produces no reply
    assert_eq!(responses.len(), 2);

    // Check initialize response
    let init_res = &responses[0];
    assert_eq!(init_res["id"], 1);
    assert_eq!(init_res["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(init_res["result"]["serverInfo"]["name"], "rcal");
    assert!(init_res["result"]["capabilities"]["tools"].is_object());
    assert!(init_res["result"]["capabilities"]["resources"].is_object());
    assert!(init_res["result"]["capabilities"]["prompts"].is_object());

    // Check ping response
    let ping_res = &responses[1];
    assert_eq!(ping_res["id"], 2);
    assert_eq!(ping_res["result"], json!({}));
}

#[tokio::test]
async fn test_mcp_tools_list() {
    let db = Arc::new(Database::in_memory().unwrap());

    let messages = vec![json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/list"
    })];

    let responses = send_mcp_messages(db, &messages).await;
    assert_eq!(responses.len(), 1);

    let tools = responses[0]["result"]["tools"]
        .as_array()
        .expect("tools should be an array");

    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    assert!(tool_names.contains(&"list_calendars"));
    assert!(tool_names.contains(&"list_events"));
    assert!(tool_names.contains(&"get_event"));
    assert!(tool_names.contains(&"create_event"));
    assert!(tool_names.contains(&"update_event"));
    assert!(tool_names.contains(&"delete_event"));
    assert!(tool_names.contains(&"search_events"));
    assert!(tool_names.contains(&"find_free_slots"));
    assert!(tool_names.contains(&"get_current_time"));
    assert!(tool_names.contains(&"sync_calendars"));
}

#[tokio::test]
async fn test_mcp_tool_lifecycle() {
    let db = Arc::new(Database::in_memory().unwrap());

    // 1. List Calendars
    let list_cals_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "list_calendars",
            "arguments": {}
        }
    });

    let res = send_mcp_messages(db.clone(), &[list_cals_msg]).await;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0]["result"]["isError"], false);
    let cals_text = res[0]["result"]["content"][0]["text"].as_str().unwrap();
    assert!(cals_text.contains("Personal"));

    // 2. Create Event
    let create_msg = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "create_event",
            "arguments": {
                "title": "AI Agent Design Review",
                "start_time": "2026-11-10T14:00:00",
                "end_time": "2026-11-10T15:30:00",
                "description": "Discussing MCP architecture",
                "location": "Room 404"
            }
        }
    });

    let res = send_mcp_messages(db.clone(), &[create_msg]).await;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0]["result"]["isError"], false);
    let create_text = res[0]["result"]["content"][0]["text"].as_str().unwrap();
    let created_json: Value = serde_json::from_str(create_text).unwrap();
    let event_id = created_json["event"]["id"].as_str().unwrap().to_string();
    assert_eq!(created_json["event"]["title"], "AI Agent Design Review");

    // 3. Get Event by ID
    let get_msg = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "get_event",
            "arguments": {
                "id": &event_id
            }
        }
    });

    let res = send_mcp_messages(db.clone(), &[get_msg]).await;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0]["result"]["isError"], false);
    let get_text = res[0]["result"]["content"][0]["text"].as_str().unwrap();
    assert!(get_text.contains("Room 404"));

    // 4. Update Event
    let update_msg = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "update_event",
            "arguments": {
                "id": &event_id,
                "title": "AI Agent Architecture Sync",
                "location": "Virtual Meet"
            }
        }
    });

    let res = send_mcp_messages(db.clone(), &[update_msg]).await;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0]["result"]["isError"], false);
    let update_text = res[0]["result"]["content"][0]["text"].as_str().unwrap();
    assert!(update_text.contains("AI Agent Architecture Sync"));
    assert!(update_text.contains("Virtual Meet"));

    // 5. Search Events
    let search_msg = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "search_events",
            "arguments": {
                "query": "Architecture"
            }
        }
    });

    let res = send_mcp_messages(db.clone(), &[search_msg]).await;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0]["result"]["isError"], false);
    let search_text = res[0]["result"]["content"][0]["text"].as_str().unwrap();
    assert!(search_text.contains("AI Agent Architecture Sync"));

    // 6. List Events with range
    let list_events_msg = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "list_events",
            "arguments": {
                "start_date": "2026-11-10",
                "end_date": "2026-11-11"
            }
        }
    });

    let res = send_mcp_messages(db.clone(), &[list_events_msg]).await;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0]["result"]["isError"], false);
    let list_text = res[0]["result"]["content"][0]["text"].as_str().unwrap();
    assert!(list_text.contains("AI Agent Architecture Sync"));

    // 7. Delete Event
    let delete_msg = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "delete_event",
            "arguments": {
                "id": &event_id
            }
        }
    });

    let res = send_mcp_messages(db.clone(), &[delete_msg]).await;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0]["result"]["isError"], false);

    // 8. Verify Event is deleted
    let get_after_delete = json!({
        "jsonrpc": "2.0",
        "id": 8,
        "method": "tools/call",
        "params": {
            "name": "get_event",
            "arguments": {
                "id": &event_id
            }
        }
    });

    let res = send_mcp_messages(db.clone(), &[get_after_delete]).await;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0]["result"]["isError"], true);
}

#[tokio::test]
async fn test_mcp_find_free_slots() {
    let db = Arc::new(Database::in_memory().unwrap());

    // Schedule 2 events on 2026-11-15: 10:00-11:00 and 13:00-14:00
    let ev1 = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "create_event",
            "arguments": {
                "title": "Morning Standup",
                "start_time": "2026-11-15T10:00:00",
                "end_time": "2026-11-15T11:00:00"
            }
        }
    });

    let ev2 = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "create_event",
            "arguments": {
                "title": "Lunch Review",
                "start_time": "2026-11-15T13:00:00",
                "end_time": "2026-11-15T14:00:00"
            }
        }
    });

    let _ = send_mcp_messages(db.clone(), &[ev1, ev2]).await;

    // Find free slots for 30 minute meeting between 09:00 and 17:00
    let slots_msg = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "find_free_slots",
            "arguments": {
                "date": "2026-11-15",
                "duration_minutes": 30,
                "start_hour": 9,
                "end_hour": 17
            }
        }
    });

    let res = send_mcp_messages(db.clone(), &[slots_msg]).await;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0]["result"]["isError"], false);

    let slots_text = res[0]["result"]["content"][0]["text"].as_str().unwrap();
    let slots_json: Value = serde_json::from_str(slots_text).unwrap();

    let free_slots = slots_json["free_slots"].as_array().unwrap();
    assert_eq!(free_slots.len(), 3);

    // 09:00 - 10:00 (60 mins)
    assert_eq!(free_slots[0]["start_formatted"], "09:00");
    assert_eq!(free_slots[0]["end_formatted"], "10:00");
    assert_eq!(free_slots[0]["duration_minutes"], 60);

    // 11:00 - 13:00 (120 mins)
    assert_eq!(free_slots[1]["start_formatted"], "11:00");
    assert_eq!(free_slots[1]["end_formatted"], "13:00");
    assert_eq!(free_slots[1]["duration_minutes"], 120);

    // 14:00 - 17:00 (180 mins)
    assert_eq!(free_slots[2]["start_formatted"], "14:00");
    assert_eq!(free_slots[2]["end_formatted"], "17:00");
    assert_eq!(free_slots[2]["duration_minutes"], 180);
}

#[tokio::test]
async fn test_mcp_resources_and_prompts() {
    let db = Arc::new(Database::in_memory().unwrap());

    // 1. Resources list
    let res_list_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/list"
    });
    let res = send_mcp_messages(db.clone(), &[res_list_msg]).await;
    let resources = res[0]["result"]["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 3);

    // 2. Resource read
    let res_read_msg = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "resources/read",
        "params": {
            "uri": "rcal://calendars"
        }
    });
    let res = send_mcp_messages(db.clone(), &[res_read_msg]).await;
    assert_eq!(res[0]["result"]["contents"][0]["uri"], "rcal://calendars");

    // 3. Prompts list
    let prompt_list_msg = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "prompts/list"
    });
    let res = send_mcp_messages(db.clone(), &[prompt_list_msg]).await;
    let prompts = res[0]["result"]["prompts"].as_array().unwrap();
    assert_eq!(prompts.len(), 3);

    // 4. Prompts get
    let prompt_get_msg = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "prompts/get",
        "params": {
            "name": "daily_briefing",
            "arguments": {
                "date": "2026-11-01"
            }
        }
    });
    let res = send_mcp_messages(db.clone(), &[prompt_get_msg]).await;
    let messages = res[0]["result"]["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    let prompt_text = messages[0]["content"]["text"].as_str().unwrap();
    assert!(prompt_text.contains("2026-11-01"));
}

#[tokio::test]
async fn test_mcp_error_handling() {
    let db = Arc::new(Database::in_memory().unwrap());

    let messages = vec![
        // Unknown method
        json!({
            "jsonrpc": "2.0",
            "id": 100,
            "method": "unknown_function"
        }),
        // Tool call with non-existent tool
        json!({
            "jsonrpc": "2.0",
            "id": 101,
            "method": "tools/call",
            "params": {
                "name": "fly_to_moon"
            }
        }),
        // Invalid resource
        json!({
            "jsonrpc": "2.0",
            "id": 102,
            "method": "resources/read",
            "params": {
                "uri": "rcal://invalid_uri"
            }
        }),
    ];

    let responses = send_mcp_messages(db, &messages).await;
    assert_eq!(responses.len(), 3);

    // Unknown method error code -32601
    assert_eq!(responses[0]["error"]["code"], -32601);

    // Unknown tool returns ToolCallResult error with isError: true
    assert_eq!(responses[1]["result"]["isError"], true);

    // Invalid resource error code -32602
    assert_eq!(responses[2]["error"]["code"], -32602);
}
