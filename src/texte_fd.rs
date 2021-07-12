use unicode_segmentation::UnicodeSegmentation;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TexteFD {
    pub buffer: usize,
    pub line: usize,
    pub var_def: String,
    pub texte_f: String,
    pub texte_d: String,
}

pub enum Langue {
    Francais,
    Allemand,
    FD
}

pub const SIZE_VAR_DEF: usize = 34;
pub const SIZE_VAR_DEF_WITHOUT_SIGN: usize = 32;

impl Langue {
    // Write PIC X in Working Français/Deutch + Combined structure FD
    pub fn write_texte_fd(&self, mut var_def: String, texte: String, last_size: usize) -> (usize, Vec<String>) {
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
