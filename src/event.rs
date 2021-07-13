mod t_neo_vim;
mod t_texte_fd;
mod t_tpl;

use crate::event::t_texte_fd::TTexteFd;
use crate::{Messages, NeoVimCobol};
use log::*;
use neovim_lib::{Neovim, NeovimApi, Session};

pub struct EventHandler {
    nvim: Neovim,
    cobol: NeoVimCobol,
}

impl EventHandler {
    pub fn new() -> EventHandler {
        let session = Session::new_parent().unwrap();
        let nvim = Neovim::new(session);
        let cobol = NeoVimCobol::new();

        EventHandler { nvim, cobol }
    }

    pub fn recv(&mut self) {
        let receiver = self.nvim.session.start_event_loop_channel();

        for (event, values) in receiver {
            match Messages::from(event) {
                // Handle Texte Working Cobol
                Messages::WriteWorkingTexteFD => {
                    match self.event_texte_fd(values.iter()) {
                        Ok(()) => {},
                        Err(()) => {
                            error!("Error in event_texte_fd in recv -> Messages::WriteWorkingTexteFD");
                            panic!("panic");
                        },
                    }
                },

                // Handle anything else
                Messages::Unknown(event) => {
                    self.nvim
                        .command(&format!(
                            "echo \"Unknown command: {}\"",
                            event
                        ))
                        .unwrap();
                },
            }
        }
    }
}
