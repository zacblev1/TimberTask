use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Truncate text to fit within a maximum width, adding ellipsis if needed
pub fn truncate_text(text: &str, max_width: usize) -> String {
    if text.width() <= max_width {
        text.to_string()
    } else {
        // Truncate the string while respecting Unicode character boundaries
        let mut truncated = String::new();
        let mut current_width = 0;
        
        for c in text.chars() {
            let char_width = c.width().unwrap_or(1);
            
            // Check if adding this character would exceed the max width (leaving room for ellipsis)
            if current_width + char_width > max_width - 3 {
                break;
            }
            
            truncated.push(c);
            current_width += char_width;
        }
        
        // Add ellipsis
        truncated + "..."
    }
}