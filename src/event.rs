use neovim_lib::{Neovim, Session, Value, NeovimApi};
use crate::{NeoVimCobol, Messages};
use neovim_lib::neovim_api::Buffer;
use std::slice::Iter;
use log::*;

pub struct EventHandler {
    nvim: Neovim,
    cobol: NeoVimCobol,
}

trait TNeoVim {
    fn list_bufs(&mut self) -> Result<Vec<Buffer>, ()>;
    fn line_count(&mut self, buffers: Vec<&Buffer>, no_buffer: usize) -> Result<i64, ()>;
}

impl TNeoVim for EventHandler {
    // NeoVim List of buffers
    fn list_bufs(&mut self) -> Result<Vec<Buffer>, ()> {
        match self.nvim.list_bufs() {
            Ok(buffers) => {
                Ok(buffers.into_iter().collect())
            },
            Err(err) => {
                error!("list_bufs {:?}", err);
                return Err(());
            }
        }
    }
    // NeoVim Line count
    fn line_count(&mut self, buffers: Vec<&Buffer>, no_buffer: usize) -> Result<i64, ()> {
        match buffers[no_buffer].line_count(&mut self.nvim) {
            Ok(count) => Ok(count),
            Err(err) => {
                error!("line count error {:?}", err);
                return Err(());
            }
        }
    }
}

trait TTexteFd {
    fn event_texte_fd(&mut self, values: Iter<Value>) -> Result<(), ()>;
}

impl TTexteFd for EventHandler {
    fn event_texte_fd(&mut self, mut values: Iter<Value>) -> Result<(), ()> {
        let serialized_value: Value;
        match values.next() {
            Some(s) => serialized_value = s.clone(),
            None => {
                error!("serialized_value None");
                return Err(());
            }
        }
        let serialized: String;
        match serialized_value.as_str() {
            Some(ok) => serialized = ok.to_string(),
            None => {
                error!("serialized: String conversion None");
                return Err(());
            }
        }
        let (no_buffer, current_line_no, vec_str_f, vec_str_d, vec_str_fd) = self.cobol.write_working_texte_fd(serialized)?;
        let buffers: Vec<Buffer> = self.list_bufs()?;
        // Find "       01  TEXTE-D."
        let lines_buffer = self.line_count(buffers.iter().collect(), no_buffer)?;
        let mut line_insert: i64 = 0;
        match buffers[no_buffer].get_lines(&mut self.nvim, 0, lines_buffer, true) {
            Ok(ok) => {
                for (i, x) in ok.into_iter().enumerate() {
                    if x.starts_with(&"       01  TEXTE-D.".to_string()) {
                        if i > 0 {
                            line_insert = i as i64;
                            break;
                        }
                    }
                }
            },
            Err(err) => {
                error!("get_lines {:?}", err);
                return Err(());
            }
        }
        match buffers[no_buffer].set_lines(&mut self.nvim, line_insert, line_insert, true, vec_str_f.into_iter().collect()) {
            Ok(()) => {},
            Err(err) => {
                error!("set_lines {:?}", err);
                return Err(());
            }
        }
        // Find "       01  TEXTE-FD."
        let lines_buffer: i64;
        match buffers[no_buffer].line_count(&mut self.nvim) {
            Ok(ok) => lines_buffer = ok,
            Err(err) => {
                error!("line count error {:?}", err);
                return Err(());
            }
        }
        let mut line_insert: i64 = 0;
        match buffers[no_buffer].get_lines(&mut self.nvim, 0, lines_buffer, true) {
            Ok(ok) => {
                for (i, x) in ok.into_iter().enumerate() {
                    if x.starts_with(&"       01  TEXTE-FD.".to_string()) {
                        if i > 0 {
                            line_insert = i as i64;
                            break;
                        }
                    }
                }
            },
            Err(err) => {
                error!("get_lines {:?}", err);
                return Err(());
            }
        }
        match buffers[no_buffer].set_lines(&mut self.nvim, line_insert, line_insert, true, vec_str_d.into_iter().collect()) {
            Ok(()) => {},
            Err(err) => {
                error!("set_lines {:?}", err);
                return Err(());
            }
        }
        // Find "      *END TEXTE-FD"
        let lines_buffer: i64;
        match buffers[no_buffer].line_count(&mut self.nvim) {
            Ok(ok) => lines_buffer = ok,
            Err(err) => {
                error!("line count error {:?}", err);
                return Err(());
            }
        }
        let mut line_insert: i64 = 0;
        match buffers[no_buffer].get_lines(&mut self.nvim, 0, lines_buffer, true) {
            Ok(ok) => {
                for (i, x) in ok.into_iter().enumerate() {
                    if x.starts_with(&"      *END TEXTE-FD".to_string()) {
                        if i > 0 {
                            line_insert = i as i64;
                            break;
                        }
                    }
                }
            },
            Err(err) => {
                error!("get_lines {:?}", err);
                return Err(());
            }
        }
        match buffers[no_buffer].set_lines(&mut self.nvim, line_insert, line_insert, true, vec_str_fd.into_iter().collect()) {
            Ok(()) => {},
            Err(err) => {
                error!("set_lines {:?}", err);
                return Err(());
            }
        }
        self.nvim.command(&*format!(":normal {}G{}", current_line_no, 11));
        Ok(())
    }
}

impl EventHandler {
    pub fn new() -> EventHandler {
        let session = Session::new_parent().unwrap();
        let nvim = Neovim::new(session);
        let calculator = NeoVimCobol::new();

        EventHandler { nvim, cobol: calculator }
    }

    pub fn recv(&mut self) {
        let receiver = self.nvim.session.start_event_loop_channel();

        for (event, values) in receiver {
            match Messages::from(event) {
                // Handle Texte Working Cobol
                Messages::WriteWorkingTexteFD => {
                    //let mut str = values.iter();
                    //let serialized_value: Value;
                    match self.event_texte_fd(values.iter()) {
                        Ok(()) => {},
                        Err(()) => {
                            error!("Error in event_texte_fd in recv -> Messages::WriteWorkingTexteFD");
                            panic!("panic");
                        }
                    }
                }

                // Handle anything else
                Messages::Unknown(event) => {
                    self.nvim
                        .command(&format!("echo \"Unknown command: {}\"", event))
                        .unwrap();
                }
            }
        }
    }
}
