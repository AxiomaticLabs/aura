use crate::index::PrimaryIndex;
use crate::page::{Page, PAGE_SIZE};
use crate::StoreError;
use aura_security::symmetric::{self, KEY_SIZE};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

// Encrypted page size = PAGE_SIZE + NONCE_SIZE + TAG_SIZE
pub const ENCRYPTED_PAGE_SIZE: usize = PAGE_SIZE + symmetric::NONCE_SIZE + symmetric::TAG_SIZE;

pub struct Pager {
    file: File,
    total_pages: u32,
    master_key: [u8; KEY_SIZE],

    // NEW: The Index lives here
    pub index: PrimaryIndex,
}

impl Pager {
    pub fn open(path: impl AsRef<Path>, master_key: [u8; KEY_SIZE]) -> Result<Self, StoreError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        let len = file.metadata()?.len();
        let total_pages = (len / ENCRYPTED_PAGE_SIZE as u64) as u32;

        // LOAD THE INDEX
        // Convention: Page 0 is ALWAYS the Index Page.
        let index = PrimaryIndex::new();

        if total_pages > 0 {
            // If DB exists, try to read Page 0
            // For now, we'll load it after creating the pager
            // This is a bit circular, but we'll handle it
        }

        let mut pager = Self {
            file,
            total_pages,
            master_key,
            index,
        };

        // Now try to load the index from page 0
        if pager.total_pages > 0 {
            match pager.read_page(0) {
                Ok(page) if page.page_type == 2 => {
                    // Index page type
                    let index_bytes = &page.data[..page.used_space as usize];
                    match PrimaryIndex::from_bytes(index_bytes) {
                        Ok(loaded_index) => {
                            pager.index = loaded_index;
                        }
                        Err(_) => {
                            // Index corruption, start fresh
                            pager.index = PrimaryIndex::new();
                        }
                    }
                }
                _ => {
                    // No index page or wrong type, start fresh
                }
            }
        }

        Ok(pager)
    }

    /// Writes a page to disk with transparent encryption
    pub fn write_page(&mut self, page: &Page) -> Result<(), StoreError> {
        let offset = page.id as u64 * ENCRYPTED_PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;

        // Convert struct to raw bytes safely
        let plaintext =
            unsafe { std::slice::from_raw_parts(page as *const Page as *const u8, PAGE_SIZE) };

        // Encrypt the page data
        let encrypted_data = symmetric::encrypt(plaintext, &self.master_key)
            .map_err(|_| StoreError::Tampered(page.id))?;

        // Write encrypted data to disk
        self.file.write_all(&encrypted_data)?;

        // Update total_pages if we wrote beyond the current end
        if page.id >= self.total_pages {
            self.total_pages = page.id + 1;
        }

        Ok(())
    }

    /// Reads a page from disk with transparent decryption
    pub fn read_page(&mut self, id: u32) -> Result<Page, StoreError> {
        if id >= self.total_pages {
            return Err(StoreError::PageNotFound(id));
        }

        let offset = id as u64 * ENCRYPTED_PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;

        // Read encrypted data from disk
        let mut encrypted_data = vec![0u8; ENCRYPTED_PAGE_SIZE];
        self.file.read_exact(&mut encrypted_data)?;

        // Decrypt the data
        let plaintext = symmetric::decrypt(&encrypted_data, &self.master_key)
            .map_err(|_| StoreError::Tampered(id))?;

        // Ensure decrypted data is exactly PAGE_SIZE
        if plaintext.len() != PAGE_SIZE {
            return Err(StoreError::Tampered(id));
        }

        // Convert back to Page struct safely
        let mut page: Page = unsafe { std::mem::zeroed() };
        unsafe {
            std::ptr::copy_nonoverlapping(
                plaintext.as_ptr(),
                &mut page as *mut Page as *mut u8,
                PAGE_SIZE,
            );
        }

        Ok(page)
    }

    /// Allocates a new empty page
    pub fn allocate_page(&mut self) -> u32 {
        // Page 0 is reserved for index, so start from page 1
        let id = if self.total_pages == 0 {
            1
        } else {
            self.total_pages
        };
        self.total_pages = id + 1;
        id
    }

    // NEW: Save the index to Page 0
    pub fn sync_index(&mut self) -> Result<(), StoreError> {
        if !self.index.dirty {
            return Ok(());
        }

        let bytes = self.index.to_bytes()?;

        // Ensure Page 0 exists
        if self.total_pages == 0 {
            self.total_pages = 1;
        }

        let mut page = Page::new(0); // Page 0 is reserved
        page.page_type = 2; // 2 = Index Type

        // Safety: If index > 4KB, this crashes.
        // FUTURE TODO: B-Tree splitting. For now, we assume small index.
        if bytes.len() > crate::page::DATA_SIZE {
            return Err(StoreError::Io(std::io::Error::other(
                "Index too big for Page 0",
            )));
        }

        page.data[..bytes.len()].copy_from_slice(&bytes);
        page.used_space = bytes.len() as u16;

        self.write_page(&page)?;
        self.index.dirty = false;
        Ok(())
    }
}
