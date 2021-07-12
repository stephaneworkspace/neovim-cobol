use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Tpl {
    pub buffer: usize,
    pub line: usize,
    pub id: String,
    pub open_mode: OpenMode
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OpenMode {
    Input(OpenStatus),
    InputOutput,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OpenStatusMode {
    Output,
    Error(Vec<String>)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenStatus {
    pub status: Vec<(usize, OpenStatusMode)>, // Vec<n° status, OpenStatusMode>
}

struct Cbl {
    pub name: String, // data table name uppercase
    pub sel: Vec<(usize, String)>, // line, id search, ex for: "./sel/db01.cpy" -> "db01"
    pub cop: Vec<(usize, String)>, // line, id search, ex for: "./cop/imdb01.cpy" -> "imdb01"
    pub status: Vec<(usize, usize, String, String)>, // line begin, line end, id for search, 01 Status
    pub open: Vec<(usize, usize, String)>, // line begin, muss be part of 01 of status, line end (close/open action), id (Tpl.id)
}