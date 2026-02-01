use crate::page::{Page, PAGE_SIZE};
use crate::StoreError;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub struct Pager {
    file: File,
    total_pages: u32,
}

impl Pager {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let len = file.metadata()?.len();
        let total_pages = (len / PAGE_SIZE as u64) as u32;

        Ok(Self { file, total_pages })
    }

    /// Writes a raw page to disk (TODO: Add Encryption Hook Here)
    pub fn write_page(&mut self, page: &Page) -> Result<(), StoreError> {
        let offset = page.id as u64 * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;

        // Convert struct to raw bytes safely
        let bytes =
            unsafe { std::slice::from_raw_parts(page as *const Page as *const u8, PAGE_SIZE) };
        self.file.write_all(bytes)?;

        Ok(())
    }

    /// Reads a page from disk
    pub fn read_page(&mut self, id: u32) -> Result<Page, StoreError> {
        if id >= self.total_pages {
            return Err(StoreError::PageNotFound(id));
        }

        let offset = id as u64 * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;

        let mut page = Page::new(id);
        let bytes =
            unsafe { std::slice::from_raw_parts_mut(&mut page as *mut Page as *mut u8, PAGE_SIZE) };
        self.file.read_exact(bytes)?;

        Ok(page)
    }

    /// Allocates a new empty page
    pub fn allocate_page(&mut self) -> u32 {
        let id = self.total_pages;
        self.total_pages += 1;
        id
    }
}
