mod event;
mod neovim_cobol_const;
mod texte_fd;
mod tpl;

extern crate neovim_lib;

use crate::event::EventHandler;
use crate::neovim_cobol_const::TEMPLATE_PATH;
use crate::texte_fd::{Langue, TexteFD, SIZE_VAR_DEF_WITHOUT_SIGN};
use crate::tpl::{decode, Cbl, Tpl};
use log::*;
use simplelog::*;
use std::fs::File;
use std::path::Path;

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
        ])
        .unwrap();
        NeoVimCobol {}
    }

    // Template to cbl
    fn write_template_to_cbl(&self, serialized: String) -> Result<(), ()> {
        let deserialized: Tpl;
        match serde_json::from_str(&serialized) {
            Ok(ok) => deserialized = ok,
            Err(err) => {
                error!("serde deserialize error {:?} {:?}", err, &serialized);
                return Err(());
            },
        }
        let id = deserialized.id.clone().to_lowercase();
        let mut cbl: Option<Cbl> = None;
        let path: &Path = Path::new(&TEMPLATE_PATH);
        match path.read_dir() {
            Ok(paths) => {
                for res_directory_list in paths {
                    let mut sw_tpl_file = false;
                    let filename: String;
                    match res_directory_list {
                        Ok(directory_list) => {
                            match directory_list.file_name().into_string() {
                                Ok(f) => {
                                    info!("Read dir: {}", f.clone()); // TODO remove this to much log
                                    filename = f.clone();
                                    let v: Vec<&str> = f
                                        .as_str()
                                        .rsplit(|c| c == '.')
                                        .collect();
                                    for (i, x) in v.iter().enumerate() {
                                        if i == 0 && x == &"tpl" {
                                            sw_tpl_file = true;
                                            break;
                                        }
                                    }
                                },
                                Err(err) => {
                                    error!("paths read error filename {:?} {:?} {:?}",path, directory_list, err,);
                                    return Err(());
                                },
                            }
                        },
                        Err(err) => {
                            error!("paths read error {:?} {:?}", path, err);
                            return Err(());
                        },
                    }
                    if sw_tpl_file {
                        let v: Vec<&str> =
                            filename.as_str().split(|c| c == '-').collect();
                        sw_tpl_file = false;
                        for (i, x) in v.into_iter().enumerate() {
                            if i == 0 && x.to_string().to_lowercase() == id {
                                sw_tpl_file = true;
                                break;
                            }
                        }
                        if sw_tpl_file {
                            let path =
                                Path::new(TEMPLATE_PATH).join(filename.clone());
                            match decode(path, filename.clone()) {
                                Ok(c) => {
                                    cbl = Some(Cbl {
                                        name: c.name.clone(),
                                        sel: (c.sel.0, c.sel.1.clone()),
                                        cop: (c.cop.0, c.cop.1.clone()),
                                        status: (
                                            c.status.0,
                                            c.status.1,
                                            c.status.2.clone(),
                                            c.status.3.into_iter().collect(),
                                        ),
                                        open: (
                                            c.open.0,
                                            c.open.1,
                                            c.open.2.clone(),
                                            c.open.3.into_iter().collect(),
                                        ),
                                    });
                                    break;
                                },
                                Err(()) => {
                                    error!(
                                        "template read failed {:?}",
                                        filename.clone(),
                                    );
                                    return Err(());
                                },
                            }
                        }
                    }
                }
                match cbl {
                    Some(c) => {
                        info!("Cbl struct {:?}", c);
                        Ok(())
                    },
                    None => Err(()),
                }
            },
            Err(err) => {
                error!("paths read error {:?} {:?}", path, err);
                return Err(());
            },
        }
    }

    // Write texte PIC X Fr/De in Working Storage
    fn write_working_texte_fd(
        &self,
        serialized: String,
    ) -> Result<(usize, usize, Vec<String>, Vec<String>, Vec<String>), ()> {
        let deserialized: TexteFD;
        match serde_json::from_str(&serialized) {
            Ok(ok) => deserialized = ok,
            Err(err) => {
                error!("serde deserialize error {:?} {:?}", err, &serialized);
                return Err(());
            },
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
        let (next_size, vec_str_f) = Langue::Francais.write_texte_fd(
            format!("{}-F", var_def.clone()),
            t.clone(),
            0,
        );

        // Allemand
        let t = deserialized.texte_d;
        let (next_size, vec_str_d) = Langue::Allemand.write_texte_fd(
            format!("{}-D", var_def.clone()),
            t.clone(),
            next_size,
        );

        // FD
        let t = "".to_string();
        let (_, vec_str_fd) =
            Langue::FD.write_texte_fd(var_def.clone(), t.clone(), next_size);

        // Return
        Ok((
            deserialized.buffer,
            deserialized.line,
            vec_str_f,
            vec_str_d,
            vec_str_fd,
        ))
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
