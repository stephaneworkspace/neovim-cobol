use crate::event::t_neo_vim::TNeoVim;
use crate::event::EventHandler;
use log::*;
use neovim_lib::neovim_api::Buffer;
use neovim_lib::Value;
use std::slice::Iter;

pub trait TTexteFd {
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
            },
        }
        let serialized: String;
        match serialized_value.as_str() {
            Some(ok) => serialized = ok.to_string(),
            None => {
                error!("serialized: String conversion None");
                return Err(());
            },
        }
        let (no_buffer, current_line_no, vec_str_f, vec_str_d, vec_str_fd) =
            self.cobol.write_working_texte_fd(serialized)?;
        let buffers: Vec<Buffer> = self.list_bufs()?;
        // Find "       01  TEXTE-D."
        let lines_buffer =
            self.line_count(buffers.iter().collect(), no_buffer)?;
        let mut line_insert: i64 = 0;
        match buffers[no_buffer].get_lines(
            &mut self.nvim,
            0,
            lines_buffer,
            true,
        ) {
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
            },
        }
        self.set_lines(
            buffers.iter().collect(),
            no_buffer.clone(),
            line_insert.clone(),
            line_insert,
            vec_str_f.into_iter().collect(),
        )?;
        // Find "       01  TEXTE-FD."
        let lines_buffer =
            self.line_count(buffers.iter().collect(), no_buffer)?;
        let mut line_insert: i64 = 0;
        match buffers[no_buffer].get_lines(
            &mut self.nvim,
            0,
            lines_buffer,
            true,
        ) {
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
            },
        }
        self.set_lines(
            buffers.iter().collect(),
            no_buffer.clone(),
            line_insert.clone(),
            line_insert,
            vec_str_d.into_iter().collect(),
        )?;
        // Find "      *END TEXTE-FD"
        let lines_buffer: i64;
        match buffers[no_buffer].line_count(&mut self.nvim) {
            Ok(ok) => lines_buffer = ok,
            Err(err) => {
                error!("line count error {:?}", err);
                return Err(());
            },
        }
        let mut line_insert: i64 = 0;
        match buffers[no_buffer].get_lines(
            &mut self.nvim,
            0,
            lines_buffer,
            true,
        ) {
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
            },
        }
        self.set_lines(
            buffers.iter().collect(),
            no_buffer.clone(),
            line_insert.clone(),
            line_insert,
            vec_str_fd.into_iter().collect(),
        )?;
        self.command(
            (&*format!(":normal {}G{}", current_line_no, 11))
                .parse()
                .unwrap(),
        )?;
        Ok(())
    }
}
