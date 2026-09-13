use super::*;

#[tokio::test]
async fn cloud_exec_disconnect_after_send_is_not_replayed() {
    use microsandbox_protocol::{
        codec,
        core::Ready,
        message::{Message, MessageType},
    };
    use tokio::io::AsyncWriteExt;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backend: Arc<dyn Backend> = Arc::new(
        CloudBackend::new(
            format!("http://{}", listener.local_addr().unwrap()),
            "test-key",
        )
        .unwrap(),
    );
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let websocket = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut stream = ws_io::WsByteStream::new(websocket);
        stream.write_all(&1u32.to_be_bytes()).await.unwrap();
        stream.write_all(&1024u32.to_be_bytes()).await.unwrap();
        let ready = Message::with_payload(
            MessageType::Ready,
            0,
            &Ready {
                boot_time_ns: 0,
                init_time_ns: 0,
                ready_time_ns: 0,
                agent_version: "ambiguous-exec-test".into(),
            },
        )
        .unwrap();
        codec::write_message(&mut stream, &ready).await.unwrap();
        let request = codec::read_raw_frame(&mut stream).await.unwrap();
        assert_eq!(
            codec::raw_frame_to_message(request).unwrap().t,
            MessageType::ExecRequest
        );
        // The command may have executed. Losing its result must not cause a retry.
        drop(stream);
        listener
    });
    let sandbox = crate::sandbox::Sandbox::from_cloud_state(
        backend,
        crate::backend::SandboxCloudState {
            id: "captured-id".into(),
            org_id: "org".into(),
            created_at: chrono::Utc::now(),
        },
        "ambiguous".into(),
        crate::sandbox::SandboxConfig::default(),
    );
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        sandbox.exec("write-once", std::iter::empty::<&str>()),
    )
    .await
    .unwrap();
    assert!(result.is_err());
    let listener = server.await.unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(50), listener.accept())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn consecutive_cloud_execs_reuse_one_websocket() {
    use microsandbox_protocol::{
        codec,
        core::Ready,
        exec::ExecExited,
        message::{Message, MessageType},
    };
    use tokio::io::AsyncWriteExt;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backend: Arc<dyn Backend> = Arc::new(
        CloudBackend::new(
            format!("http://{}", listener.local_addr().unwrap()),
            "test-key",
        )
        .unwrap(),
    );
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let websocket = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut stream = ws_io::WsByteStream::new(websocket);
        stream.write_all(&1u32.to_be_bytes()).await.unwrap();
        stream.write_all(&1024u32.to_be_bytes()).await.unwrap();
        let ready = Message::with_payload(
            MessageType::Ready,
            0,
            &Ready {
                boot_time_ns: 0,
                init_time_ns: 0,
                ready_time_ns: 0,
                agent_version: "cloud-pool-test".into(),
            },
        )
        .unwrap();
        codec::write_message(&mut stream, &ready).await.unwrap();
        let mut ids = Vec::new();
        for _ in 0..2 {
            // A second WebSocket means the public exec path missed the pool.
            let request = tokio::select! {
                frame = codec::read_raw_frame(&mut stream) => frame.unwrap(),
                _ = listener.accept() => panic!("consecutive exec opened another connection"),
            };
            let message = codec::raw_frame_to_message(request).unwrap();
            assert_eq!(message.t, MessageType::ExecRequest);
            ids.push(message.id);
            let exited =
                Message::with_payload(MessageType::ExecExited, message.id, &ExecExited { code: 0 })
                    .unwrap();
            codec::write_message(&mut stream, &exited).await.unwrap();
        }
        assert_ne!(ids[0], ids[1]);
    });
    let sandbox = crate::sandbox::Sandbox::from_cloud_state(
        backend,
        crate::backend::SandboxCloudState {
            id: "captured-id".into(),
            org_id: "org".into(),
            created_at: chrono::Utc::now(),
        },
        "pooled".into(),
        crate::sandbox::SandboxConfig::default(),
    );
    tokio::time::timeout(Duration::from_secs(3), async {
        assert!(
            sandbox
                .exec("true", std::iter::empty::<&str>())
                .await
                .unwrap()
                .status()
                .success
        );
        assert!(
            sandbox
                .exec("true", std::iter::empty::<&str>())
                .await
                .unwrap()
                .status()
                .success
        );
        server.await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn replacement_handle_cannot_borrow_original_handles_idle_connection() {
    use microsandbox_protocol::{
        codec,
        core::Ready,
        exec::ExecExited,
        message::{Message, MessageType},
    };
    use tokio::io::AsyncWriteExt;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backend: Arc<dyn Backend> = Arc::new(
        CloudBackend::new(
            format!("http://{}", listener.local_addr().unwrap()),
            "test-key",
        )
        .unwrap(),
    );
    let server = tokio::spawn(async move {
        let mut paths = Vec::new();
        let mut held = Vec::new();
        for _ in 0..2 {
            let (stream, _) = listener.accept().await.unwrap();
            let websocket = tokio_tungstenite::accept_hdr_async(
                stream,
                |request: &tokio_tungstenite::tungstenite::handshake::server::Request, response| {
                    paths.push(request.uri().path().to_owned());
                    Ok(response)
                },
            )
            .await
            .unwrap();
            let mut stream = ws_io::WsByteStream::new(websocket);
            stream.write_all(&1u32.to_be_bytes()).await.unwrap();
            stream.write_all(&1024u32.to_be_bytes()).await.unwrap();
            let ready = Message::with_payload(
                MessageType::Ready,
                0,
                &Ready {
                    boot_time_ns: 0,
                    init_time_ns: 0,
                    ready_time_ns: 0,
                    agent_version: "identity-pool-test".into(),
                },
            )
            .unwrap();
            codec::write_message(&mut stream, &ready).await.unwrap();
            let request = codec::read_raw_frame(&mut stream).await.unwrap();
            let exited =
                Message::with_payload(MessageType::ExecExited, request.id, &ExecExited { code: 0 })
                    .unwrap();
            codec::write_message(&mut stream, &exited).await.unwrap();
            // Keep the original socket healthy so an incorrect shared pool
            // would reuse it, rather than accidentally passing via EOF recovery.
            held.push(stream);
        }
        paths
    });
    let handle = |backend, id: &str| {
        crate::sandbox::Sandbox::from_cloud_state(
            backend,
            crate::backend::SandboxCloudState {
                id: id.into(),
                org_id: "org".into(),
                created_at: chrono::Utc::now(),
            },
            "reused-name".into(),
            crate::sandbox::SandboxConfig::default(),
        )
    };
    let original = handle(backend, "original-id");
    tokio::time::timeout(Duration::from_secs(3), async {
        original
            .exec("true", std::iter::empty::<&str>())
            .await
            .unwrap();
        let replacement = handle(original.backend().clone(), "replacement-id");
        replacement
            .exec("true", std::iter::empty::<&str>())
            .await
            .unwrap();
        assert_eq!(
            server.await.unwrap(),
            [
                "/v1/sandboxes/original-id/agent",
                "/v1/sandboxes/replacement-id/agent",
            ]
        );
    })
    .await
    .unwrap();
}
