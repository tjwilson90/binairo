use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use bstr::{BString, ByteSlice};
use regex::bytes::Regex;
use z3::{Config, Context, SatResult, Solver};
use z3::ast::Bool;

#[derive(PartialEq)]
enum Fill {
    Blank,
    White,
    Black,
}

fn parse(bytes: &[u8]) -> i32 {
    let mut val = 0;
    for b in bytes {
        val = 10 * val + (*b - b'0');
    }
    val as i32
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut file = File::open("/Users/twilson/code/binairo/binairo.html")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let mut cells = Vec::new();
    let mut width = HashSet::new();
    let mut height = HashSet::new();
    let cell_regex = Regex::new(r#"<div tabindex="1" class="(.*?)" style=".*?top: (.*?); left: (.*?);"#)?;
    for cap in cell_regex.captures_iter(&buf) {
        let class = &cap[1];
        cells.push(if class.find("cell-0").is_some() {
            Fill::White
        } else if class.find("cell-1").is_some() {
            Fill::Black
        } else {
            Fill::Blank
        });
        height.insert(BString::from(&cap[2]));
        width.insert(BString::from(&cap[3]));
    }
    let height = height.len();
    let width = width.len();

    let config = Config::new();
    let context = Context::new(&config);
    let mut variables = Vec::new();
    for i in 0..height {
        for j in 0..width {
            variables.push(Bool::new_const(&context, format!("cell_{}_{}", i, j)));
        }
    }
    let solver = Solver::new(&context);
    for i in 0..height {
        for j in 0..width {
            match &cells[width * i + j] {
                Fill::White => {
                    let var = &variables[width * i + j];
                    solver.assert(&var.not());
                }
                Fill::Black => {
                    let var = &variables[width * i + j];
                    solver.assert(var);
                }
                _ => {}
            }
            if i >= 2 {
                let var1 = &variables[width * i + j];
                let var2 = &variables[width * (i - 1) + j];
                let var3 = &variables[width * (i - 2) + j];
                solver.assert(&Bool::and(&context, &[var1, var2, var3]).not());
                solver.assert(&Bool::or(&context, &[var1, var2, var3]));
            }
            if j >= 2 {
                let var1 = &variables[width * i + j];
                let var2 = &variables[width * i + j - 1];
                let var3 = &variables[width * i + j - 2];
                solver.assert(&Bool::and(&context, &[var1, var2, var3]).not());
                solver.assert(&Bool::or(&context, &[var1, var2, var3]));
            }
        }
    }
    let counter_regex = Regex::new(r#"<span class="sc1.*?">(.*?)</span>"#)?;
    let mut caps = counter_regex.captures_iter(&buf);
    for i in 0..height {
        let cap = caps.next().unwrap();
        let mut blacks = parse(&cap[1]);
        let mut vars = Vec::new();
        for j in 0..width {
            vars.push((&variables[width * i + j], 1));
            if cells[width * i + j] == Fill::Black {
                blacks += 1;
            }
        }
        solver.assert(&Bool::pb_eq(&context, &vars, blacks));
    }
    for j in 0..width {
        let cap = caps.next().unwrap();
        let mut blacks = parse(&cap[1]);
        let mut vars = Vec::new();
        for i in 0..height {
            vars.push((&variables[width * i + j], 1));
            if cells[width * i + j] == Fill::Black {
                blacks += 1;
            }
        }
        solver.assert(&Bool::pb_eq(&context, &vars, blacks));
    }
    assert_eq!(solver.check(), SatResult::Sat);
    let model = solver.get_model().unwrap();

    for i in 0..height {
        for j in 0..width {
            let cell = model.eval(&variables[width * i + j], false).unwrap().as_bool().unwrap();
            if cell {
                print!("⚫");
            } else {
                print!("⚪");
            }
        }
        println!();
    }
    Ok(())
}
