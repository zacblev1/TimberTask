use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Tag for categorizing notes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Tag {
    /// Unique ID of the tag
    pub id: String,
    /// Name of the tag
    pub name: String,
    /// Optional color for the tag (hex code)
    pub color: Option<String>,
}

/// Note model for the notes system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Unique ID of the note
    pub id: String,
    /// Title of the note
    pub title: String,
    /// Content of the note
    pub content: String,
    /// Parent note ID (None if root level)
    pub parent_id: Option<String>,
    /// IDs of child notes
    pub children: Vec<String>,
    /// IDs of tags associated with this note
    pub tags: HashSet<String>,
    /// Whether the note is expanded in the UI
    pub expanded: bool,
    /// Timestamp when the note was created
    pub created_at: u64,
    /// Timestamp when the note was last updated
    pub updated_at: u64,
}

/// Notes state data for serialization
#[derive(Serialize, Deserialize)]
struct NotesStateData {
    /// Map of note IDs to notes
    notes: HashMap<String, Note>,
    /// Map of tag IDs to tags
    tags: HashMap<String, Tag>,
    /// IDs of root-level notes (no parent)
    root_notes: Vec<String>,
    /// ID of the currently selected note
    selected_note_id: Option<String>,
}

/// Notes system state
pub struct NotesState {
    /// Map of note IDs to notes
    pub notes: HashMap<String, Note>,
    /// Map of tag IDs to tags
    pub tags: HashMap<String, Tag>,
    /// IDs of root-level notes (no parent)
    pub root_notes: Vec<String>,
    /// Path to the data file
    pub data_file_path: PathBuf,
    /// ID of the currently selected note
    pub selected_note_id: Option<String>,
}

impl Default for NotesState {
    fn default() -> Self {
        // Get application data directory
        let app_data_dir = home::home_dir()
            .expect("Failed to get home directory")
            .join(".timber-task");
        let data_file_path = app_data_dir.join("notes_data.json");
        
        Self {
            notes: HashMap::new(),
            tags: HashMap::new(),
            root_notes: Vec::new(),
            data_file_path,
            selected_note_id: None,
        }
    }
}

impl NotesState {
    /// Save the notes state to disk
    pub fn save_to_disk(&self) -> Result<()> {
        let data = NotesStateData {
            notes: self.notes.clone(),
            tags: self.tags.clone(),
            root_notes: self.root_notes.clone(),
            selected_note_id: self.selected_note_id.clone(),
        };
        
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| anyhow!("Failed to serialize notes state: {}", e))?;
        
        fs::create_dir_all(self.data_file_path.parent().unwrap())
            .map_err(|e| anyhow!("Failed to create data directory: {}", e))?;
            
        fs::write(&self.data_file_path, json)
            .map_err(|e| anyhow!("Failed to write notes state to disk: {}", e))?;
            
        Ok(())
    }
    
    /// Load the notes state from disk
    pub fn load_from_disk(&mut self) -> Result<()> {
        if !self.data_file_path.exists() {
            // No file yet, create an empty state
            return Ok(());
        }
        
        let json = fs::read_to_string(&self.data_file_path)
            .map_err(|e| anyhow!("Failed to read notes state from disk: {}", e))?;
            
        let data: NotesStateData = serde_json::from_str(&json)
            .map_err(|e| anyhow!("Failed to deserialize notes state: {}", e))?;
            
        self.notes = data.notes;
        self.tags = data.tags;
        self.root_notes = data.root_notes;
        self.selected_note_id = data.selected_note_id;
        
        Ok(())
    }
    
    /// Create a new tag
    pub fn create_tag(&mut self, name: &str, color: Option<&str>) -> Result<Tag> {
        // Generate a unique ID for the tag
        let id = Uuid::new_v4().to_string();
        
        let tag = Tag {
            id: id.clone(),
            name: name.to_string(),
            color: color.map(String::from),
        };
        
        self.tags.insert(id.clone(), tag.clone());
        
        // Save changes to disk
        self.save_to_disk()?;
        
        Ok(tag)
    }
    
    /// Get a tag by ID
    pub fn get_tag(&self, tag_id: &str) -> Option<&Tag> {
        self.tags.get(tag_id)
    }
    
    /// Update a tag
    #[allow(dead_code)]
    pub fn update_tag(&mut self, tag_id: &str, name: &str, color: Option<&str>) -> Result<Tag> {
        {
            let tag = self.tags.get_mut(tag_id)
                .ok_or_else(|| anyhow!("Tag not found"))?;
            
            tag.name = name.to_string();
            tag.color = color.map(String::from);
        }
        
        // Save changes to disk
        self.save_to_disk()?;
        
        // Return the updated tag
        Ok(self.tags.get(tag_id).unwrap().clone())
    }
    
    /// Delete a tag
    pub fn delete_tag(&mut self, tag_id: &str) -> Result<()> {
        // Remove tag from all notes
        for note in self.notes.values_mut() {
            note.tags.remove(tag_id);
        }
        
        // Remove tag from tags map
        self.tags.remove(tag_id)
            .ok_or_else(|| anyhow!("Tag not found"))?;
        
        // Save changes to disk
        self.save_to_disk()?;
        
        Ok(())
    }
    
    /// Create a new note
    pub fn create_note(&mut self, title: &str, content: &str, parent_id: Option<&str>) -> Result<Note> {
        // Generate a unique ID for the note
        let id = Uuid::new_v4().to_string();
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let note = Note {
            id: id.clone(),
            title: title.to_string(),
            content: content.to_string(),
            parent_id: parent_id.map(String::from),
            children: Vec::new(),
            tags: HashSet::new(),
            expanded: true,
            created_at: timestamp,
            updated_at: timestamp,
        };
        
        // If this note has a parent, add it to the parent's children
        if let Some(parent_id) = parent_id {
            let parent = self.notes.get_mut(parent_id)
                .ok_or_else(|| anyhow!("Parent note not found"))?;
            
            parent.children.push(id.clone());
            parent.updated_at = timestamp;
        } else {
            // If no parent, add to root notes
            self.root_notes.push(id.clone());
        }
        
        self.notes.insert(id.clone(), note.clone());
        
        // If no note is selected yet, select this one
        if self.selected_note_id.is_none() {
            self.selected_note_id = Some(id.clone());
        }
        
        // Save changes to disk
        self.save_to_disk()?;
        
        Ok(note)
    }
    
    /// Get a note by ID
    pub fn get_note(&self, note_id: &str) -> Option<&Note> {
        self.notes.get(note_id)
    }
    
    /// Update a note's content
    pub fn update_note(&mut self, note_id: &str, title: &str, content: &str) -> Result<Note> {
        {
            let note = self.notes.get_mut(note_id)
                .ok_or_else(|| anyhow!("Note not found"))?;
            
            note.title = title.to_string();
            note.content = content.to_string();
            note.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        // Save changes to disk
        self.save_to_disk()?;
        
        // Return the updated note
        Ok(self.notes.get(note_id).unwrap().clone())
    }
    
    /// Move a note to a new parent
    #[allow(dead_code)]
    pub fn move_note(&mut self, note_id: &str, new_parent_id: Option<&str>) -> Result<Note> {
        // Get the note to move (clone needed data to avoid borrow issues)
        let old_parent_id;
        
        {
            let note = self.notes.get(note_id)
                .ok_or_else(|| anyhow!("Note not found"))?;
            
            // Verify this wouldn't create a cycle (can't move a note to its own descendant)
            if let Some(new_parent_id) = new_parent_id {
                if note_id == new_parent_id {
                    return Err(anyhow!("Cannot move a note to itself"));
                }
                
                let mut ancestor_id = new_parent_id;
                while let Some(ancestor) = self.notes.get(ancestor_id) {
                    if ancestor.id == note_id {
                        return Err(anyhow!("Cannot move a note to its own descendant"));
                    }
                    
                    if let Some(parent_id) = &ancestor.parent_id {
                        ancestor_id = parent_id;
                    } else {
                        break;
                    }
                }
            }
            
            // Clone current parent ID
            old_parent_id = note.parent_id.clone();
        }
        
        // Remove from old parent's children
        if let Some(ref old_parent_id) = old_parent_id {
            if let Some(old_parent) = self.notes.get_mut(old_parent_id) {
                old_parent.children.retain(|id| id != note_id);
            }
        } else {
            // Was a root note, remove from root_notes
            self.root_notes.retain(|id| id != note_id);
        }
        
        // Update the note's parent
        {
            let note = self.notes.get_mut(note_id).unwrap();
            note.parent_id = new_parent_id.map(String::from);
            note.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        // Add to new parent's children
        if let Some(new_parent_id) = new_parent_id {
            let new_parent = self.notes.get_mut(new_parent_id)
                .ok_or_else(|| anyhow!("New parent note not found"))?;
            
            new_parent.children.push(note_id.to_string());
        } else {
            // Moving to root
            self.root_notes.push(note_id.to_string());
        }
        
        // Save changes to disk
        self.save_to_disk()?;
        
        // Return the updated note
        Ok(self.notes.get(note_id).unwrap().clone())
    }
    
    /// Delete a note and all its children
    pub fn delete_note(&mut self, note_id: &str) -> Result<()> {
        // Get the note to delete and clone the necessary data
        let children;
        let parent_id;
        
        {
            let note = self.notes.get(note_id)
                .ok_or_else(|| anyhow!("Note not found"))?;
            
            // Clone children and parent info
            children = note.children.clone();
            parent_id = note.parent_id.clone();
        }
        
        // Recursively delete all children
        for child_id in children {
            self.delete_note(&child_id)?;
        }
        
        // Remove from parent's children
        if let Some(ref parent_id) = parent_id {
            if let Some(parent) = self.notes.get_mut(parent_id) {
                parent.children.retain(|id| id != note_id);
            }
        } else {
            // Was a root note, remove from root_notes
            self.root_notes.retain(|id| id != note_id);
        }
        
        // Remove note from notes map
        self.notes.remove(note_id);
        
        // If this was the selected note, select parent or clear selection
        if self.selected_note_id.as_deref() == Some(note_id) {
            self.selected_note_id = parent_id;
        }
        
        // Save changes to disk
        self.save_to_disk()?;
        
        Ok(())
    }
    
    /// Toggle a note's expanded state
    pub fn toggle_note_expanded(&mut self, note_id: &str) -> Result<Note> {
        {
            let note = self.notes.get_mut(note_id)
                .ok_or_else(|| anyhow!("Note not found"))?;
            
            note.expanded = !note.expanded;
        }
        
        // Save changes to disk
        self.save_to_disk()?;
        
        // Return the updated note
        Ok(self.notes.get(note_id).unwrap().clone())
    }
    
    /// Add a tag to a note
    #[allow(dead_code)]
    pub fn add_tag_to_note(&mut self, note_id: &str, tag_id: &str) -> Result<Note> {
        // Verify tag exists
        if !self.tags.contains_key(tag_id) {
            return Err(anyhow!("Tag not found"));
        }
        
        {
            let note = self.notes.get_mut(note_id)
                .ok_or_else(|| anyhow!("Note not found"))?;
            
            note.tags.insert(tag_id.to_string());
            note.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        // Save changes to disk
        self.save_to_disk()?;
        
        // Return the updated note
        Ok(self.notes.get(note_id).unwrap().clone())
    }
    
    /// Remove a tag from a note
    #[allow(dead_code)]
    pub fn remove_tag_from_note(&mut self, note_id: &str, tag_id: &str) -> Result<Note> {
        {
            let note = self.notes.get_mut(note_id)
                .ok_or_else(|| anyhow!("Note not found"))?;
            
            note.tags.remove(tag_id);
            note.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        // Save changes to disk
        self.save_to_disk()?;
        
        // Return the updated note
        Ok(self.notes.get(note_id).unwrap().clone())
    }
    
    /// Get all notes with a specific tag
    pub fn get_notes_with_tag(&self, tag_id: &str) -> Vec<&Note> {
        self.notes.values()
            .filter(|note| note.tags.contains(tag_id))
            .collect()
    }
    
    /// Get all root notes
    pub fn get_root_notes(&self) -> Vec<&Note> {
        self.root_notes.iter()
            .filter_map(|id| self.notes.get(id))
            .collect()
    }
    
    /// Get all child notes of a parent
    pub fn get_child_notes(&self, parent_id: &str) -> Vec<&Note> {
        if let Some(parent) = self.notes.get(parent_id) {
            parent.children.iter()
                .filter_map(|id| self.notes.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Select a note
    pub fn select_note(&mut self, note_id: &str) -> Result<()> {
        if self.notes.contains_key(note_id) {
            self.selected_note_id = Some(note_id.to_string());
            self.save_to_disk()?;
            Ok(())
        } else {
            Err(anyhow!("Note not found"))
        }
    }
    
    /// Clear note selection
    #[allow(dead_code)]
    pub fn clear_selection(&mut self) -> Result<()> {
        self.selected_note_id = None;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get the currently selected note
    pub fn get_selected_note(&self) -> Option<&Note> {
        self.selected_note_id.as_ref().and_then(|id| self.notes.get(id))
    }
    
    /// Get the path of a note (ancestors from root to the note)
    #[allow(dead_code)]
    pub fn get_note_path(&self, note_id: &str) -> Vec<&Note> {
        let mut path = Vec::new();
        let mut current_id = note_id;
        
        while let Some(note) = self.notes.get(current_id) {
            path.push(note);
            
            if let Some(parent_id) = &note.parent_id {
                current_id = parent_id;
            } else {
                break;
            }
        }
        
        path.reverse();
        path
    }
    
    /// Search notes by title or content
    pub fn search_notes(&self, query: &str) -> Vec<&Note> {
        let query = query.to_lowercase();
        
        self.notes.values()
            .filter(|note| {
                note.title.to_lowercase().contains(&query) || 
                note.content.to_lowercase().contains(&query)
            })
            .collect()
    }
}