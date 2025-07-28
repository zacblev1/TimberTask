/// Common test utilities and fixtures for TimberTask tests
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use timber_task::state::{timer_state::TimerState, kanban_state::KanbanState, notes_state::NotesState};
use tempfile::TempDir;
use uuid::Uuid;

/// Test fixture that creates a temporary directory for tests
pub struct TestFixture {
    pub temp_dir: TempDir,
    pub data_dir: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with a temporary directory
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let data_dir = temp_dir.path().join(".timber-task");
        fs::create_dir_all(&data_dir).expect("Failed to create data dir");
        
        Self {
            temp_dir,
            data_dir,
        }
    }
    
    /// Get the path to the temporary data directory
    pub fn data_path(&self) -> &Path {
        &self.data_dir
    }
    
    /// Create a test timer state
    pub fn create_timer_state() -> Arc<Mutex<TimerState>> {
        Arc::new(Mutex::new(TimerState::default()))
    }
    
    /// Create a test kanban state with temporary directory
    pub fn create_kanban_state(&self) -> KanbanState {
        let mut state = KanbanState::default();
        // Override the data file path for testing
        state.data_file_path = self.data_dir.join("kanban_data.json");
        state
    }
    
    /// Create a test notes state with temporary directory
    pub fn create_notes_state(&self) -> NotesState {
        let mut state = NotesState::default();
        // Override the data file path for testing
        state.data_file_path = self.data_dir.join("notes_data.json");
        state
    }
}

/// Factory for creating test data
pub struct TestFactory;

impl TestFactory {
    /// Create a test task with default values
    pub fn create_task(title: &str) -> timber_task::state::kanban_state::Task {
        use timber_task::state::kanban_state::{Task, TaskStatus};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Task {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: format!("Description for {}", title),
            status: TaskStatus::Todo,
            time_spent: 0,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create a test project with default values
    pub fn create_project(name: &str) -> timber_task::state::kanban_state::Project {
        use timber_task::state::kanban_state::Project;
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Project {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            tasks: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create a test note with default values
    pub fn create_note(title: &str) -> timber_task::state::notes_state::Note {
        use timber_task::state::notes_state::Note;
        use std::collections::HashSet;
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Note {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            content: format!("Content for {}", title),
            parent_id: None,
            children: Vec::new(),
            tags: HashSet::new(),
            expanded: false,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create a test tag
    pub fn create_tag(name: &str) -> timber_task::state::notes_state::Tag {
        use timber_task::state::notes_state::Tag;
        
        Tag {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            color: Some("#007ACC".to_string()),
        }
    }
}

/// Helper macro to assert that an operation returns an error
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        assert!($result.is_err(), "Expected error but got Ok");
    };
    ($result:expr, $pattern:pat) => {
        match $result {
            Err($pattern) => {},
            Ok(_) => panic!("Expected error matching {} but got Ok", stringify!($pattern)),
            Err(e) => panic!("Expected error matching {} but got {:?}", stringify!($pattern), e),
        }
    };
}

/// Helper macro to assert that a mutex can be locked
#[macro_export]
macro_rules! assert_mutex_locked {
    ($mutex:expr) => {
        assert!($mutex.try_lock().is_ok(), "Mutex should not be locked");
    };
}

/// Mock timer for testing time-dependent features
pub struct MockTimer {
    current_time: std::time::Instant,
}

impl MockTimer {
    pub fn new() -> Self {
        Self {
            current_time: std::time::Instant::now(),
        }
    }
    
    pub fn advance(&mut self, duration: std::time::Duration) {
        // In real tests, we'd need to mock the Instant::now() calls
        // For now, this serves as a placeholder for the pattern
        self.current_time += duration;
    }
}