// Copyright 2023-2025, [rust@localhost] $ (@github-handle)
// // //
// // // This file is part of LightScheduling.
// // //
// // // LightScheduling is free software: you can redistribute it and/or modify it under
// // // the terms of the GNU General Public License as published by the Free
// // // Software Foundation, either version 3 of the License, or (at your option)
// // // any later version.
// // //
// // // LightScheduling is distributed in the hope that it will be useful, but WITHOUT ANY
// // // WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// // // FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// // // details.
// // //
// // // You should have received a copy of the GNU General Public License along
// // // with LightScheduling. If not, see <https://www.gnu.org/licenses/>.

use libc::{cpu_set_t, sched_setaffinity};

use std::{error::Error, ffi::CStr, fs};

fn set_cpu_affinity(pid: i32, cpus: &[usize]) -> Result<(), Box<dyn Error>> {
    let mut set = unsafe { std::mem::zeroed::<cpu_set_t>() };
    unsafe { libc::CPU_ZERO(&mut set) };
    for &cpu in cpus {
        if cpu >= libc::CPU_SETSIZE as usize {
            return Err(format!(
                "CPU {} exceeds maximum allowed ({})",
                cpu,
                libc::CPU_SETSIZE
            )
            .into());
        }
        unsafe { libc::CPU_SET(cpu, &mut set) };
    }
    let res = unsafe { sched_setaffinity(pid, std::mem::size_of_val(&set), &set) };
    if res == -1 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}

#[derive(Debug)]
pub struct Migrate {
    pids: Vec<i32>,
}

impl Migrate {
    pub fn new(topapp: &str) -> Self {
        if let Ok(pid) = get_pids_by_process_name(topapp) {
            return Self { pids: pid };
        }
        Self { pids: [1].to_vec() }
    }
    pub fn setting(&self) -> Result<(), Box<dyn Error>> {
        let cpus = vec![0, 1, 2, 3, 4, 5, 6];
        for i in &self.pids {
            set_cpu_affinity(*i, &cpus)?;
        }
        Ok(())
    }
}

fn get_pids_by_process_name(target_name: &str) -> Result<Vec<i32>, std::io::Error> {
    let mut pids = Vec::new();

    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(pid_str) = path.file_name().and_then(|n| n.to_str()) {
                if let Ok(pid) = pid_str.parse::<i32>() {
                    let cmdline_path = path.join("cmdline");
                    if let Ok(cmdline) = fs::read(&cmdline_path) {
                        let process_name = cmdline
                            .split(|&c| c == b'\0')
                            .next()
                            .and_then(|bytes| CStr::from_bytes_until_nul(bytes).ok())
                            .and_then(|cstr| cstr.to_str().ok())
                            .map(|s| s.trim().to_string());
                        if let Some(name) = process_name {
                            if name == target_name {
                                pids.push(pid);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(pids)
}
