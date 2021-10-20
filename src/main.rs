mod xxHash;

use lol_html::{element, HtmlRewriter, Settings};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Instant,
};
use structopt::StructOpt;

use crate::xxHash::xx_hash32;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    file_path: PathBuf,
}

fn main() {
    let args = Cli::from_args();

    dbg!("args parsed");
    dbg!(&args.file_path);

    let base = args.file_path.parent().unwrap();

    env::set_current_dir(base).unwrap();
    dbg!(format!("base path set to {}", &base.display()));

    let now = Instant::now();

    let contents = fs::read_to_string(&args.file_path).unwrap();
    dbg!(format!(
        "read contents of file {}",
        &args.file_path.display()
    ));

    let mut output = vec![];

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("script[src]", |el| {
                    let src = el.get_attribute("src").expect("src is required");

                    let src_path = Path::new(&src);
                    dbg!(format!("found script with src {}", src));

                    if let Ok(file_to_hash) = fs::read(src_path) {
                        let hash = xx_hash32(&file_to_hash);
                        let hash_string = format!("{:x}", hash);
                        dbg!(format!("hash of {} is {}", src, hash_string));

                        let new_filename = format!(
                            "{}_{}.{}",
                            src_path.file_stem().unwrap().to_str().unwrap(),
                            hash_string,
                            src_path.extension().unwrap().to_str().unwrap()
                        );

                        dbg!(format!("new filename is {}", &new_filename));
                        println!("{}", new_filename);

                        fs::write(&new_filename, file_to_hash).unwrap();
                        el.set_attribute("src", &new_filename).unwrap();
                    }

                    Ok(())
                }),
                element!("link[rel=stylesheet][href]", |el| {
                    let href = el.get_attribute("href").expect("href is required");

                    let href_path = Path::new(&href);
                    dbg!(format!("found link with href {}", href));

                    if let Ok(file_to_hash) = fs::read(href_path) {
                        let hash = xx_hash32(&file_to_hash);
                        let hash_string = format!("{:x}", hash);
                        dbg!(format!("hash of {} is {}", href, hash_string));

                        let new_filename = format!(
                            "{}_{}.{}",
                            href_path.file_stem().unwrap().to_str().unwrap(),
                            hash_string,
                            href_path.extension().unwrap().to_str().unwrap()
                        );

                        dbg!(format!("new filename is {}", &new_filename));
                        println!("{}", new_filename);

                        fs::write(&new_filename, file_to_hash).unwrap();
                        el.set_attribute("href", &new_filename).unwrap();
                    }
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );

    rewriter.write(contents.as_bytes()).unwrap();
    rewriter.end().unwrap();

    fs::write(&args.file_path, output).unwrap();

    println!("{:#?}", now.elapsed());
}
