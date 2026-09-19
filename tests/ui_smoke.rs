use std::time::Duration;

use eframe::egui;
use sublens::{
    jobs::{JobEvent, JobManager},
    protocols::parse_uri,
    resolver::ResolverConfig,
    ui::config_card,
};

#[test]
fn background_job_reports_invalid_input_without_blocking_the_caller() {
    let manager = JobManager::new().unwrap();
    let handle = manager.start_analysis("not-a-url".to_owned(), ResolverConfig::default());
    let mut events = Vec::new();
    while let Ok(event) = handle.receiver.recv_timeout(Duration::from_secs(2)) {
        let done = matches!(
            event,
            JobEvent::Failed(_)
                | JobEvent::Cancelled
                | JobEvent::Completed(_)
                | JobEvent::DiagnosticCompleted(_)
        );
        events.push(format!("{event:?}"));
        if done {
            break;
        }
    }
    assert!(events.iter().any(|event| event.contains("Started")));
    assert!(events.iter().any(|event| event.contains("Failed")));
}

#[test]
fn background_job_forwards_sanitized_pipeline_stages() {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request);
        let body = "vless://12345678-1234-1234-1234-123456789abc@stage.example:443";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes());
    });

    let manager = JobManager::new().unwrap();
    let handle = manager.start_analysis(
        format!("http://{address}/subscription"),
        ResolverConfig::default(),
    );
    let mut events = Vec::new();
    while let Ok(event) = handle.receiver.recv_timeout(Duration::from_secs(2)) {
        let done = matches!(event, JobEvent::Completed(_) | JobEvent::Failed(_));
        events.push(format!("{event:?}"));
        if done {
            break;
        }
    }
    let stage_position = events
        .iter()
        .position(|event| event.contains("Stage"))
        .expect("analysis should emit a sanitized stage");
    let completed_position = events
        .iter()
        .position(|event| event.contains("Completed"))
        .expect("analysis should complete");
    assert!(stage_position < completed_position);
}

#[test]
fn config_card_body_is_clickable_without_using_the_details_button() {
    let context = egui::Context::default();
    let config = parse_uri(
        "vless://12345678-1234-1234-1234-123456789abc@example.test:443#Example",
        None,
        0,
    )
    .unwrap();
    let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(420.0, 160.0));
    let body_click = egui::pos2(80.0, 34.0);

    let render_card = |input: egui::RawInput| {
        let mut response = None;
        let _ = context.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                response = Some(config_card::show(ui, &config, false, false));
            });
        });
        response.expect("card should render").clicked
    };

    render_card(egui::RawInput {
        screen_rect: Some(viewport),
        events: vec![egui::Event::PointerMoved(body_click)],
        ..Default::default()
    });
    render_card(egui::RawInput {
        screen_rect: Some(viewport),
        events: vec![egui::Event::PointerButton {
            pos: body_click,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        }],
        ..Default::default()
    });
    let clicked = render_card(egui::RawInput {
        screen_rect: Some(viewport),
        events: vec![egui::Event::PointerButton {
            pos: body_click,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        }],
        ..Default::default()
    });

    assert!(clicked, "clicking the profile body must select the profile");
}
