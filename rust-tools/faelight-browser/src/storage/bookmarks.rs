//! Bookmark storage - flat JSON file
//! ~/.local/share/faelight-browser/bookmarks.json

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub name: String,
    pub url: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

pub struct BookmarkStore {
    path: PathBuf,
    bookmarks: Vec<Bookmark>,
}

impl BookmarkStore {
    pub fn new() -> Result<Self, std::io::Error> {
        let path = Self::get_path()?;
        
        // Create directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Load existing bookmarks or create empty
        let bookmarks = if path.exists() {
            let content = fs::read_to_string(&path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };
        
        Ok(Self { path, bookmarks })
    }
    
    fn get_path() -> Result<PathBuf, std::io::Error> {
        let home = std::env::var("HOME")
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME not set"))?;
        Ok(PathBuf::from(home)
            .join(".local/share/faelight-browser")
            .join("bookmarks.json"))
    }
    
    pub fn add(&mut self, name: String, url: String, tags: Vec<String>) -> Result<(), std::io::Error> {
        let bookmark = Bookmark {
            name,
            url,
            tags,
            created_at: chrono::Local::now().to_rfc3339(),
        };
        
        self.bookmarks.push(bookmark);
        self.save()
    }
    
    pub fn list(&self) -> &[Bookmark] {
        &self.bookmarks
    }
    
    pub fn remove(&mut self, url: &str) -> Result<(), std::io::Error> {
        self.bookmarks.retain(|b| b.url != url);
        self.save()
    }
    
    fn save(&self) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self.bookmarks)?;
        fs::write(&self.path, json)
    }
}

impl Default for BookmarkStore {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            path: PathBuf::new(),
            bookmarks: Vec::new(),
        })
    }
}
