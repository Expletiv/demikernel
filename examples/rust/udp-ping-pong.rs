// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//======================================================================================================================
// Imports
//======================================================================================================================

use ::anyhow::Result;
use ::demikernel::{demi_sgarray_t, runtime::types::demi_opcode_t, LibOS, LibOSName, QDesc};
use histogram::Histogram;
use ::std::{env, net::SocketAddr, slice, str::FromStr, time::{Duration, Instant}};

#[cfg(target_os = "windows")]
pub const AF_INET: i32 = windows::Win32::Networking::WinSock::AF_INET.0 as i32;

#[cfg(target_os = "windows")]
pub const SOCK_DGRAM: i32 = windows::Win32::Networking::WinSock::SOCK_DGRAM.0 as i32;

#[cfg(target_os = "linux")]
pub const AF_INET: i32 = libc::AF_INET;

#[cfg(target_os = "linux")]
pub const SOCK_DGRAM: i32 = libc::SOCK_DGRAM;

//======================================================================================================================
// Constants
//======================================================================================================================

const BUFSIZE_BYTES: usize = 64;
const NUM_PINGS: usize = 1_000_000;
const TIMEOUT_SECONDS: Duration = Duration::from_secs(60);
const RETRY_TIMEOUT_SECONDS: Duration = Duration::from_secs(1);
const LOG_INTERVAL_SECONDS: u64 = 5;

fn mksga(libos: &mut LibOS, size: usize, timestamp: u64) -> Result<demi_sgarray_t> {
    let sga = match libos.sgaalloc(size) {
        Ok(sga) => sga,
        Err(e) => anyhow::bail!("failed to allocate scatter-gather array: {:?}", e),
    };

    // Ensure that allocated array has the requested size.
    if sga.segments[0].data_len_bytes as usize != size {
        freesga(libos, sga);
        let seglen = sga.segments[0].data_len_bytes as usize;
        anyhow::bail!(
            "failed to allocate scatter-gather array: expected size={:?} allocated size={:?}",
            size,
            seglen
        );
    }

    // Fill in the array.
    let ptr = sga.segments[0].data_buf_ptr as *mut u8;
    let len = sga.segments[0].data_len_bytes as usize;
    let slice = unsafe { slice::from_raw_parts_mut(ptr, len) };

    // Write the timestamp into the first 8 bytes.
    let ts_bytes = timestamp.to_le_bytes();
    slice[0..8].copy_from_slice(&ts_bytes);

    // Zero out the remaining payload space.
    slice[8..].fill(0);

    Ok(sga)
}

fn freesga(libos: &mut LibOS, sga: demi_sgarray_t) {
    if let Err(e) = libos.sgafree(sga) {
        println!("ERROR: sgafree() failed (error={:?})", e);
        println!("WARN: leaking sga");
    }
}

fn close(libos: &mut LibOS, sockqd: QDesc) {
    if let Err(e) = libos.close(sockqd) {
        println!("ERROR: close() failed (error={:?})", e);
        println!("WARN: leaking sockqd={:?}", sockqd);
    }
}

fn issue_pushto(libos: &mut LibOS, sockqd: QDesc, remote_addr: SocketAddr, sga: &demi_sgarray_t) -> Result<()> {
    let qt = match libos.pushto(sockqd, sga, remote_addr) {
        Ok(qt) => qt,
        Err(e) => anyhow::bail!("push failed: {:?}", e),
    };

    match libos.wait(qt, Some(TIMEOUT_SECONDS)) {
        Ok(qr) if qr.qr_opcode == demi_opcode_t::DEMI_OPC_PUSH => (),
        Ok(_) => anyhow::bail!("unexpected result"),
        Err(e) => anyhow::bail!("operation failed: {:?}", e),
    };
    Ok(())
}

pub struct UdpServer {
    libos: LibOS,
    sockqd: QDesc,
}

impl UdpServer {
    pub fn new(mut libos: LibOS) -> Result<Self> {
        let sockqd = match libos.socket(AF_INET, SOCK_DGRAM, 0) {
            Ok(sockqd) => sockqd,
            Err(e) => anyhow::bail!("failed to create socket: {:?}", e),
        };
        return Ok(Self { libos, sockqd });
    }

    pub fn run(&mut self, local_addr: SocketAddr, remote_addr: SocketAddr) -> Result<()> {
        if let Err(e) = self.libos.bind(self.sockqd, local_addr) {
            anyhow::bail!("bind failed: {:?}", e)
        };

        let mut received_responses = 0;
        loop {
            let qt = match self.libos.pop(self.sockqd, None) {
                Ok(qt) => qt,
                Err(e) => anyhow::bail!("pop failed: {:?}", e),
            };

            let sga = match self.libos.wait(qt, Some(TIMEOUT_SECONDS)) {
                Ok(qr) if qr.qr_opcode == demi_opcode_t::DEMI_OPC_POP => unsafe { qr.qr_value.sga },
                Ok(_) => anyhow::bail!("unexpected result"),
                // If we haven't received a message in the last 60 seconds, we can assume that the client is done.
                Err(e) if e.errno == libc::ETIMEDOUT => break,
                Err(e) => anyhow::bail!("operation failed: {:?}", e),
            };

            issue_pushto(&mut self.libos, self.sockqd, remote_addr, &sga)?;
            self.libos.sgafree(sga)?;
            received_responses += 1;
            println!("pong {:?}", received_responses);
        }

        Ok(())
    }
}

impl Drop for UdpServer {
    fn drop(&mut self) {
        close(&mut self.libos, self.sockqd);
    }
}

pub struct UdpClient {
    libos: LibOS,
    sockqd: QDesc,
}

impl UdpClient {
    pub fn new(mut libos: LibOS) -> Result<Self> {
        let sockqd = match libos.socket(AF_INET, SOCK_DGRAM, 0) {
            Ok(sockqd) => sockqd,
            Err(e) => anyhow::bail!("failed to create socket: {:?}", e),
        };

        return Ok(Self { libos, sockqd });
    }

    pub fn run(
        &mut self,
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
        bufsize_bytes: usize,
        num_pings: usize,
    ) -> Result<()> {
        if let Err(e) = self.libos.bind(self.sockqd, local_addr) {
            anyhow::bail!("bind failed: {:?}", e)
        };

        let mut total_received = 0;
        let mut interval_received = 0;

        let start_time = Instant::now();
        let mut last_log_time = Instant::now();
        let mut stats = Histogram::new(7, 64)?;

        println!("HEADERS:tx,rx,rps,p50,p90,p99,p99.9,p99.99,p99.999,p99.9999,p100");

        while total_received < num_pings {
            // Encode the timestamp
            let timestamp = Instant::now().duration_since(start_time).as_nanos() as u64;
            let sga = mksga(&mut self.libos, bufsize_bytes, timestamp)?;

            // Send packet and wait for response.
            let returned_sga = loop {
                issue_pushto(&mut self.libos, self.sockqd, remote_addr, &sga)?;

                // Wait for the response.
                let qt = match self.libos.pop(self.sockqd, None) {
                    Ok(qt) => qt,
                    Err(e) => anyhow::bail!("pop failed: {:?}", e),
                };

                match self.libos.wait(qt, Some(RETRY_TIMEOUT_SECONDS)) {
                    Ok(qr) if qr.qr_opcode == demi_opcode_t::DEMI_OPC_POP => break unsafe { qr.qr_value.sga },
                    Ok(_) => anyhow::bail!("unexpected result"),
                    Err(e) if e.errno == libc::ETIMEDOUT => continue,
                    Err(e) => anyhow::bail!("operation failed: {:?}", e),
                };
            };

            // Free the sent sga.
            self.libos.sgafree(sga)?;

            // Decode the timestamp
            let ptr = returned_sga.segments[0].data_buf_ptr as *const u8;
            let slice = unsafe { slice::from_raw_parts(ptr, 8) };
            let mut ts_bytes = [0u8; 8];
            ts_bytes.copy_from_slice(slice);
            let echoed_timestamp = u64::from_le_bytes(ts_bytes);

            // Log elapsed time
            let now = Instant::now().duration_since(start_time).as_nanos() as u64;
            let elapsed = now - echoed_timestamp;
            stats.increment(elapsed).unwrap_or(());

            // Free returned sga.
            self.libos.sgafree(returned_sga)?;
            total_received += 1;
            interval_received += 1;

            // Dump statistics periodically
            if last_log_time.elapsed() >= Duration::from_secs(LOG_INTERVAL_SECONDS) {
                let time_elapsed = last_log_time.elapsed().as_secs_f64();
                let rps = interval_received as f64 / time_elapsed;

                println!(
                    "METRICS:{:?},{:?},{:.1},{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?}",
                    total_received,
                    total_received,
                    rps,
                    stats.percentile(50.0)?.unwrap().start(),
                    stats.percentile(90.0)?.unwrap().start(),
                    stats.percentile(99.0)?.unwrap().start(),
                    stats.percentile(99.9)?.unwrap().start(),
                    stats.percentile(99.99)?.unwrap().start(),
                    stats.percentile(99.999)?.unwrap().start(),
                    stats.percentile(99.9999)?.unwrap().start(),
                    stats.percentile(100.0)?.unwrap().start(),
                );

                last_log_time = Instant::now();
                interval_received = 0;
            }
        }

        Ok(())
    }
}

impl Drop for UdpClient {
    fn drop(&mut self) {
        close(&mut self.libos, self.sockqd);
    }
}

fn usage(program_name: &String) {
    println!("Usage: {} MODE local remote\n", program_name);
    println!("Modes:\n");
    println!("  --client    Run program in client mode.");
    println!("  --server    Run program in server mode.");
}

pub fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 4 {
        let libos_name = match LibOSName::from_env() {
            Ok(libos_name) => libos_name.into(),
            Err(e) => anyhow::bail!("{:?}", e),
        };
        let libos = match LibOS::new(libos_name, None) {
            Ok(libos) => libos,
            Err(e) => anyhow::bail!("failed to initialize libos: {:?}", e),
        };

        let local_addr = SocketAddr::from_str(&args[2])?;
        let remote_addr = SocketAddr::from_str(&args[3])?;

        if args[1] == "--server" {
            let mut server = UdpServer::new(libos)?;
            return server.run(local_addr, remote_addr);
        } else if args[1] == "--client" {
            let mut client = UdpClient::new(libos)?;
            return client.run(local_addr, remote_addr, BUFSIZE_BYTES, NUM_PINGS);
        }
    }

    usage(&args[0]);

    Ok(())
}