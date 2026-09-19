use std::time::Duration;

use eframe::egui;
use std::collections::{HashMap, HashSet};
use sublens::{
    app::configure_style,
    jobs::{JobEvent, JobManager},
    protocols::parse_uri,
    resolver::{AnalysisReport, ResolverConfig},
    ui::config_card,
    ui::main_view::{self, CompactPage, ConnectivityFilter, DuplicateFilter, LteFilter},
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

#[test]
fn scroll_area_accepts_mouse_wheel_over_rows() {
    let context = egui::Context::default();
    let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(420.0, 160.0));
    let pointer = egui::pos2(80.0, 80.0);

    let render = |events: Vec<egui::Event>| {
        let mut offset = None;
        let _ = context.run(
            egui::RawInput {
                screen_rect: Some(viewport),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let output = egui::ScrollArea::vertical()
                        .id_salt("wheel-regression")
                        .max_height(120.0)
                        .min_scrolled_height(120.0)
                        .show_rows(ui, 40.0, 20, |ui, row_range| {
                            for row in row_range {
                                ui.label(format!("Configuration {row}"));
                            }
                        });
                    offset = Some(output.state.offset.y);
                });
            },
        );
        offset.expect("scroll area should render")
    };

    let before = render(vec![egui::Event::PointerMoved(pointer)]);
    let after = render(vec![egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, -3.0),
        modifiers: egui::Modifiers::default(),
    }]);

    assert!(
        after > before,
        "mouse wheel should move the scroll offset: before={before}, after={after}"
    );
}

#[test]
fn config_card_rows_accept_mouse_wheel() {
    let context = egui::Context::default();
    let config = parse_uri(
        "vless://12345678-1234-1234-1234-123456789abc@example.test:443#Example",
        None,
        0,
    )
    .unwrap();
    let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(420.0, 260.0));
    let pointer = egui::pos2(80.0, 160.0);

    let render = |events: Vec<egui::Event>| {
        let mut offset = None;
        let _ = context.run(
            egui::RawInput {
                screen_rect: Some(viewport),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let output = egui::ScrollArea::vertical()
                        .id_salt("card-wheel-regression")
                        .max_height(220.0)
                        .min_scrolled_height(220.0)
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .auto_shrink([false, false])
                        .show_rows(ui, 74.0, 20, |ui, row_range| {
                            for row in row_range {
                                ui.push_id(row, |ui| {
                                    config_card::show(ui, &config, false, false);
                                });
                                ui.add_space(6.0);
                            }
                        });
                    offset = Some(output.state.offset.y);
                });
            },
        );
        offset.expect("scroll area should render")
    };

    let before = render(vec![egui::Event::PointerMoved(pointer)]);
    let after = render(vec![egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, -3.0),
        modifiers: egui::Modifiers::default(),
    }]);

    assert!(
        after > before,
        "mouse wheel should move a config catalog: before={before}, after={after}"
    );
}

#[test]
fn main_view_catalog_accepts_mouse_wheel() {
    let context = egui::Context::default();
    configure_style(&context);
    let mut configs = Vec::new();
    for row in 0..20 {
        let mut config = parse_uri(
            &format!(
                "vless://12345678-1234-1234-1234-123456789abc@example{row}.test:443#Example-{row}"
            ),
            None,
            row,
        )
        .unwrap();
        config.id = format!("fixture-{row}");
        configs.push(config);
    }
    let report = AnalysisReport {
        configs,
        ..AnalysisReport::default()
    };
    let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 720.0));
    let pointer = egui::pos2(140.0, 430.0);

    let render = |events: Vec<egui::Event>| {
        let mut selected = Some("fixture-0".to_owned());
        let mut protocol_filters = HashSet::new();
        let mut selected_configs = HashSet::new();
        let mut search = String::new();
        let mut security_filter = None;
        let mut transport_filter = None;
        let mut connectivity_filter = ConnectivityFilter::All;
        let mut duplicate_filter = DuplicateFilter::All;
        let mut lte_filter = LteFilter::All;
        let mut selection_mode = false;
        let mut compact_page = CompactPage::ConfigList;
        let mut qr = None;
        let diagnostics = HashMap::new();
        let mut copied_until = None;
        let mut scroll_offset = None;
        let _ = context.run(
            egui::RawInput {
                screen_rect: Some(viewport),
                events,
                ..Default::default()
            },
            |ctx| {
                let result = main_view::show(
                    ctx,
                    &mut String::new(),
                    &[],
                    Some(&report),
                    &mut selected,
                    &mut protocol_filters,
                    &mut selected_configs,
                    &mut search,
                    &mut security_filter,
                    &mut transport_filter,
                    &mut connectivity_filter,
                    &mut duplicate_filter,
                    &mut lte_filter,
                    &mut selection_mode,
                    &mut compact_page,
                    false,
                    "Fixture loaded",
                    &mut qr,
                    &diagnostics,
                    &mut copied_until,
                );
                scroll_offset = result.catalog_scroll_offset;
            },
        );
        scroll_offset.expect("wide catalog scroll state should be reported")
    };

    let before = render(vec![egui::Event::PointerMoved(pointer)]);
    let after = render(vec![egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, -3.0),
        modifiers: egui::Modifiers::default(),
    }]);

    assert!(
        after > before,
        "main catalog should move on mouse wheel: before={before}, after={after}"
    );
}

#[test]
fn compact_main_view_catalog_accepts_mouse_wheel() {
    let context = egui::Context::default();
    configure_style(&context);
    let mut configs = Vec::new();
    for row in 0..20 {
        let mut config = parse_uri(
            &format!(
                "vless://12345678-1234-1234-1234-123456789abc@example{row}.test:443#Example-{row}"
            ),
            None,
            row,
        )
        .unwrap();
        config.id = format!("compact-fixture-{row}");
        configs.push(config);
    }
    let report = AnalysisReport {
        configs,
        ..AnalysisReport::default()
    };
    let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(980.0, 640.0));
    // The catalog controls sit above the list. Wheel input there should still
    // scroll the same catalog, since users perceive the whole column as one
    // scrollable surface.
    let pointer = egui::pos2(420.0, 170.0);

    let render = |events: Vec<egui::Event>| {
        let mut selected = Some("compact-fixture-0".to_owned());
        let mut protocol_filters = HashSet::new();
        let mut selected_configs = HashSet::new();
        let mut search = String::new();
        let mut security_filter = None;
        let mut transport_filter = None;
        let mut connectivity_filter = ConnectivityFilter::All;
        let mut duplicate_filter = DuplicateFilter::All;
        let mut lte_filter = LteFilter::All;
        let mut selection_mode = false;
        let mut compact_page = CompactPage::ConfigList;
        let mut qr = None;
        let diagnostics = HashMap::new();
        let mut copied_until = None;
        let mut scroll_offset = None;
        let _ = context.run(
            egui::RawInput {
                screen_rect: Some(viewport),
                events,
                ..Default::default()
            },
            |ctx| {
                let result = main_view::show(
                    ctx,
                    &mut String::new(),
                    &[],
                    Some(&report),
                    &mut selected,
                    &mut protocol_filters,
                    &mut selected_configs,
                    &mut search,
                    &mut security_filter,
                    &mut transport_filter,
                    &mut connectivity_filter,
                    &mut duplicate_filter,
                    &mut lte_filter,
                    &mut selection_mode,
                    &mut compact_page,
                    false,
                    "Fixture loaded",
                    &mut qr,
                    &diagnostics,
                    &mut copied_until,
                );
                scroll_offset = result.catalog_scroll_offset;
            },
        );
        scroll_offset.expect("compact catalog scroll state should be reported")
    };

    let before = render(vec![egui::Event::PointerMoved(pointer)]);
    let after = render(vec![egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, -3.0),
        modifiers: egui::Modifiers::default(),
    }]);

    assert!(
        after > before,
        "compact catalog should move on mouse wheel: before={before}, after={after}"
    );
}
