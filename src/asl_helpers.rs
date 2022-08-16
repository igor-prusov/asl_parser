use std::{
    error,
    fs::{self, OpenOptions},
    io::Cursor,
    path::{Path, PathBuf},
    process::Command,
};

use futures::future::try_join_all;
use tempdir::TempDir;

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

pub fn regs_asl_path() -> PathBuf {
    let path = dirs::data_dir().expect("Can't get user data directory");

    let config_dir = path.join("mra_parser");

    fs::create_dir_all(&config_dir).expect("Can't create app data directory");

    config_dir.join("regs.asl")
}

fn run_make(dir: &Path, target: &str) -> Result<()> {
    Command::new("make").current_dir(dir).arg(target).output()?;
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

pub async fn prepare() -> Result<()> {
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
