//! NexusOffices: what the MCP server tells an agent names Nexus Slides and its installed binaries
//! (`nexus-slides`, `nexus-slides-cli`); the machine name `serverInfo.name` stays `deckcraft`.

use deckcraft_mcp::{Headless, Server, tool_definitions};
use serde_json::{Value, json};

fn upstream_free(text: &str) {
    // `deck.deckcraft --print` (a file name, then a flag) is fine; the app binary is not.
    for bad in ["DeckCraft", "ArtCraft", "deckcraft-cli", "`deckcraft ", " deckcraft --control", "artcraft", "discord", "storytold"] {
        assert!(!text.contains(bad), "{bad:?} in: {text}");
    }
}

#[test]
fn tool_definitions_name_nexus_binaries() {
    let tools = Value::Array(tool_definitions()).to_string();
    upstream_free(&tools);
    assert!(tools.contains("nexus-slides-cli mcp --connect"), "the app-only hint names the installed CLI");
}

#[test]
fn server_info_is_nexus_slides() {
    let mut server = Server::new(Box::new(Headless::new()));
    let line = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18"}}).to_string();
    let reply: Value = serde_json::from_str(&server.handle_line(&line).unwrap()).unwrap();
    let info = &reply["result"]["serverInfo"];
    assert_eq!(info["name"], "deckcraft", "machine name unchanged");
    assert_eq!(info["title"], "Nexus Slides");
    assert_eq!(info["websiteUrl"], "https://github.com/nhdai111983/deckcraft");
    let instructions = reply["result"]["instructions"].as_str().unwrap();
    assert!(instructions.starts_with("Nexus Slides"), "{instructions}");
    upstream_free(&reply.to_string());
}
