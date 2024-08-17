use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::{self, Write},
};

use clap::Parser;
use libafl::events::{EventConfig, Launcher, MonitorTypedEventManager};
use libafl::{
    monitors::{tui::TuiMonitor, Monitor, MultiMonitor},
    Error,
};

use libafl_bolts::current_time;

use libafl_bolts::shmem::{ShMemProvider, StdShMemProvider};
#[cfg(unix)]
use {
    nix::unistd::dup,
    std::os::unix::io::{AsRawFd, FromRawFd},
};

use crate::{client::Client, options::FuzzerOptions};

pub struct Fuzzer {
    options: FuzzerOptions,
}

impl Fuzzer {
    pub fn new() -> Fuzzer {
        let options = FuzzerOptions::parse();
        options.validate();
        Fuzzer { options }
    }

    pub fn fuzz(&self) -> Result<(), Error> {
        if self.options.tui {
            let monitor = TuiMonitor::builder()
                .title("QEMU Launcher")
                .version("0.13.1")
                .enhanced_graphics(true)
                .build();
            self.launch(monitor)
        } else {
            let log = self.options.log.as_ref().and_then(|l| {
                OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(l)
                    .ok()
                    .map(RefCell::new)
            });

            #[cfg(unix)]
            let stdout_cpy = RefCell::new(unsafe {
                let new_fd = dup(io::stdout().as_raw_fd())?;
                File::from_raw_fd(new_fd)
            });

            // The stats reporter for the broker
            let monitor = MultiMonitor::new(|s| {
                #[cfg(unix)]
                writeln!(stdout_cpy.borrow_mut(), "{s}").unwrap();
                #[cfg(windows)]
                println!("{s}");

                if let Some(log) = &log {
                    writeln!(log.borrow_mut(), "{:?} {}", current_time(), s).unwrap();
                }
            });
            self.launch(monitor)
        }
    }

    fn launch<M>(&self, monitor: M) -> Result<(), Error>
    where
        M: Monitor + Clone,
    {
        let shmem_provider = StdShMemProvider::new()?;

        let stdout = if self.options.verbose {
            None
        } else {
            Some("/dev/null")
        };

        let client = Client::new(&self.options);
        
        match Launcher::builder()
            .shmem_provider(shmem_provider)
            .broker_port(self.options.port)
            .configuration(EventConfig::from_build_id())
            .monitor(monitor)
            .run_client(|s, m, c| client.run(s, MonitorTypedEventManager::<_, M>::new(m), c))
            .cores(&self.options.cores)
            .stdout_file(stdout)
            .stderr_file(stdout)
            .build()
            .launch()
        {
            Ok(()) => Ok(()),
            Err(Error::ShuttingDown) => {
                println!("Fuzzing stopped by user. Good bye.");
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
