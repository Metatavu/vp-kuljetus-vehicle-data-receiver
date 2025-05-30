pub mod cache_handler;

use nom_teltonika::AVLRecord;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    fs::create_dir_all,
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

/// Base trait for all cacheable telematics data
pub trait Cacheable
where
    Self: Serialize + Sized + for<'a> Deserialize<'a> + Clone + Debug,
{
    /// File path to store the cache
    fn get_file_path() -> String;

    /// Converts a Teltonika record to a cacheable object
    /// This is only used for [TruckLocation]s at the moment, hence returning an Option.
    fn from_teltonika_record(_: &AVLRecord) -> Option<Self> {
        None
    }

    /// Gets the file handle for the cache file
    ///
    /// # Arguments
    /// * `base_cache_path` - The base path to the cache directory
    ///
    /// # Returns
    /// * A file handle to the cache file
    fn get_cache_file_handle(base_cache_path: PathBuf) -> std::fs::File {
        let cache_file_path = format!("{}/{}", base_cache_path.to_str().unwrap(), Self::get_file_path());
        create_dir_all(Path::new(&base_cache_path)).unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(Path::new(&cache_file_path))
            .unwrap()
    }

    /// Writes the cache to a file
    ///
    /// # Arguments
    /// * `base_cache_path` - The base path to the cache directory
    fn write_to_file(&self, base_cache_path: PathBuf) -> Result<(), std::io::Error> {
        let mut file = Self::get_cache_file_handle(base_cache_path.clone());
        let (mut existing_cache, _) = Self::read_from_file(base_cache_path, 0);
        existing_cache.push(self.clone());
        let json = serde_json::to_string(&existing_cache).unwrap();
        if let Err(_) = file.set_len(0) {
            panic!("Error truncating cache file!");
        };
        return file.write_all(json.as_bytes());
    }

    /// Writes a vector of cacheable objects to a file
    ///
    /// # Arguments
    /// * `cache` - The cacheable objects to write to the file
    /// * `base_cache_path` - The base path to the cache directory
    fn write_vec_to_file(cache: Vec<Self>, base_cache_path: PathBuf) -> Result<(), std::io::Error> {
        let mut file = Self::get_cache_file_handle(base_cache_path.clone());
        let (mut existing_cache, _) = Self::read_from_file(base_cache_path, 0);
        existing_cache.extend(cache);
        let json = serde_json::to_string(&existing_cache).unwrap();
        if let Err(_) = file.set_len(0) {
            panic!("Error truncating cache file!");
        };
        return file.write_all(json.as_bytes());
    }

    /// Takes a cache from the file and purges the cache file
    ///
    /// # Arguments
    /// * `base_cache_path` - The base path to the cache directory
    /// * `purge_cache_size` - The size of the cache to purge
    fn take_from_file(base_cache_path: PathBuf, purge_cache_size: usize) -> (Vec<Self>, usize) {
        let file = Self::get_cache_file_handle(base_cache_path.clone());
        let reader = BufReader::new(file);

        let full_content: Vec<Self> = serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new());
        let cache_size = full_content.len();

        // Treat 0 as no cache size limit
        if purge_cache_size == 0 {
            Self::clear_cache(base_cache_path.clone());
            return (full_content, cache_size);
        }
        let full_content_iterable = full_content.into_iter();
        let cache = full_content_iterable.clone().take(purge_cache_size).collect();

        let remaining_cache = full_content_iterable.skip(purge_cache_size).collect();
        Self::clear_cache(base_cache_path.clone());
        Self::write_vec_to_file(remaining_cache, base_cache_path).expect("Error caching locations");

        return (cache, cache_size);
    }

    /// Reads the cache from a file
    ///
    /// # Arguments
    /// * `base_cache_path` - The base path to the cache directory
    /// * `purge_cache_size` - The size of the cache to purge
    ///
    /// # Returns
    /// * A vector of cacheable objects
    fn read_from_file(base_cache_path: PathBuf, purge_cache_size: usize) -> (Vec<Self>, usize) {
        let file = Self::get_cache_file_handle(base_cache_path);
        let reader = BufReader::new(file);

        let full_content: Vec<Self> = serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new());
        let cache_size = full_content.len();

        // Treat 0 as no cache size limit
        if purge_cache_size == 0 {
            return (full_content, cache_size);
        }

        let cache = full_content.into_iter().take(purge_cache_size).collect();

        return (cache, cache_size);
    }

    /// Clears the cache file
    ///
    /// # Arguments
    /// * `base_cache_path` - The base path to the cache directory
    fn clear_cache(base_cache_path: PathBuf) {
        let file = Self::get_cache_file_handle(base_cache_path);
        if let Err(_) = file.set_len(0) {
            panic!("Error truncating cache file!");
        };
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::utils::{avl_record_builder::avl_record_builder::AVLRecordBuilder, test_utils::get_temp_dir_path};

    use super::Cacheable;

    impl Cacheable for HashMap<String, String> {
        fn get_file_path() -> String
        where
            Self: Sized,
        {
            String::from("hash_map_cache.json")
        }
    }

    #[test]
    fn test_cacheable_from_teltonika_record() {
        let record = AVLRecordBuilder::new().build();

        let cacheable = HashMap::from_teltonika_record(&record);
        assert_eq!(cacheable, None);
    }

    #[test]
    fn test_cacheable() {
        let temp_dir = get_temp_dir_path();
        let mut cacheable = HashMap::new();

        cacheable.insert("key".to_string(), "value".to_string());
        cacheable.write_to_file(temp_dir.clone()).unwrap();

        let (cache, cache_size) = HashMap::read_from_file(temp_dir.clone(), 0);
        assert_eq!(cache_size, 1);

        let cache = cache.into_iter().next().unwrap();
        assert_eq!(cache.get("key").unwrap(), "value");

        HashMap::clear_cache(temp_dir.clone());
        let (cache, _) = HashMap::read_from_file(temp_dir, 0);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_purge_cache_size() {
        let temp_dir = get_temp_dir_path();

        for i in 0..10 {
            let mut cacheable = HashMap::new();
            cacheable.insert(i.to_string(), i.to_string());
            cacheable.write_to_file(temp_dir.clone()).unwrap();
        }

        let (cache, cache_size) = HashMap::read_from_file(temp_dir.clone(), 5);
        assert_eq!(cache_size, 10);
        assert_eq!(cache.len(), 5);

        HashMap::clear_cache(temp_dir.clone());

        let (cache, cache_size) = HashMap::read_from_file(temp_dir.clone(), 0);
        assert_eq!(cache.len(), 0);
        assert_eq!(cache_size, 0);
    }
}
