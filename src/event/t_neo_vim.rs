use crate::event::EventHandler;
use log::*;
use neovim_lib::neovim_api::Buffer;
use neovim_lib::NeovimApi;

pub trait TNeoVim {
    fn list_bufs(&mut self) -> Result<Vec<Buffer>, ()>;
    fn line_count(
        &mut self,
        buffers: Vec<&Buffer>,
        no_buffer: usize,
    ) -> Result<i64, ()>;
    fn set_lines(
        &mut self,
        buffers: Vec<&Buffer>,
        no_buffer: usize,
        line_start: i64,
        line_end: i64,
        replacement: Vec<String>,
    ) -> Result<(), ()>;
    fn command(&mut self, command: String) -> Result<(), ()>;
}

impl TNeoVim for EventHandler {
    // NeoVim List of buffers
    fn list_bufs(&mut self) -> Result<Vec<Buffer>, ()> {
        match self.nvim.list_bufs() {
            Ok(buffers) => Ok(buffers.into_iter().collect()),
            Err(err) => {
                error!("list_bufs {:?}", err);
                return Err(());
            },
        }
    }
    // NeoVim Line count
    fn line_count(
        &mut self,
        buffers: Vec<&Buffer>,
        no_buffer: usize,
    ) -> Result<i64, ()> {
        match buffers[no_buffer].line_count(&mut self.nvim) {
            Ok(count) => Ok(count),
            Err(err) => {
                error!("line count error {:?}", err);
                return Err(());
            },
        }
    }
    // NeoVim Set line (:put )
    fn set_lines(
        &mut self,
        buffers: Vec<&Buffer>,
        no_buffer: usize,
        line_start: i64,
        line_end: i64,
        replacement: Vec<String>,
    ) -> Result<(), ()> {
        match buffers[no_buffer].set_lines(
            &mut self.nvim,
            line_start,
            line_end,
            true,
            replacement.into_iter().collect(),
        ) {
            Ok(()) => Ok(()),
            Err(err) => {
                error!("set_lines {:?}", err);
                return Err(());
            },
        }
    }
    // Command
    fn command(&mut self, command: String) -> Result<(), ()> {
        match self.nvim.command(command.as_str()) {
            Ok(()) => Ok(()),
            Err(err) => {
                error!("command error {:?}", err);
                Err(())
            },
        }
    }
}
