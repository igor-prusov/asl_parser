use std::{
    env::args,
    fs,
    fs::{File, OpenOptions},
    io::{self, Cursor, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use futures::future::join_all;
use mra_parser::{parse_registers, RegisterDesc};
use tempdir::TempDir;

fn regs_asl_path() -> PathBuf {
    let path = dirs::data_dir().unwrap();

    path.join("mra_parser").join("regs.asl")
}

fn init_state() -> File {
    let path = regs_asl_path();

    match File::open(&path) {
        Ok(x) => x,
        Err(e) => panic!("Can't open {}: {}", path.display(), e),
    }
}

async fn download_file(from: String, to: PathBuf) {
    println!("Downloading: {}", from);
    let response = reqwest::get(&from).await.unwrap();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&to)
        .unwrap();

    let mut content = Cursor::new(response.bytes().await.unwrap());
    std::io::copy(&mut content, &mut file).unwrap();
    println!("Done: {}", from);

    let parent = to.parent().unwrap();
    let output = Command::new("/usr/bin/tar")
        .current_dir(parent)
        .arg("zxf")
        .arg(&to)
        .output()
        .unwrap();
    println!("untar {} status: {}", to.display(), output.status);
}

fn clone_repo(url: &str, dst: &Path) {
    let output = Command::new("git")
        .current_dir(dst)
        .arg("clone")
        .arg(url)
        .output()
        .unwrap();
    println!("git clone status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

async fn download_files(url_prefix: &str, to: &Path, files: &Vec<&str>) {
    let data = files.iter().map(|x| {
        let url = [url_prefix, x].join("");
        let path = to.join(x);
        (url, path)
    });

    let mut promises = Vec::new();

    for (url, path) in data {
        promises.push(download_file(url, path));
    }

    join_all(promises).await;
}

fn run_make(dir: &Path, target: &str) {
    Command::new("make")
        .current_dir(dir)
        .arg(target)
        .output()
        .unwrap();
}

async fn prepare() {
    let tmp_dir = TempDir::new("regs_asl_parser").unwrap().into_path();
    let repo_dir = tmp_dir.join("mra_tools");
    let spec_dir = repo_dir.join("v8.6");

    std::fs::create_dir_all(&tmp_dir).unwrap();

    clone_repo(
        "https://github.com/alastairreid/mra_tools.git",
        tmp_dir.as_path(),
    );

    std::fs::create_dir_all(&spec_dir).unwrap();

    let spec_files = vec![
        "SysReg_xml_v86A-2019-12.tar.gz",
        "A64_ISA_xml_v86A-2019-12.tar.gz",
        "AArch32_ISA_xml_v86A-2019-12.tar.gz",
    ];

    let url_prefix = "https://developer.arm.com/-/media/developer/products/architecture/armv8-a-architecture/2019-12/";
    download_files(url_prefix, &spec_dir, &spec_files).await;

    run_make(&repo_dir, "all");

    let regs_asl = repo_dir.join("arch").join("regs.asl");

    fs::copy(regs_asl, regs_asl_path()).unwrap();
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = args().collect();
    if args.len() > 1 && args[1] == "init" {
        prepare().await;
    }

    let mut file = init_state();
    let mut input = String::new();

    file.read_to_string(&mut input).unwrap();

    let data = parse_registers(&input);

    println!("Enter register names:");
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim().to_lowercase();
        if input.len() == 0 {
            break;
        }

        let it = data
            .range(String::from(&input)..)
            .take_while(|x| x.0.starts_with(&input));

        let m: Vec<&RegisterDesc> = it.map(|(_, v)| v).collect();

        if m.is_empty() {
            continue;
        }

        let mut index = 0;

        if m.len() != 1 {
            for (i, reg) in m.iter().enumerate() {
                println!("{}: {}", i, reg.name);
            }

            print!("{}> ", input);
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            index = match input.trim().parse::<usize>() {
                Ok(x) if x <= m.len() => x,
                _ => continue,
            }
        }

        println!("{}", m[index]);
    }
}
