use anyhow::Result;
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use sysinfo::{Pid, System};

#[derive(Debug, Clone)]
pub struct PortProcess {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub protocol: String,
    pub state: String,
}

pub fn get_listening_ports() -> Result<Vec<PortProcess>> {
    let sockets = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP | ProtocolFlags::UDP,
    )?;

    let mut sys = System::new_all();
    sys.refresh_all();

    let mut entries: Vec<PortProcess> = Vec::new();
    let mut seen_ports = std::collections::HashSet::new();

    for si in sockets {
        let (port, protocol, state) = match &si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => {
                if tcp.state != TcpState::Listen {
                    continue;
                }
                (
                    tcp.local_port,
                    "TCP".to_string(),
                    "LISTEN".to_string(),
                )
            }
            ProtocolSocketInfo::Udp(udp) => (udp.local_port, "UDP".to_string(), "LISTEN".to_string()),
        };

        if seen_ports.contains(&port) {
            continue;
        }
        seen_ports.insert(port);

        let pid = si.associated_pids.first().copied().unwrap_or(0);
        let process_name = if pid > 0 {
            sys.process(Pid::from(pid as usize))
                .map(|p| p.name().to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            "unknown".to_string()
        };

        entries.push(PortProcess {
            port,
            pid,
            process_name,
            protocol,
            state,
        });
    }

    entries.sort_by_key(|e| e.port);
    Ok(entries)
}

pub fn kill_process(pid: u32) -> bool {
    let mut sys = System::new_all();
    sys.refresh_all();
    if let Some(process) = sys.process(Pid::from(pid as usize)) {
        process.kill()
    } else {
        false
    }
}
