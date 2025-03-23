// SPDX-License-Identifier: GNU GENERAL PUBLIC LICENSE Version 3
//
// Copyleft (c) 2024 James Wong. This file is part of James Wong.
// is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// James Wong is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with James Wong.  If not, see <https://www.gnu.org/licenses/>.
//
// IMPORTANT: Any software that fully or partially contains or uses materials
// covered by this license must also be released under the GNU GPL license.
// This includes modifications and derived works.

// Thanks based on: https://github.com/wl4g-collect/openobserve/blob/v0.10.9/src/config/src/utils/cgroup.rs
use sysinfo::SystemExt;

/// Get cpu limit by cgroup or return the node cpu cores
pub fn get_cpu_limit() -> usize {
    let mut cpu_num = read_cpu_cgroup_v1();
    if cpu_num == 0 {
        cpu_num = read_cpu_cgroup_v2();
    }
    if cpu_num > 0 {
        if cpu_num < 100000 {
            1 // maybe the limit less than 1 core
        } else {
            cpu_num / 100000
        }
    } else {
        let mut system = sysinfo::System::new();
        system.refresh_cpu();
        system.cpus().len()
    }
}

/// Get memory limit by cgroup or return the node memory size
pub fn get_memory_limit() -> usize {
    let mut mem_size = read_memory_cgroup_v1();
    if mem_size == 0 {
        mem_size = read_memory_cgroup_v2();
    }
    let node_mem_size = {
        let mut system = sysinfo::System::new();
        system.refresh_memory();
        system.total_memory() as usize
    };
    if mem_size == 0 || mem_size > node_mem_size {
        node_mem_size
    } else {
        mem_size
    }
}

fn read_cpu_cgroup_v1() -> usize {
    if let Ok(val) = std::fs::read_to_string("/sys/fs/cgroup/cpu.max") {
        if !val.is_empty() && !val.to_lowercase().starts_with("max") {
            let columns = val.split(' ').collect::<Vec<&str>>();
            return columns[0].parse::<usize>().unwrap_or_default();
        }
    }
    0
}

fn read_memory_cgroup_v1() -> usize {
    if let Ok(val) = std::fs::read_to_string("/sys/fs/cgroup/memory.max") {
        if !val.is_empty() && !val.to_lowercase().starts_with("max") {
            return val.trim_end().parse::<usize>().unwrap_or_default();
        }
    }
    0
}

/// Get cpu limit by cgroup v2: if there is no limit, default is: -1
fn read_cpu_cgroup_v2() -> usize {
    if let Ok(val) = std::fs::read_to_string("/sys/fs/cgroup/cpu/cpu.cfs_quota_us") {
        val.trim().to_string().parse::<usize>().unwrap_or_default()
    } else {
        0
    }
}

/// Get memory limit by cgroup v2: if there is no limit, default is:
/// 9223372036854775807
fn read_memory_cgroup_v2() -> usize {
    if let Ok(val) = std::fs::read_to_string("/sys/fs/cgroup/memory/memory.limit_in_bytes") {
        val.trim_end().parse::<usize>().unwrap_or_default()
    } else {
        0
    }
}

mod tests {
    #[allow(unused)]
    use super::*;
    #[test]
    fn test_get_cpu_limit() {
        println!("get cpu limit: {}", get_cpu_limit());
    }
    #[test]
    fn test_get_memory_limit() {
        println!("get memory limit: {}", get_memory_limit());
    }
}
