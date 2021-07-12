mod texte_fd;
mod tpl;
mod event;

extern crate neovim_lib;

use log::*;
use simplelog::*;
use std::fs::File;
use crate::texte_fd::{Langue, TexteFD, SIZE_VAR_DEF_WITHOUT_SIGN};
use crate::tpl::Tpl;
use crate::event::EventHandler;

struct Detail {
    line: usize,
    name: String,
}

struct NeoVimCobol;

impl NeoVimCobol {
    fn new() -> NeoVimCobol {
        CombinedLogger::init(vec![
            #[cfg(feature = "termcolor")]
                TermLogger::new(
                LevelFilter::Warn,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            #[cfg(not(feature = "termcolor"))]
                SimpleLogger::new(LevelFilter::Warn, Config::default()),
            WriteLogger::new(
                LevelFilter::Info,
                Config::default(),
                File::create("neovim_cobol_rust.log").unwrap(),
            ),
        ]).unwrap();
        NeoVimCobol {}
    }

    // Template to cbl
    fn write_template_to_cbl(&self, serialized: String) {
        let deserialized: Tpl;
        match serde_json::from_str(&serialized) {
            Ok(ok) => deserialized = ok,
            Err(err) => {
                error!("serde deserialize error {:?} {:?}", err, &serialized);
                panic!("panic");
            }
        }
        let _id = deserialized.id.clone().to_lowercase();
        // Search template file

        // Read template file
        let _name = "TODO";

    }

    // Write texte PIC X Fr/De in Working Storage
    fn write_working_texte_fd(&self, serialized: String) -> Result<(usize, usize, Vec<String>, Vec<String>, Vec<String>), ()> {
        let deserialized: TexteFD;
        match serde_json::from_str(&serialized) {
            Ok(ok) => deserialized = ok,
            Err(err) => {
                error!("serde deserialize error {:?} {:?}", err, &serialized);
                return Err(());
            }
        }

        let mut var_def = deserialized.var_def.clone().to_uppercase();
        let l = if var_def.len() > SIZE_VAR_DEF_WITHOUT_SIGN {
            SIZE_VAR_DEF_WITHOUT_SIGN
        } else {
            var_def.len()
        };
        let (first, _) = var_def.split_at(l); // warning this can panic
        var_def = first.to_string();

        // Français
        let t = deserialized.texte_f;
        let (next_size, vec_str_f) = Langue::Francais.write_texte_fd(format!("{}-F", var_def.clone()), t.clone(), 0);

        // Allemand
        let t = deserialized.texte_d;
        let (next_size, vec_str_d) = Langue::Allemand.write_texte_fd(format!("{}-D", var_def.clone()), t.clone(), next_size);

        // FD
        let t = "".to_string();
        let (_, vec_str_fd) = Langue::FD.write_texte_fd(var_def.clone(), t.clone(), next_size);

        // Return
        Ok((deserialized.buffer, deserialized.line, vec_str_f, vec_str_d, vec_str_fd))
    }
}

enum Messages {
    WriteWorkingTexteFD,
    Unknown(String),
}

impl From<String> for Messages {
    fn from(event: String) -> Self {
        match &event[..] {
            "write_working_texte_fd" => Messages::WriteWorkingTexteFD,
            _ => Messages::Unknown(event),
        }
    }
}


fn main() {
    let mut event_handler = EventHandler::new();
    event_handler.recv();
}