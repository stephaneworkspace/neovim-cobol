use crate::neovim_cobol_const::{
    COBOL_COMMENT_TEXT, TEMPLATE_DECORATOR_COP, TEMPLATE_DECORATOR_NAME,
    TEMPLATE_DECORATOR_OPEN_BEGIN, TEMPLATE_DECORATOR_OPEN_END,
    TEMPLATE_DECORATOR_SEL, TEMPLATE_DECORATOR_STATUS_BEGIN,
    TEMPLATE_DECORATOR_STATUS_END,
};
use log::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Tpl {
    pub buffer: usize,
    pub line: usize,
    pub id: String,
    pub open_mode: OpenMode,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OpenMode {
    Input(OpenStatus),
    InputOutput,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OpenStatusMode {
    Output,
    Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenStatus {
    pub status: Vec<(usize, OpenStatusMode)>, // Vec<n° status, OpenStatusMode>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cbl {
    pub name: String,         // data table name uppercase
    pub sel: (usize, String), // line, id search, ex for: "./sel/db01.cpy" -> "db01"
    pub cop: (usize, String), // line, id search, ex for: "./cop/imdb01.cpy" -> "imdb01"
    pub status: (usize, usize, String, Vec<String>), // line begin, line end, id for search W 01 Status
    pub open: (usize, usize, String, Vec<String>), // line begin, muss be part of 01 of status, line end (close/open action), id (Tpl.id) / TODO
}

pub enum TemplateDecorator {
    Name(String),
    Sel(String),
    Cop(String),
    Status((TemplateDecoratorPosition, String)),
    Open((TemplateDecoratorPosition, String)),
}

impl fmt::Display for TemplateDecorator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TemplateDecorator::*;
        match self {
            Name(x) => {
                write!(f, "{}", *x)
            },
            Sel(x) => {
                write!(f, "{}", *x)
            },
            Cop(x) => {
                write!(f, "{}", *x)
            },
            Status((_, x)) => {
                write!(f, "{}", *x)
            },
            Open((_, x)) => {
                write!(f, "{}", *x)
            },
        }
    }
}
pub fn decode(path_buf: PathBuf, filename: String) -> Result<Cbl, ()> {
    use TemplateDecorator::*;
    let file = match File::open(path_buf) {
        Ok(ok) => ok,
        Err(err) => {
            error!("file read error {:?} {:?}", filename.clone(), err);
            panic!("panic");
        },
    };
    let reader = BufReader::new(&file);
    let decode_string = |param: (String, String)| {
        let (p, s) = param;
        let pattern_select = format!("{} ", p);
        let split_sel: Vec<&str> = s.rsplit(&pattern_select).collect();
        split_sel.iter().fold(String::new(), |mut str, &w| {
            str.push_str(w);
            str
        })
    };
    let mut cbl = Cbl {
        name: "".to_string(),
        sel: (0, "".to_string()),
        cop: (0, "".to_string()),
        status: (0, 0, "".to_string(), vec![]),
        open: (0, 0, "".to_string(), vec![]),
    };
    let mut sw_pos = false;
    for (n, l) in reader.lines().enumerate() {
        match l {
            Ok(l_string) => {
                for x in get_template_arr() {
                    if l_string.starts_with(&x.to_string()) {
                        match x {
                            Name(y) => {
                                cbl.name = decode_string((y, l_string.clone()));
                            },
                            Sel(y) => {
                                cbl.sel = (
                                    n.clone(),
                                    decode_string((y, l_string.clone())),
                                );
                            },
                            Cop(y) => {
                                cbl.cop = (
                                    n.clone(),
                                    decode_string((y, l_string.clone())),
                                );
                            },
                            Status((pos, y)) => {
                                if pos == TemplateDecoratorPosition::Begin {
                                    sw_pos = true;
                                    cbl.status.0 = n.clone();
                                    cbl.status.2 =
                                        decode_string((y, l_string.clone()));
                                }
                                if pos == TemplateDecoratorPosition::End {
                                    sw_pos = false;
                                    cbl.status.1 = n.clone();
                                }
                                if sw_pos {
                                    cbl.status.3.push(l_string.clone());
                                }
                            },
                            Open((pos, y)) => {
                                if pos == TemplateDecoratorPosition::Begin {
                                    sw_pos = true;
                                    cbl.status.0 = n.clone();
                                    cbl.status.2 =
                                        decode_string((y, l_string.clone()));
                                }
                                if pos == TemplateDecoratorPosition::End {
                                    sw_pos = false;
                                    cbl.status.1 = n.clone();
                                }
                                if sw_pos {
                                    cbl.status.3.push(l_string.clone())
                                }
                            },
                        }
                    }
                }
            },
            Err(err) => {
                error!(
                    "line read error in file {:?} {:?}",
                    filename.clone(),
                    err
                );
                panic!("panic");
            },
        }
    }
    if cbl.name.is_empty() {
        error!("name is empty");
        return Err(());
    }
    // TODO more tests
    Ok(cbl)
}

#[derive(PartialEq, Debug)]
pub enum TemplateDecoratorPosition {
    Begin,
    End,
}

pub fn get_template_arr() -> Vec<TemplateDecorator> {
    use TemplateDecorator::*;
    use TemplateDecoratorPosition::*;
    vec![
        Name(format!(
            "{}{}",
            COBOL_COMMENT_TEXT.to_string(),
            TEMPLATE_DECORATOR_NAME.to_string()
        )),
        Sel(format!(
            "{}{}",
            COBOL_COMMENT_TEXT.to_string(),
            TEMPLATE_DECORATOR_SEL.to_string()
        )),
        Cop(format!(
            "{}{}",
            COBOL_COMMENT_TEXT.to_string(),
            TEMPLATE_DECORATOR_COP.to_string()
        )),
        Status((
            Begin,
            format!(
                "{}{}",
                COBOL_COMMENT_TEXT.to_string(),
                TEMPLATE_DECORATOR_STATUS_BEGIN.to_string()
            ),
        )),
        Status((
            End,
            format!(
                "{}{}",
                COBOL_COMMENT_TEXT.to_string(),
                TEMPLATE_DECORATOR_STATUS_END.to_string()
            ),
        )),
        Open((
            Begin,
            format!(
                "{}{}",
                COBOL_COMMENT_TEXT.to_string(),
                TEMPLATE_DECORATOR_OPEN_BEGIN.to_string()
            ),
        )),
        Open((
            End,
            format!(
                "{}{}",
                COBOL_COMMENT_TEXT.to_string(),
                TEMPLATE_DECORATOR_OPEN_END.to_string()
            ),
        )),
    ]
}
