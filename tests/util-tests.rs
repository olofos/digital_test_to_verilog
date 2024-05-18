mod util;
use util::TempDir;

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
