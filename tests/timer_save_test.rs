use timber_task::state::timer_state::TimerState;
use timber_task::state::save_request::SaveRequest;

#[test]
fn test_timer_methods_return_save_requests() {
    let mut timer = TimerState::default();
    
    // Test start() returns SaveRequest::Full
    let save_req = timer.start();
    assert!(save_req.is_needed());
    
    // Test pause() returns SaveRequest::Full
    let save_req = timer.pause();
    assert!(save_req.is_needed());
    
    // Test reset() returns SaveRequest::Full
    let save_req = timer.reset();
    assert!(save_req.is_needed());
    
    // Test switch_period() returns SaveRequest::Full
    let save_req = timer.switch_period();
    assert!(save_req.is_needed());
    
    // Test complete_period() returns SaveRequest::Full
    let save_req = timer.complete_period();
    assert!(save_req.is_needed());
    
    // Test set_current_task() returns SaveRequest::Full
    let save_req = timer.set_current_task(Some("task123".to_string()));
    assert!(save_req.is_needed());
}

#[test]
fn test_timer_tick_periodic_save() {
    let mut timer = TimerState::default();
    
    // When timer is not running, tick should return None
    timer.is_running = false;
    let save_req = timer.tick();
    assert!(!save_req.is_needed());
    
    // Start timer and test periodic saves
    timer.start();
    
    // First 9 ticks should not trigger save
    for _ in 0..9 {
        let save_req = timer.tick();
        assert!(!save_req.is_needed());
    }
    
    // 10th tick should trigger save
    let save_req = timer.tick();
    assert!(save_req.is_needed());
    
    // Next 9 ticks should not trigger save
    for _ in 0..9 {
        let save_req = timer.tick();
        assert!(!save_req.is_needed());
    }
    
    // 20th tick should trigger save again
    let save_req = timer.tick();
    assert!(save_req.is_needed());
}

#[test]
fn test_skip_with_actual_elapsed_time() {
    let mut timer = TimerState::default();
    
    // Start the timer
    timer.start();
    
    // Simulate some time passing by manually setting the start time
    use std::time::{Duration, Instant};
    timer.start_time = Some(Instant::now() - Duration::from_secs(10 * 60)); // 10 minutes ago
    
    // When we skip, it should calculate the actual elapsed time
    let remaining_before = timer.get_remaining_seconds();
    assert_eq!(remaining_before, 15 * 60); // Should be 15 minutes remaining (25 - 10)
    
    // Skip the period
    let save_req = timer.complete_period();
    assert!(save_req.is_needed());
    
    // Timer should now be in break period
    assert!(!timer.is_work_period);
    assert_eq!(timer.completed_pomodoros, 1);
}