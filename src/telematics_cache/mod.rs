use std::{fs::read_to_string, io::{Read, Write}, path::Path};
use serde::{Deserialize, Serialize};

pub mod cacheable_truck_speed;

/// Trait for caching u16 Telematics values
///
/// Vehicle VIN is required to be known before telematics can be sent to Vehicle Management Service.
/// Therefore we need to cache the telematics values in file system until the VIN is known.
pub trait CacheableU16 {
  /// Create a new instance of the struct from a Teltonika event
  ///
  /// # Arguments
  /// * `value` - The value of the telematics event
  /// * `timestamp` - The timestamp of the telematics event
  ///
  /// # Returns
  /// * `Self` - The new instance of the struct
  fn from_teltonika_event(value: u16, timestamp: i64) -> Self;
}

pub trait Cacheable {
  /// The file path to store the cache of these telematics values
  const FILE_PATH: &'static str;

  /// Gets the file handle for the cache file
  ///
  /// # Arguments
  /// * `base_file_path` - The base path to the file
  ///
  /// # Returns
  /// * `std::fs::File` - The file handle
  fn get_file_handle(base_file_path: &str) -> std::fs::File {
    println!("file path: {:?}", Path::new(base_file_path).join(Self::FILE_PATH));
    std::fs::OpenOptions::new()
      .write(true)
      .append(false)
      .create(true)
      .open(Path::new(base_file_path).join(Self::FILE_PATH))
      .unwrap()
  }

  /// Reads the existing cache from the file and appends this instance to it.
  /// Then writes the updated cache back to the file.
  ///
  /// # Arguments
  /// * `file_path` - The path to the file to write to
  ///
  /// # Returns
  /// * `Result<(), std::io::Error>` - The result of writing to the file
  fn write_to_file(&self, file_path: String) -> Result<(), std::io::Error> where Self: Serialize + Sized + for<'a>Deserialize<'a> + Clone {
    let mut file = Self::get_file_handle(&file_path);
    let mut existing_cache = Self::read_from_file(file_path.clone());
    existing_cache.push(self.clone());
    let json = serde_json::to_string(&existing_cache).unwrap();
    file.write_all(json.as_bytes())
  }

  /// Reads the struct from a file
  ///
  /// # Arguments
  /// * `file_path` - The path to the file to read from
  ///
  /// # Returns
  /// * `Result<Vec<Self>, std::io::Error>` - The result of reading from the file
  fn read_from_file(file_path: String) -> Vec<Self> where Self: Sized + for<'a>Deserialize<'a> {
    let file  = Self::get_file_handle(&file_path);
    let reader = std::io::BufReader::new(file);

    return serde_json::from_reader(reader).unwrap();
    // let reader = std::io::BufReader::new(file);
    // let data: Vec<Self> = serde_json::from_reader(reader).unwrap();
    // Ok(data)
  }
}