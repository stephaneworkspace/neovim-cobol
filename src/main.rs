extern crate neovim_lib;

use neovim_lib::{Neovim, NeovimApi, Session, Value};
use log::*;
use simplelog::*;
use std::fs::File;
use neovim_lib::neovim_api::Buffer;
use serde::{Serialize, Deserialize};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Serialize, Deserialize, Debug)]
struct TexteFD {
    buffer: usize,
    var_def: String,
    texte_f: String,
    texte_d: String,
}

enum Langue {
    Francais,
    Allemand,
    FD
}

const SIZE_VAR_DEF: usize = 34;
const SIZE_VAR_DEF_WITHOUT_SIGN: usize = 32;

impl Langue {
    fn write_texte_fd(&self, mut var_def: String, texte: String, last_size: usize) -> (usize, Vec<String>) {
        const SIZE: usize = 60; // size parentese
        let mut vec_ret: Vec<String> = Vec::new();
        let mut vec_pic_x_60: Vec<String> = Vec::new();
        let mut size = 0;
        let l = if var_def.len() > SIZE_VAR_DEF {
            SIZE_VAR_DEF
        } else {
            var_def.len()
        };
        let (first, _) = var_def.split_at(l); // warning, this can panic
        var_def = first.to_string();
        let mut texte_remain = texte;
        loop {
            // this lib unicode_segmentation get the true value of char. string.len() is a utf8 value varying from 1 to 4 per char. French accent is 2.
            if texte_remain.graphemes(true).count() > SIZE {
                let mut i = 0;
                loop {
                    let (first, last) = texte_remain.split_at(SIZE + i);
                    if first.graphemes(true).count() == 60 {
                        vec_pic_x_60.push(first.to_string());
                        texte_remain = last.to_string();
                        size += SIZE;
                        break
                    } else {
                        i += 1;
                    }
                }
            } else {
                vec_pic_x_60.push(texte_remain.clone());
                // size += texte_remain.len();
                size += SIZE;
                break;
            }
        }
        if size == 0 {
            size = 1;
            vec_pic_x_60.push(" ".to_string());
        }
        match self {
            Langue::Francais | Langue::Allemand => {
                if last_size > size {
                    vec_ret.push(format!("           03 {:<34} PIC X({}) VALUE", var_def, last_size));
                } else {
                    vec_ret.push(format!("           03 {:<34} PIC X({}) VALUE", var_def, size));
                }
                for (i, x) in vec_pic_x_60.iter().enumerate() {
                    if i == 0 {
                        //if size > SIZE {
                        if x.clone().graphemes(true).count() > SIZE {
                            vec_ret.push(format!("           \"{:<60}\"", x.clone()));
                        } else if x.clone().graphemes(true).count() == SIZE {
                            vec_ret.push(format!("           \"{}\"", x.clone()));
                            vec_ret.push(format!("           ."));
                        } else {
                            vec_ret.push(format!("           \"{}\".", x.clone()));
                        }
                    } else {
                        if i == vec_pic_x_60.len() - 1 {
                            vec_ret.push(format!("      -    \"{}\".", x.clone()));
                        } else {
                            vec_ret.push(format!("      -    \"{:<60}\"", x.clone()));
                        }
                    }
                }
            },
            Langue::FD => {
                vec_ret.push(format!("           03 {:<34} PIC X({}).", var_def, last_size));
            }
        }
        if last_size > size {
            (last_size, vec_ret.into_iter().collect())
        } else {
            (size, vec_ret.into_iter().collect())
        }
    }
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

    // Write texte PIC X Fr/De in Working Storage
    fn write_working_texte_fd(&self, serialized: String) -> (usize, Vec<String>, Vec<String>, Vec<String>) {
        let deserialized: TexteFD;
        match serde_json::from_str(&serialized) {
            Ok(ok) => deserialized = ok,
            Err(err) => {
                error!("serde deserialize error {:?} {:?}", err, &serialized);
                panic!("panic");
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
        (deserialized.buffer, vec_str_f, vec_str_d, vec_str_fd)
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

struct EventHandler {
    nvim: Neovim,
    cobol: NeoVimCobol,
}

impl EventHandler {
    fn new() -> EventHandler {
        let session = Session::new_parent().unwrap();
        let nvim = Neovim::new(session);
        let calculator = NeoVimCobol::new();

        EventHandler { nvim, cobol: calculator }
    }

    fn recv(&mut self) {
        let receiver = self.nvim.session.start_event_loop_channel();

        for (event, values) in receiver {
            match Messages::from(event) {
                // Handle Texte Working Cobol
                Messages::WriteWorkingTexteFD => {
                    let mut str = values.iter();
                    let serialized_value: Value;
                    match str.next() {
                        Some(s) => serialized_value = s.clone(),
                        None => {
                            error!("serialized_value None");
                            panic!("panic");
                        }
                    }
                    let serialized: String;
                    match serialized_value.as_str() {
                        Some(ok) => serialized = ok.to_string(),
                        None => {
                            error!("serialized: String conversion None");
                            panic!("panic");
                        }
                    }
                    let (no_buffer, vec_str_f, vec_str_d, vec_str_fd) = self.cobol.write_working_texte_fd(serialized);
                    let buffers: Vec<Buffer>;
                    match self.nvim.list_bufs() {
                        Ok(ok) => {
                            buffers = ok.into_iter().collect()
                        },
                        Err(err) => {
                            error!("list_bufs {:?}", err);
                            panic!("panic");
                        }
                    }
                    // Find "       01  TEXTE-D."
                    let lines_buffer: i64;
                    match buffers[no_buffer].line_count(&mut self.nvim) {
                        Ok(ok) => lines_buffer = ok,
                        Err(err) => {
                            error!("line count error {:?}", err);
                            panic!("panic");
                        }
                    }
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
                            panic!("panic");
                        }
                    }
                    match buffers[no_buffer].set_lines(&mut self.nvim, line_insert, line_insert, true, vec_str_f.into_iter().collect()) {
                        Ok(()) => {},
                        Err(err) => {
                            error!("set_lines {:?}", err);
                            panic!("panic");
                        }
                    }
                    // Find "       01  TEXTE-FD."
                    let lines_buffer: i64;
                    match buffers[no_buffer].line_count(&mut self.nvim) {
                        Ok(ok) => lines_buffer = ok,
                        Err(err) => {
                            error!("line count error {:?}", err);
                            panic!("panic");
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
                            panic!("panic");
                        }
                    }
                    match buffers[no_buffer].set_lines(&mut self.nvim, line_insert, line_insert, true, vec_str_d.into_iter().collect()) {
                        Ok(()) => {},
                        Err(err) => {
                            error!("set_lines {:?}", err);
                            panic!("panic");
                        }
                    }
                    // Find "      *END TEXTE-FD"
                    let lines_buffer: i64;
                    match buffers[no_buffer].line_count(&mut self.nvim) {
                        Ok(ok) => lines_buffer = ok,
                        Err(err) => {
                            error!("line count error {:?}", err);
                            panic!("panic");
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
                            panic!("panic");
                        }
                    }
                    match buffers[no_buffer].set_lines(&mut self.nvim, line_insert, line_insert, true, vec_str_fd.into_iter().collect()) {
                        Ok(()) => {},
                        Err(err) => {
                            error!("set_lines {:?}", err);
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

fn main() {
    let mut event_handler = EventHandler::new();
    event_handler.recv();
}