use std::{
    env::args,
    fs,
    fs::{File, OpenOptions},
    io::{self, Cursor, Read, Write},
    path::PathBuf,
    process::Command,
};

use mra_parser::{parse_registers, RegisterDesc};

fn regs_asl_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap();
    path.push("mra_parser");
    path.push("regs.asl");
    path
}

fn init_state() -> File {
    let path = regs_asl_path();

    match File::open(&path) {
        Ok(x) => x,
        Err(e) => panic!("Can't open {}: {}", path.display(), e),
    }
}

async fn prepare() {
    let url_prefix = "https://developer.arm.com/-/media/developer/products/architecture/armv8-a-architecture/2019-12/";
    let tmp_dir = String::from("/Users/igor/tmp/aml/");

    std::fs::create_dir_all(&tmp_dir).unwrap();

    let output = Command::new("git")
        .current_dir(&tmp_dir)
        .arg("clone")
        .arg("https://github.com/alastairreid/mra_tools.git")
        .output()
        .unwrap();

    println!("git clone status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    let mut repo_dir = tmp_dir.clone();
    repo_dir.push_str("mra_tools/");

    let mut spec_dir = repo_dir.clone();
    spec_dir.push_str("v8.6/");
    std::fs::create_dir_all(&spec_dir).unwrap();

    let v = vec![
        "SysReg_xml_v86A-2019-12.tar.gz",
        "A64_ISA_xml_v86A-2019-12.tar.gz",
        "AArch32_ISA_xml_v86A-2019-12.tar.gz",
    ];

    for entry in &v {
        println!("{} downloading...", entry);
        let mut url = String::from(url_prefix);
        url.push_str(entry);

        let mut path = String::from(&spec_dir);
        path.push_str(entry);

        let response = reqwest::get(url).await.unwrap();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();

        let mut content = Cursor::new(response.bytes().await.unwrap());
        std::io::copy(&mut content, &mut file).unwrap();
        println!("{} done ", entry);
    }

    for name in v {
        let output = Command::new("/usr/bin/tar")
            .current_dir(&spec_dir)
            .arg("zxf")
            .arg(name)
            .output()
            .unwrap();
        println!("untar {} status: {}", name, output.status);
    }

    Command::new("make")
        .current_dir(&repo_dir)
        .arg("all")
        .output()
        .unwrap();

    let mut regs_asl = repo_dir.clone();
    regs_asl.push_str("arch/");
    regs_asl.push_str("regs.asl");
    println!("copy: {} -> {}\n", regs_asl, regs_asl_path().display());
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
