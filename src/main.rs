use std::{
    collections::BTreeMap,
    env::args,
    error, fs,
    fs::{File, OpenOptions},
    io::{self, Cursor, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use futures::future::try_join_all;
use mra_parser::{parse_registers, RegisterDesc};
use tempdir::TempDir;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

enum Event {
    Text { value: String },
    Number { value: usize },
}

#[derive(Clone)]
enum TState<'a> {
    Empty {},
    Ambiguous {
        vec: Vec<&'a RegisterDesc>,
        prefix: String,
    },
    Selected {
        reg: &'a RegisterDesc,
    },
}

struct FSM<'a> {
    data: &'a BTreeMap<String, RegisterDesc>,
    state: TState<'a>,
}

impl<'a> TState<'a> {
    fn from_prefix(prefix: &str, data: &'a BTreeMap<String, RegisterDesc>) -> TState<'a> {
        let it = data
            .range(String::from(prefix)..)
            .take_while(|x| x.0.starts_with(&prefix));
        let m: Vec<&RegisterDesc> = it.map(|(_, v)| v).collect();
        match m.len() {
            0 => TState::Empty {},
            1 => TState::Selected { reg: m[0] },
            _ => TState::Ambiguous {
                vec: m,
                prefix: prefix.to_string(),
            },
        }
    }
}

impl<'a> FSM<'a> {
    fn new(data: &'a BTreeMap<String, RegisterDesc>) -> FSM<'a> {
        FSM {
            data: data,
            state: TState::Empty {},
        }
    }
    fn next<'b>(&'b mut self, event: Event) {
        self.state = match (&self.state, event) {
            /* From Empty */
            (TState::Empty {}, Event::Number { value: _ }) => TState::Empty {},
            (TState::Empty {}, Event::Text { value }) => TState::from_prefix(&value, &self.data),

            /* From Ambiguous */
            (TState::Ambiguous { vec, prefix }, Event::Number { value }) => {
                if value < vec.len() {
                    TState::Selected { reg: vec[value] }
                } else {
                    TState::Ambiguous {
                        vec: vec.to_vec(),
                        prefix: prefix.to_string(),
                    }
                }
            }

            (TState::Ambiguous { vec, prefix }, Event::Text { value: _ }) => TState::Ambiguous {
                vec: vec.to_vec(),
                prefix: prefix.to_string(),
            },

            /* From Selected */
            (TState::Selected { reg }, Event::Number { value: _ }) => {
                /* TODO: decode here */
                TState::Selected { reg: reg }
            }

            (TState::Selected { reg: _ }, Event::Text { value }) => {
                TState::from_prefix(&value, &self.data)
            }
        };

        if let TState::Selected { reg } = &self.state {
            println!("{}", reg)
        }

        if let TState::Ambiguous { vec, prefix: _ } = &self.state {
            for (i, reg) in vec.iter().enumerate() {
                println!("{}: {}", i, reg.name);
            }
        }
    }

    fn prompt(&self) -> &str {
        match &self.state {
            TState::Empty {} => "",
            TState::Ambiguous { vec: _, prefix } => prefix,
            TState::Selected { reg } => reg.name.as_ref(),
        }
    }
}

fn regs_asl_path() -> PathBuf {
    let path = dirs::data_dir().expect("Can't get user data directory");

    let config_dir = path.join("mra_parser");

    fs::create_dir_all(&config_dir).expect("Can't create app data directory");

    config_dir.join("regs.asl")
}

fn init_state() -> File {
    let path = regs_asl_path();

    match File::open(&path) {
        Ok(x) => x,
        Err(e) => panic!("Can't open {}: {}", path.display(), e),
    }
}

async fn download_file(from: String, to: PathBuf) -> Result<()> {
    let response = reqwest::get(&from).await?;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&to)?;

    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;

    let parent = to.parent().expect("Destination must have parent directory");
    Command::new("/usr/bin/tar")
        .current_dir(parent)
        .arg("zxf")
        .arg(&to)
        .output()?;

    Ok(())
}

fn clone_repo(url: &str, dst: &Path) -> Result<()> {
    Command::new("git")
        .current_dir(dst)
        .arg("clone")
        .arg(url)
        .output()?;
    Ok(())
}

async fn download_files(url_prefix: &str, to: &Path, files: &[&str]) -> Result<()> {
    let data = files.iter().map(|x| {
        let url = [url_prefix, x].join("");
        let path = to.join(x);
        (url, path)
    });

    let mut promises = Vec::new();

    for (url, path) in data {
        promises.push(download_file(url, path));
    }

    try_join_all(promises).await?;
    Ok(())
}

fn run_make(dir: &Path, target: &str) -> Result<()> {
    Command::new("make").current_dir(dir).arg(target).output()?;
    Ok(())
}

async fn prepare() -> Result<()> {
    let tmp_dir = TempDir::new("regs_asl_parser")?.into_path();
    let repo_dir = tmp_dir.join("mra_tools");
    let spec_dir = repo_dir.join("v8.6");

    std::fs::create_dir_all(&tmp_dir)?;

    println!("Cloning alastarreid/mra_tools");
    clone_repo(
        "https://github.com/alastairreid/mra_tools.git",
        tmp_dir.as_path(),
    )?;

    std::fs::create_dir_all(&spec_dir)?;

    let spec_files = vec![
        "SysReg_xml_v86A-2019-12.tar.gz",
        "A64_ISA_xml_v86A-2019-12.tar.gz",
        "AArch32_ISA_xml_v86A-2019-12.tar.gz",
    ];

    let url_prefix = "https://developer.arm.com/-/media/developer/products/architecture/armv8-a-architecture/2019-12/";

    println!("Downloading and unpacking armv8-A spec");
    download_files(url_prefix, &spec_dir, &spec_files).await?;

    println!("Building mra_tools");
    run_make(&repo_dir, "all")?;

    let regs_asl = repo_dir.join("arch").join("regs.asl");

    println!("Copying regs.asl");
    fs::copy(regs_asl, regs_asl_path())?;
    println!("Initialized");

    Ok(())
}

fn run_tui(data: &BTreeMap<String, RegisterDesc>) -> Result<()> {
    let mut fsm = FSM::new(data);
    println!("Enter register names:");
    loop {
        print!("{}> ", fsm.prompt());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_lowercase();
        if input.is_empty() {
            break;
        }

        let event = match input.parse::<usize>() {
            Ok(x) => Event::Number { value: x },
            Err(_) => Event::Text { value: input },
        };

        fsm.next(event);
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = args().collect();
    if args.len() > 1 && args[1] == "init" {
        if let Err(e) = prepare().await {
            panic!("Can't initialize regs.asl: {}", e);
        }
    }

    let mut file = init_state();
    let mut input = String::new();

    file.read_to_string(&mut input)
        .expect("Can't open regs.asl");

    let data = parse_registers(&input);

    run_tui(&data).expect("Error while interacting with user");
}
