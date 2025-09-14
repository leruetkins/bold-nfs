use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::thread;
use std::fs;
use std::io::Write;

fn main() {
    // Create a temporary directory to serve
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let temp_path = temp_dir.path().to_path_buf();
    
    // Create some test files and directories
    let test_dir = temp_path.join("test_dir");
    fs::create_dir(&test_dir).expect("Failed to create test directory");
    
    let test_file = temp_path.join("test_file.txt");
    let mut file = fs::File::create(&test_file).expect("Failed to create test file");
    file.write_all(b"This is a test file").expect("Failed to write to test file");
    
    let nested_file = test_dir.join("nested_file.txt");
    let mut file = fs::File::create(&nested_file).expect("Failed to create nested file");
    file.write_all(b"This is a nested file").expect("Failed to write to nested file");
    
    println!("Created test files in: {:?}", temp_path);
    
    // Start the NFS server with the temporary directory
    let mut child = Command::new("cargo")
        .args(&["run", "-p", "bold-mem", "--bin", "bold-real-fs", "--", temp_path.to_str().unwrap()])
        .spawn()
        .expect("Failed to start NFS server");
    
    // Give the server time to start
    thread::sleep(Duration::from_secs(2));
    
    // Check that the process is still running
    if let Some(status) = child.try_wait().expect("Failed to check process status") {
        panic!("NFS server exited unexpectedly with status: {}", status);
    }
    
    println!("NFS server started successfully");
    
    // Here you would normally mount the NFS share and interact with it
    // For this example, we'll just stop the server
    
    // Stop the NFS server
    child.kill().expect("Failed to kill NFS server");
    child.wait().expect("Failed to wait for NFS server to exit");
    
    println!("NFS server stopped");
    
    // Clean up is automatic when temp_dir goes out of scope
}