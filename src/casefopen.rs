use std::fs::File;
use std::path::Path;

pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
  let path_ref = path.as_ref();
  let result = std::fs::File::open(path_ref);
  if let Ok(r) = result {
    return Ok(r);
  }

  let parent = match path_ref.parent() {
    Some(p) => p,
    _ => return result,
  };
  let directory = match std::fs::read_dir(parent) {
    Ok(d) => d,
    _ => return result,
  };

  let lower_name = path_ref.file_name().unwrap().to_str().unwrap().to_lowercase();
  for entry in directory {
    let name = match entry {
      Ok(n) => n,
      _ => continue,
    }
    .path();
    if name.is_dir() {
      continue;
    }
    let file_name = name.file_name().unwrap().to_str().unwrap().to_lowercase();
    if file_name == lower_name {
      println!("Found mismatching case");
      return std::fs::File::open(name);
    }
  }
  return result;
}
