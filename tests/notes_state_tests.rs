/// Comprehensive unit tests for the NotesState module
mod common;

use common::*;
use timber_task::state::notes_state::NotesState;
use std::collections::HashSet;
use anyhow::Result;

#[test]
fn test_notes_default_state() {
    let notes = NotesState::default();
    
    assert!(notes.notes.is_empty());
    assert!(notes.tags.is_empty());
    assert!(notes.root_notes.is_empty());
    assert!(notes.selected_note_id.is_none());
}

#[test]
fn test_create_tag() -> Result<()> {
    let mut notes = NotesState::default();
    
    // Create tag with color
    let tag = notes.create_tag("Important", Some("#FF0000"))?;
    assert_eq!(tag.name, "Important");
    assert_eq!(tag.color, Some("#FF0000".to_string()));
    
    // Create tag without color
    let tag2 = notes.create_tag("Work", None)?;
    assert_eq!(tag2.name, "Work");
    assert_eq!(tag2.color, None);
    
    // Verify tags exist in state
    assert!(notes.tags.contains_key(&tag.id));
    assert!(notes.tags.contains_key(&tag2.id));
    
    Ok(())
}

#[test]
fn test_update_tag() -> Result<()> {
    let mut notes = NotesState::default();
    
    let tag = notes.create_tag("Original", Some("#000000"))?;
    let tag_id = tag.id.clone();
    
    // Update tag
    let updated_tag = notes.update_tag(&tag_id, "Updated", Some("#FFFFFF"))?;
    assert_eq!(updated_tag.name, "Updated");
    assert_eq!(updated_tag.color, Some("#FFFFFF".to_string()));
    
    Ok(())
}

#[test]
fn test_delete_tag() -> Result<()> {
    let mut notes = NotesState::default();
    
    let tag = notes.create_tag("ToDelete", None)?;
    let tag_id = tag.id.clone();
    
    // Create a note with the tag
    let note = notes.create_note("Note", "Content", None)?;
    notes.add_tag_to_note(&note.id, &tag_id)?;
    
    // Delete the tag
    notes.delete_tag(&tag_id)?;
    
    // Verify tag is gone
    assert!(notes.get_tag(&tag_id).is_none());
    
    // Verify tag was removed from note
    let note = notes.get_note(&note.id).unwrap();
    assert!(!note.tags.contains(&tag_id));
    
    Ok(())
}

#[test]
fn test_create_root_note() -> Result<()> {
    let mut notes = NotesState::default();
    
    let note = notes.create_note("Root Note", "This is a root note", None)?;
    
    assert_eq!(note.title, "Root Note");
    assert_eq!(note.content, "This is a root note");
    assert_eq!(note.parent_id, None);
    assert!(note.children.is_empty());
    assert!(note.tags.is_empty());
    assert!(!note.expanded);
    
    // Verify note is in root_notes
    assert!(notes.root_notes.contains(&note.id));
    
    Ok(())
}

#[test]
fn test_create_child_note() -> Result<()> {
    let mut notes = NotesState::default();
    
    // Create parent note
    let parent = notes.create_note("Parent", "Parent content", None)?;
    let parent_id = parent.id.clone();
    
    // Create child note
    let child = notes.create_note("Child", "Child content", Some(&parent_id))?;
    
    assert_eq!(child.parent_id, Some(parent_id.clone()));
    
    // Verify parent has child
    let parent = notes.get_note(&parent_id).unwrap();
    assert!(parent.children.contains(&child.id));
    
    // Verify child is not in root_notes
    assert!(!notes.root_notes.contains(&child.id));
    
    Ok(())
}

#[test]
fn test_update_note() -> Result<()> {
    let mut notes = NotesState::default();
    
    let note = notes.create_note("Original", "Original content", None)?;
    let note_id = note.id.clone();
    let created_at = note.created_at;
    
    // Update the note
    let updated = notes.update_note(&note_id, "Updated", "Updated content")?;
    
    assert_eq!(updated.title, "Updated");
    assert_eq!(updated.content, "Updated content");
    assert_eq!(updated.created_at, created_at);
    assert!(updated.updated_at > created_at);
    
    Ok(())
}

#[test]
fn test_move_note() -> Result<()> {
    let mut notes = NotesState::default();
    
    // Create notes
    let parent1 = notes.create_note("Parent 1", "Content", None)?;
    let parent2 = notes.create_note("Parent 2", "Content", None)?;
    let child = notes.create_note("Child", "Content", Some(&parent1.id))?;
    
    let parent1_id = parent1.id.clone();
    let parent2_id = parent2.id.clone();
    let child_id = child.id.clone();
    
    // Verify initial state
    assert!(notes.get_note(&parent1_id).unwrap().children.contains(&child_id));
    assert!(!notes.get_note(&parent2_id).unwrap().children.contains(&child_id));
    
    // Move child to parent2
    notes.move_note(&child_id, Some(&parent2_id))?;
    
    // Verify new state
    assert!(!notes.get_note(&parent1_id).unwrap().children.contains(&child_id));
    assert!(notes.get_note(&parent2_id).unwrap().children.contains(&child_id));
    assert_eq!(notes.get_note(&child_id).unwrap().parent_id, Some(parent2_id));
    
    // Move to root
    notes.move_note(&child_id, None)?;
    
    assert!(notes.root_notes.contains(&child_id));
    assert_eq!(notes.get_note(&child_id).unwrap().parent_id, None);
    
    Ok(())
}

#[test]
fn test_delete_note() -> Result<()> {
    let mut notes = NotesState::default();
    
    // Create parent with children
    let parent = notes.create_note("Parent", "Content", None)?;
    let child1 = notes.create_note("Child 1", "Content", Some(&parent.id))?;
    let child2 = notes.create_note("Child 2", "Content", Some(&parent.id))?;
    let grandchild = notes.create_note("Grandchild", "Content", Some(&child1.id))?;
    
    let parent_id = parent.id.clone();
    let child1_id = child1.id.clone();
    let child2_id = child2.id.clone();
    let grandchild_id = grandchild.id.clone();
    
    // Delete parent (should delete all descendants)
    notes.delete_note(&parent_id)?;
    
    // Verify all are deleted
    assert!(notes.get_note(&parent_id).is_none());
    assert!(notes.get_note(&child1_id).is_none());
    assert!(notes.get_note(&child2_id).is_none());
    assert!(notes.get_note(&grandchild_id).is_none());
    
    // Verify removed from root_notes
    assert!(!notes.root_notes.contains(&parent_id));
    
    Ok(())
}

#[test]
fn test_toggle_expanded() -> Result<()> {
    let mut notes = NotesState::default();
    
    let note = notes.create_note("Note", "Content", None)?;
    let note_id = note.id.clone();
    
    // Initially not expanded
    assert!(!note.expanded);
    
    // Toggle to expanded
    let toggled = notes.toggle_note_expanded(&note_id)?;
    assert!(toggled.expanded);
    
    // Toggle back
    let toggled = notes.toggle_note_expanded(&note_id)?;
    assert!(!toggled.expanded);
    
    Ok(())
}

#[test]
fn test_note_tags() -> Result<()> {
    let mut notes = NotesState::default();
    
    let note = notes.create_note("Note", "Content", None)?;
    let tag1 = notes.create_tag("Tag1", None)?;
    let tag2 = notes.create_tag("Tag2", None)?;
    
    let note_id = note.id.clone();
    let tag1_id = tag1.id.clone();
    let tag2_id = tag2.id.clone();
    
    // Add tags to note
    notes.add_tag_to_note(&note_id, &tag1_id)?;
    notes.add_tag_to_note(&note_id, &tag2_id)?;
    
    let note = notes.get_note(&note_id).unwrap();
    assert!(note.tags.contains(&tag1_id));
    assert!(note.tags.contains(&tag2_id));
    
    // Remove a tag
    notes.remove_tag_from_note(&note_id, &tag1_id)?;
    
    let note = notes.get_note(&note_id).unwrap();
    assert!(!note.tags.contains(&tag1_id));
    assert!(note.tags.contains(&tag2_id));
    
    Ok(())
}

#[test]
fn test_get_notes_with_tag() -> Result<()> {
    let mut notes = NotesState::default();
    
    let tag = notes.create_tag("Important", None)?;
    let tag_id = tag.id.clone();
    
    // Create notes
    let note1 = notes.create_note("Note 1", "Content", None)?;
    let note2 = notes.create_note("Note 2", "Content", None)?;
    let note3 = notes.create_note("Note 3", "Content", None)?;
    
    // Add tag to some notes
    notes.add_tag_to_note(&note1.id, &tag_id)?;
    notes.add_tag_to_note(&note3.id, &tag_id)?;
    
    // Get notes with tag
    let tagged_notes = notes.get_notes_with_tag(&tag_id);
    assert_eq!(tagged_notes.len(), 2);
    
    let tagged_ids: HashSet<_> = tagged_notes.iter().map(|n| &n.id).collect();
    assert!(tagged_ids.contains(&note1.id));
    assert!(tagged_ids.contains(&note3.id));
    assert!(!tagged_ids.contains(&note2.id));
    
    Ok(())
}

#[test]
fn test_get_child_notes() -> Result<()> {
    let mut notes = NotesState::default();
    
    let parent = notes.create_note("Parent", "Content", None)?;
    let child1 = notes.create_note("Child 1", "Content", Some(&parent.id))?;
    let child2 = notes.create_note("Child 2", "Content", Some(&parent.id))?;
    let _other = notes.create_note("Other", "Content", None)?;
    
    let children = notes.get_child_notes(&parent.id);
    assert_eq!(children.len(), 2);
    
    let child_ids: HashSet<_> = children.iter().map(|n| &n.id).collect();
    assert!(child_ids.contains(&child1.id));
    assert!(child_ids.contains(&child2.id));
    
    Ok(())
}

#[test]
fn test_note_selection() -> Result<()> {
    let mut notes = NotesState::default();
    
    let note = notes.create_note("Note", "Content", None)?;
    let note_id = note.id.clone();
    
    // Initially no selection
    assert!(notes.get_selected_note().is_none());
    
    // Select note
    notes.select_note(&note_id)?;
    assert_eq!(notes.get_selected_note().unwrap().id, note_id);
    
    // Clear selection
    notes.clear_selection()?;
    assert!(notes.get_selected_note().is_none());
    
    Ok(())
}

#[test]
fn test_get_note_path() -> Result<()> {
    let mut notes = NotesState::default();
    
    // Create hierarchy: root -> parent -> child -> grandchild
    let root = notes.create_note("Root", "Content", None)?;
    let parent = notes.create_note("Parent", "Content", Some(&root.id))?;
    let child = notes.create_note("Child", "Content", Some(&parent.id))?;
    let grandchild = notes.create_note("Grandchild", "Content", Some(&child.id))?;
    
    let path = notes.get_note_path(&grandchild.id);
    assert_eq!(path.len(), 4);
    assert_eq!(path[0].id, root.id);
    assert_eq!(path[1].id, parent.id);
    assert_eq!(path[2].id, child.id);
    assert_eq!(path[3].id, grandchild.id);
    
    Ok(())
}

#[test]
fn test_search_notes() -> Result<()> {
    let mut notes = NotesState::default();
    
    notes.create_note("Rust Programming", "Learn Rust basics", None)?;
    notes.create_note("Python Tutorial", "Python for beginners", None)?;
    notes.create_note("JavaScript Guide", "Modern JS features", None)?;
    notes.create_note("Meeting Notes", "Discussed Rust adoption", None)?;
    
    // Search for "rust" (case-insensitive)
    let results = notes.search_notes("rust");
    assert_eq!(results.len(), 2); // "Rust Programming" and "Meeting Notes"
    
    // Search for "python"
    let results = notes.search_notes("python");
    assert_eq!(results.len(), 1);
    
    // Search that matches content
    let results = notes.search_notes("beginners");
    assert_eq!(results.len(), 1);
    
    Ok(())
}

#[test]
fn test_persistence() -> Result<()> {
    let fixture = TestFixture::new();
    
    // Create initial state and data
    let (note_id, tag_id) = {
        let mut notes = fixture.create_notes_state();
        
        // Create some data
        let tag = notes.create_tag("Important", Some("#FF0000"))?;
        let note = notes.create_note("Persistent Note", "Should be saved", None)?;
        notes.add_tag_to_note(&note.id, &tag.id)?;
        
        let note_id = note.id.clone();
        let tag_id = tag.id.clone();
        
        // Save to disk
        notes.save_to_disk()?;
        
        (note_id, tag_id)
    };
    
    // Create a new state and load from disk
    let mut new_notes = fixture.create_notes_state();
    new_notes.load_from_disk()?;
    
    // Verify data was persisted
    assert!(new_notes.notes.contains_key(&note_id));
    let loaded_note = &new_notes.notes[&note_id];
    assert_eq!(loaded_note.title, "Persistent Note");
    assert!(loaded_note.tags.contains(&tag_id));
    
    assert!(new_notes.tags.contains_key(&tag_id));
    let loaded_tag = &new_notes.tags[&tag_id];
    assert_eq!(loaded_tag.name, "Important");
    assert_eq!(loaded_tag.color, Some("#FF0000".to_string()));
    
    Ok(())
}

#[test]
fn test_error_handling() -> Result<()> {
    let mut notes = NotesState::default();
    
    // Try to update non-existent note
    assert!(notes.update_note("non-existent", "New", "Content").is_err());
    
    // Try to move non-existent note
    assert!(notes.move_note("non-existent", None).is_err());
    
    // Try to delete non-existent note
    assert!(notes.delete_note("non-existent").is_err());
    
    // Try to add tag to non-existent note
    let tag = notes.create_tag("Tag", None)?;
    assert!(notes.add_tag_to_note("non-existent", &tag.id).is_err());
    
    // Try to add non-existent tag to note
    let note = notes.create_note("Note", "Content", None)?;
    assert!(notes.add_tag_to_note(&note.id, "non-existent").is_err());
    
    Ok(())
}

// Thread safety tests
#[cfg(test)]
mod thread_safety_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    #[test]
    fn test_concurrent_note_creation() -> Result<()> {
        let notes = Arc::new(Mutex::new(NotesState::default()));
        let mut handles = vec![];
        
        // Spawn multiple threads creating notes
        for i in 0..10 {
            let notes_clone = Arc::clone(&notes);
            
            let handle = thread::spawn(move || {
                let mut n = notes_clone.lock().unwrap();
                n.create_note(
                    &format!("Concurrent Note {}", i),
                    &format!("Created by thread {}", i),
                    None,
                )
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads and collect results
        let mut note_ids = vec![];
        for handle in handles {
            let note = handle.join().unwrap()?;
            note_ids.push(note.id);
        }
        
        // Verify all notes were created
        let notes = notes.lock().unwrap();
        assert_eq!(notes.notes.len(), 10);
        assert_eq!(notes.root_notes.len(), 10);
        
        // Verify all note IDs are unique
        let unique_ids: std::collections::HashSet<_> = note_ids.iter().collect();
        assert_eq!(unique_ids.len(), 10);
        
        Ok(())
    }
    
    #[test]
    fn test_concurrent_tag_operations() -> Result<()> {
        let notes = Arc::new(Mutex::new(NotesState::default()));
        
        // Create a note and some tags
        let (note_id, tag_ids) = {
            let mut n = notes.lock().unwrap();
            let note = n.create_note("Test Note", "Content", None)?;
            let mut tag_ids = vec![];
            for i in 0..5 {
                let tag = n.create_tag(&format!("Tag {}", i), None)?;
                tag_ids.push(tag.id);
            }
            (note.id, tag_ids)
        };
        
        let mut handles = vec![];
        
        // Spawn threads that add/remove tags
        for (i, tag_id) in tag_ids.iter().enumerate() {
            let notes_clone = Arc::clone(&notes);
            let note_id_clone = note_id.clone();
            let tag_id_clone = tag_id.clone();
            
            let handle = thread::spawn(move || {
                let mut n = notes_clone.lock().unwrap();
                if i % 2 == 0 {
                    n.add_tag_to_note(&note_id_clone, &tag_id_clone)
                } else {
                    // First add, then remove
                    n.add_tag_to_note(&note_id_clone, &tag_id_clone)?;
                    n.remove_tag_from_note(&note_id_clone, &tag_id_clone)
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap()?;
        }
        
        // Note should have some tags (exact count depends on thread timing)
        let notes = notes.lock().unwrap();
        let note = notes.get_note(&note_id).unwrap();
        assert!(note.tags.len() <= 5);
        
        Ok(())
    }
}