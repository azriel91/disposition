//! End-to-end test driving the real [`async_lsp::MainLoop`] over in-memory,
//! `Content-Length`-framed byte streams -- the same wiring used in the browser,
//! minus the editor. Exercises `initialize` -> `didOpen` -> `completion`.

use disposition_lsp::transport::{byte_pipe, frame, read_message, PipeReader, PipeWriter};
use futures::{executor::block_on, io::BufReader};
use serde_json::{json, Value};

#[test]
fn completion_over_main_loop_returns_top_level_keys() {
    // client -> server and server -> client byte pipes.
    let (client_writer, server_reader) = byte_pipe();
    let (server_writer, client_reader) = byte_pipe();
    let mut client_reader = BufReader::new(client_reader);

    let server = disposition_lsp::server_run(server_reader, server_writer);

    let client = async move {
        // initialize.
        write_message(
            &client_writer,
            &json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": { "capabilities": {} }
            }),
        );
        let initialize_response = read_response(&mut client_reader, 1).await;
        assert!(
            initialize_response["result"]["capabilities"]["completionProvider"].is_object(),
            "expected completionProvider capability, got {initialize_response}"
        );

        // initialized + didOpen (empty document).
        write_message(
            &client_writer,
            &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }),
        );
        write_message(
            &client_writer,
            &json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": "file:///diagram.yaml",
                        "languageId": "yaml",
                        "version": 1,
                        "text": ""
                    }
                }
            }),
        );

        // completion at the start of the empty document.
        write_message(
            &client_writer,
            &json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "textDocument/completion",
                "params": {
                    "textDocument": { "uri": "file:///diagram.yaml" },
                    "position": { "line": 0, "character": 0 }
                }
            }),
        );
        let completion_response = read_response(&mut client_reader, 2).await;
        let labels = completion_response["result"]
            .as_array()
            .expect("expected completion result array")
            .iter()
            .map(|item| item["label"].as_str().unwrap_or_default().to_string())
            .collect::<Vec<String>>();
        assert!(
            labels.iter().any(|label| label == "things"),
            "expected `things` among completions, got {labels:?}"
        );

        // shutdown + exit so the server `MainLoop` returns.
        write_message(
            &client_writer,
            &json!({ "jsonrpc": "2.0", "id": 3, "method": "shutdown" }),
        );
        let _ = read_response(&mut client_reader, 3).await;
        write_message(
            &client_writer,
            &json!({ "jsonrpc": "2.0", "method": "exit" }),
        );
    };

    let (server_result, ()) = block_on(async { futures::join!(server, client) });
    server_result.expect("server main loop errored");
}

/// Writes `message` to `writer` with an LSP `Content-Length` frame.
fn write_message(writer: &PipeWriter, message: &Value) {
    let body = serde_json::to_string(message).expect("serialize message");
    writer.send_bytes(frame(&body));
}

/// Reads framed messages from `reader` until one with `id` arrives.
async fn read_response(reader: &mut BufReader<PipeReader>, id: i64) -> Value {
    loop {
        let json = read_message(reader).await.expect("expected a message");
        let message: Value = serde_json::from_str(&json).expect("valid JSON message");
        if message["id"].as_i64() == Some(id) {
            return message;
        }
    }
}
