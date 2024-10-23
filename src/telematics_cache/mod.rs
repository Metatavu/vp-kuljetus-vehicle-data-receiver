use nom_teltonika::AVLRecord;
use serde::{Deserialize, Serialize};
use std::{
    fs::create_dir_all,
    io::{BufReader, Write},
    path::Path,
};

/// Base trait for all cacheable telematics data
pub trait Cacheable {
    fn get_file_path() -> String
    where
        Self: Sized;
    /// File path to store the cache

    fn from_teltonika_record(record: &AVLRecord) -> Option<Self>
    where
        Self: Sized;

    /// Gets the file handle for the cache file
    ///
    /// # Arguments
    /// * `base_cache_path` - The base path to the cache directory
    ///
    /// # Returns
    /// * A file handle to the cache file
    fn get_cache_file_handle(base_cache_path: &str) -> std::fs::File
    where
        Self: Sized,
    {
        let cache_file_path = format!("{}/{}", base_cache_path, Self::get_file_path());
        create_dir_all(Path::new(&base_cache_path)).unwrap();
        std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .open(Path::new(&cache_file_path))
            .unwrap()
    }

    /// Writes the cache to a file
    ///
    /// # Arguments
    /// * `base_cache_path` - The base path to the cache directory
    fn write_to_file(&self, base_cache_path: &str) -> Result<(), std::io::Error>
    where
        Self: Serialize + Sized + for<'a> Deserialize<'a> + Clone,
    {
        let mut file = Self::get_cache_file_handle(base_cache_path);
        let mut existing_cache = Self::read_from_file(base_cache_path);
        existing_cache.push(self.clone());
        let json = serde_json::to_string(&existing_cache).unwrap();
        if let Err(_) = file.set_len(0) {
            panic!("Error truncating cache file!");
        };
        return file.write_all(json.as_bytes());
    }

    /// Reads the cache from a file
    ///
    /// # Arguments
    /// * `base_cache_path` - The base path to the cache directory
    ///
    /// # Returns
    /// * A vector of cacheable objects
    fn read_from_file(base_cache_path: &str) -> Vec<Self>
    where
        Self: Sized + for<'a> Deserialize<'a>,
    {
        let file = Self::get_cache_file_handle(&base_cache_path);
        let reader = BufReader::new(file);

        return serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new());
    }

    /// Clears the cache file
    ///
    /// # Arguments
    /// * `base_cache_path` - The base path to the cache directory
    fn clear_cache(base_cache_path: &str)
    where
        Self: Sized,
    {
        let file = Self::get_cache_file_handle(base_cache_path);
        if let Err(_) = file.set_len(0) {
            panic!("Error truncating cache file!");
        };
    }
}
