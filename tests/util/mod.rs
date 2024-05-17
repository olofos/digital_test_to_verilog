use std::path::PathBuf;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
pub struct TempDir {
    pub path: PathBuf,
}

impl TempDir {
    pub fn create(basename: impl Into<String>) -> Self {
        let mut rng = thread_rng();
        let basename: String = basename.into();

        for _ in 0..16 {
            let mut name = basename.clone();
            name.push('_');
            name.extend((&mut rng).sample_iter(Alphanumeric).take(8).map(char::from));

            let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
            if std::fs::create_dir(&path).is_ok() {
                eprintln!("Created temp dir {path:?}");
                return Self { path };
            }
        }
        panic!("Could not create temp dir with basename {basename}")
    }

    pub fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }

    pub fn delete(self) {
        let _ = std::fs::remove_dir_all(self.path);
    }
}

#[test]
fn temp_dir_works() {
    let dir = TempDir::create("abc");

    let path = dir.path.clone();
    println!("Created {}", path.to_string_lossy());

    assert!(path
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .starts_with("abc_"));
    assert!(path.exists());
    assert!(path.is_dir());

    let filename = dir.file("foobar123");
    assert!(filename.to_string_lossy().contains("abc_"));
    assert!(filename.to_string_lossy().contains("foobar123"));

    dir.delete();

    assert!(!path.exists());
}
