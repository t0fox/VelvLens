use std::time::Duration;

use sublens::{
    jobs::{JobEvent, JobManager},
    resolver::ResolverConfig,
};

#[test]
fn background_job_reports_invalid_input_without_blocking_the_caller() {
    let manager = JobManager::new().unwrap();
    let handle = manager.start_analysis("not-a-url".to_owned(), ResolverConfig::default());
    let mut events = Vec::new();
    while let Ok(event) = handle.receiver.recv_timeout(Duration::from_secs(2)) {
        let done = matches!(
            event,
            JobEvent::Failed(_) | JobEvent::Cancelled | JobEvent::Completed(_)
        );
        events.push(format!("{event:?}"));
        if done {
            break;
        }
    }
    assert!(events.iter().any(|event| event.contains("Started")));
    assert!(events.iter().any(|event| event.contains("Failed")));
}
