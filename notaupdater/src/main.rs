//! NotaUpdater — launched by Nota.exe to apply updates while the main app is closed.
//! Usage: NotaUpdater.exe --apply <zip_path> --pid <pid> [--restart <exe_path>]

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let zip_path = get_arg(&args, "--apply").expect("Missing --apply <zip>");
    let pid: u32  = get_arg(&args, "--pid")
        .expect("Missing --pid <pid>")
        .parse()
        .expect("PID must be a number");
    let restart = get_arg(&args, "--restart"); // optional: path to restart after update

    println!("NotaUpdater: waiting for PID {} to exit...", pid);
    wait_for_process(pid);
    println!("NotaUpdater: process exited, applying update from {}", zip_path);

    let install_dir = std::env::current_exe()
        .expect("Cannot find own location")
        .parent()
        .expect("No parent directory")
        .to_path_buf();

    apply_zip(&zip_path, &install_dir).expect("Failed to apply update");
    println!("NotaUpdater: update applied successfully");

    // Clean up the downloaded zip
    let _ = std::fs::remove_file(&zip_path);

    // Optionally restart the main app
    if let Some(exe) = restart {
        println!("NotaUpdater: restarting {}", exe);
        let _ = std::process::Command::new(&exe).spawn();
    }
}

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

fn apply_zip(zip_path: &str, dest_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let file    = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let out_path  = dest_dir.join(entry.name());

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out_file = std::fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
        }
    }
    Ok(())
}

// ── Wait for a process by PID ─────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn wait_for_process(pid: u32) {
    unsafe {
        use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
        use windows_sys::Win32::System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE};

        let handle: HANDLE = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
        if !handle.is_null() {
            WaitForSingleObject(handle, u32::MAX);
            CloseHandle(handle);
        }
    }
}

#[cfg(target_os = "linux")]
fn wait_for_process(pid: u32) {
    let proc_path = format!("/proc/{}", pid);
    while std::path::Path::new(&proc_path).exists() {
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn wait_for_process(_pid: u32) {
    std::thread::sleep(std::time::Duration::from_secs(3));
}