use std::path::PathBuf;

extern crate fs2;

use crate::{config::MindmapConfig, files, utils};
use anyhow::Result;
use colored::Colorize;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

pub struct MindmapWatcher {
    watcher: RecommendedWatcher,
    rx: std::sync::mpsc::Receiver<notify::Result<Event>>,
    config: MindmapConfig,
}

impl MindmapWatcher {
    pub fn handle_create_event(
        &self,
        kind: notify::event::CreateKind,
        paths: Vec<PathBuf>,
    ) -> Result<()> {
        let path_str = paths.first().expect("Path should exist");
        let path = PathBuf::from(path_str);
        if kind == notify::event::CreateKind::File {
            files::recompute_file(&path, &self.config)?;
            println!("File created: {:?}", path);
        }
        Ok(())
    }

    pub fn handle_modify_event(
        &self,
        kind: notify::event::ModifyKind,
        paths: Vec<PathBuf>,
    ) -> Result<()> {
        let path_str = paths.first().expect("Path should exist");
        let path = PathBuf::from(path_str);
        match kind {
            notify::event::ModifyKind::Data(_data) => {
                files::recompute_file(&path, &self.config)?;
                println!("File modified: {:?}", path);
            }
            notify::event::ModifyKind::Name(_name) => {
                if path.exists() {
                    files::recompute_file(&path, &self.config)?;
                    println!("File renamed: {:?}", path);
                } else {
                    files::delete_file(&path, &self.config)?;
                    println!("File deleted: {:?}", path);
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn handle_remove_event(
        &self,
        kind: notify::event::RemoveKind,
        paths: Vec<PathBuf>,
    ) -> Result<()> {
        let path_str = paths.first().expect("Path should exist");
        let path = PathBuf::from(path_str);
        if kind == notify::event::RemoveKind::File {
            println!("File removed: {:?}", path);
            files::delete_file(&path, &self.config)?;
        }
        Ok(())
    }

    pub fn handle_event(&self, event: Event) -> Result<()> {
        match event.kind {
            EventKind::Create(kind) => self.handle_create_event(kind, event.paths),
            EventKind::Modify(kind) => self.handle_modify_event(kind, event.paths),
            EventKind::Remove(kind) => self.handle_remove_event(kind, event.paths),
            _ => Ok(()),
        }
    }

    pub fn new(config: MindmapConfig) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let watcher =
            RecommendedWatcher::new(tx, Config::default()).expect("Failed to create watcher");
        Self {
            watcher,
            rx,
            config,
        }
    }

    pub fn watch(&mut self) -> Result<()> {
        utils::acquire_lock(&self.config.watcher.lock_path)?;

        println!("{}", "Watching files...".blue());
        self.watcher
            .watch(&self.config.data_dir, RecursiveMode::Recursive)?;

        for res in &self.rx {
            match res {
                Ok(event) => {
                    let _ = &self.handle_event(event);
                }
                Err(error) => println!("Error: {:?}", error),
            }
        }

        Ok(())
    }
}
