extern crate tokio;
use crate::Event;
use anyhow::Result;
use log::error;
use std::collections::BTreeMap;
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;

#[derive(Debug, Clone)]
pub struct CachedEvent {
    pub json: String,
    pub event: Event,
}

impl CachedEvent {
    pub fn from_event(event: Event) -> Result<Self> {
        let json = event.to_json()?;
        Ok(Self { json, event })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventId {
    // Timestamp must happen first so everything is stored in time-order.
    pub timestamp: u64,
    pub provider_name: String,
    pub msgid: String,
}

impl EventId {
    pub fn from_event(event: &Event) -> Self {
        match event {
            Event::Message(msg) => Self {
                timestamp: msg.timestamp,
                provider_name: msg.provider_name.clone(),
                msgid: msg.msgid.clone(),
            },
        }
    }
}

#[derive(Debug)]
pub struct Log {
    maxsize: usize,
    log_path: String,
    checkpoint_path: String,
    log_lines: usize,
    log_writer: Option<BufWriter<File>>,
    pub data: BTreeMap<EventId, CachedEvent>,
}

impl Log {
    pub fn new(maxsize: usize, log_path: String, checkpoint_path: String) -> Self {
        Self {
            maxsize,
            log_path,
            checkpoint_path,
            log_lines: 0,
            log_writer: None,
            data: BTreeMap::new(),
        }
    }
    pub async fn load(&mut self) -> Result<()> {
        self.log_writer.take();
        match File::open(self.checkpoint_path.clone()).await {
            Err(e) => {
                error!(
                    "couldn't open checkpoint file {:?}: {:?}",
                    self.checkpoint_path, e
                );
            }
            Ok(chkfile) => {
                let my_buf_read = BufReader::new(chkfile);
                let mut lines = my_buf_read.lines();

                while let Some(line) = lines.next_line().await? {
                    match Event::from_json(&line) {
                        Ok(m) => {
                            if let Err(e) = self.push_int(m) {
                                error!("error while writing to database: {:?}", e);
                            }
                        }
                        Err(e) => error!(
                            "error while parsing JSON from checkpoint file: {:?}\n\
                        original line: {:?}",
                            e, line
                        ),
                    }
                    self.log_lines += 1;
                }
            }
        }
        match File::open(self.log_path.clone()).await {
            Err(e) => {
                error!("couldn't open log file {:?}: {:?}", self.log_path, e);
            }
            Ok(logfile) => {
                let my_buf_read = BufReader::new(logfile);
                let mut lines = my_buf_read.lines();

                while let Some(line) = lines.next_line().await? {
                    match Event::from_json(&line) {
                        Ok(m) => {
                            if let Err(e) = self.push_int(m) {
                                error!("error while writing to database: {:?}", e);
                            }
                        }
                        Err(e) => error!(
                            "error while parsing JSON from log file: {:?}\n\
                        original line: {:?}",
                            e, line
                        ),
                    }
                }
            }
        }
        self.perform_checkpoint().await?;
        Ok(())
    }
    pub async fn perform_checkpoint(&mut self) -> Result<()> {
        self.log_writer.take();

        {
            let mut writer = BufWriter::new(File::create(self.checkpoint_path.clone()).await?);
            for (_, ce) in self.data.iter() {
                writer.write_all(ce.json.as_bytes()).await?;
            }
            writer.flush().await?;
        } // ensure writer is closed at this point.
        self.log_lines = 0;
        let mut writer = BufWriter::new(File::create(self.log_path.clone()).await?);
        writer.flush().await?;
        self.log_writer = Some(writer);

        Ok(())
    }
    async fn get_writer(&mut self) -> Result<&mut BufWriter<File>> {
        if self.log_writer.is_none() {
            self.perform_checkpoint().await?;
        }
        Ok(self.log_writer.as_mut().unwrap())
    }
    async fn log(&mut self, json: String) -> Result<()> {
        let writer = self.get_writer().await?;
        writer.write_all(json.as_bytes()).await?;
        // Depending on the consistency requirements, we can flush here or not.
        // unless we want to be resilient to power outages and segfaults, this
        // is not needed:
        // writer.flush().await?;
        self.log_lines += 1;
        if self.log_lines > self.maxsize {
            self.perform_checkpoint().await?;
        }
        Ok(())
    }
    fn push_int(&mut self, event: Event) -> Result<()> {
        while self.data.len() >= self.maxsize {
            let first = self.data.iter().next().unwrap().0.clone();
            self.data.remove(&first);
        }

        let key = EventId::from_event(&event);
        let cm = CachedEvent::from_event(event)?;
        self.data.insert(key, cm);

        Ok(())
    }
    pub async fn push(&mut self, event: Event) -> Result<()> {
        while self.data.len() >= self.maxsize {
            let first = self.data.iter().next().unwrap().0.clone();
            self.data.remove(&first);
        }

        let key = EventId::from_event(&event);
        let ce = CachedEvent::from_event(event)?;
        self.log(ce.json.clone()).await?;
        self.data.insert(key, ce);

        Ok(())
    }
}
