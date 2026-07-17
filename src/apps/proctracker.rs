use alloc::boxed::Box;
use alloc::format;

use crate::kui::kdraw::text_length;
use crate::kui::kfont::KODEMONO_BOLD;
use crate::kui::{draw_rect, draw_text, draw_titled_window};
use crate::proc::{IpcData, IpcReceiveError, ProcessError, ProcessEvent, ProcessStatus};

pub struct ProcessTracker {
    pid: u32,
    name: &'static str,
    status: ProcessStatus,

    tick_count: u32,
    /// How often to refresh the on-screen table / log process-count changes.
    /// UI is also painted once in `on_init` so the first frame is immediate.
    report_every: u32,
    current_amount: u32,
    last_amount: u32,

    draw_to_screen: bool,
}

/// Full chrome + process rows + present (single frame).
fn paint_ui() -> u32 {
    let table = crate::proc::registry::PROCESS_TABLE.lock();
    draw_titled_window("Process Tracker");
    for (i, entry) in table.iter().enumerate() {
        let text = &format!("PID: {} | {} | {:?}", entry.pid, entry.name, entry.status);
        let row_y = 70 + i as u32 * 22;
        let tw = text_length(text, &KODEMONO_BOLD, 16.0) as u32;
        draw_rect(20, row_y, tw.saturating_add(8), 20, 0, 0);
        draw_text(20, row_y, 16.0, &KODEMONO_BOLD, text, 0x55EAD4);
    }
    let n = table.len() as u32;
    drop(table);
    crate::drivers::display::present();
    n
}

impl crate::proc::Process for ProcessTracker {
    fn new() -> Box<Self> {
        Box::new(Self {
            pid: 0,
            name: "ProcessTracker",
            status: ProcessStatus::Running,

            tick_count: 0,
            report_every: 1_000_000,
            current_amount: 0,
            last_amount: 0,

            draw_to_screen: true,
        })
    }

    fn on_init(&mut self) {
        if self.draw_to_screen {
            // Paint immediately (process is already in PROCESS_TABLE).
            self.current_amount = paint_ui();
            self.last_amount = self.current_amount;
        }
    }

    fn on_tick(&mut self) -> Result<ProcessEvent, ProcessError> {
        self.tick_count += 1;
        if !self.tick_count.is_multiple_of(self.report_every) {
            return Ok(ProcessEvent::Yielded);
        }
        crate::board::gpio::PE3.into_output().toggle();
        crate::klog::log(
            self.name,
            &format!("Ticks: {}", self.tick_count),
            crate::klog::MessageType::Info,
        );

        if self.current_amount < self.last_amount {
            crate::klog::log(
                self.name,
                &format!(
                    "One or more processes closed. {} active",
                    self.current_amount
                ),
                crate::klog::MessageType::Info,
            );
        }
        if self.current_amount > self.last_amount {
            crate::klog::log(
                self.name,
                &format!("Registered {} new process(es)", self.current_amount),
                crate::klog::MessageType::Info,
            );
        }

        if !self.draw_to_screen {
            let table = crate::proc::registry::PROCESS_TABLE.lock();
            crate::println!("--- {} processes alive ---", table.len());
            for entry in table.iter() {
                crate::println!(
                    "  pid {:>3}  {:<16} {:?}",
                    entry.pid,
                    entry.name,
                    entry.status
                );
            }
            self.current_amount = table.len() as u32;
        } else {
            self.current_amount = paint_ui();
        }
        self.last_amount = self.current_amount;
        Ok(ProcessEvent::Yielded)
    }

    fn on_uninit(self: Box<Self>) {}

    fn pid(&self) -> u32 {
        self.pid
    }
    fn name(&self) -> &'static str {
        self.name
    }
    fn set_pid(&mut self, pid: u32) {
        self.pid = pid;
    }
    fn set_name(&mut self, name: &'static str) {
        self.name = name;
    }
    fn status(&self) -> ProcessStatus {
        self.status
    }
    fn set_status(&mut self, status: ProcessStatus) {
        self.status = status;
    }

    fn receive(&mut self, _data: IpcData) -> Result<(), IpcReceiveError> {
        Err(IpcReceiveError::Message("Not expecting any data"))
    }
    fn bind(&mut self, _subscriber: u32) {}
}
