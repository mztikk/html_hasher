mod xxHash;

use lol_html::{element, HtmlRewriter, Settings};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use structopt::StructOpt;

use crate::xxHash::xx_hash32;

#[derive(StructOpt)]
struct Cli {
    #[structopt(
        parse(from_os_str),
        help = "File to transform and look for files to hash"
    )]
    file_path: PathBuf,
    #[structopt(
        parse(from_os_str),
        help = "Optional base path to use, if left empty will use directory of file"
    )]
    base_path: Option<PathBuf>,
    #[structopt(
        help = "Keep original files which have been hashed and don't delete them",
        long
    )]
    keep: bool,
    #[structopt(help = "Prints the time it took to run", long, short)]
    show_time: bool,
}

macro_rules! debug {
    ($($e:expr),+) => {
        {
            #[cfg(debug_assertions)]
            {
                dbg!($($e),+)
            }
            #[cfg(not(debug_assertions))]
            {
                ($($e),+)
            }
        }
    };
}

fn get_file_path(file_path: &str, base_path: &Path) -> String {
    if file_path.starts_with('/') {
        let trimmed = file_path.trim_start_matches('/');
        let joined = base_path.join(trimmed);
        return joined.to_str().unwrap().to_string();
    }

    file_path.to_string()
}

fn create_hash_file(file_path: &Path) -> io::Result<String> {
    let file_to_hash = fs::read(file_path)?;
    debug!(format!("read file {}", file_path.display()));

    let hash = xx_hash32(&file_to_hash);
    let hash_string = format!("{:x}", hash);
    debug!(format!(
        "hash of {} is {}",
        &file_path.display(),
        hash_string
    ));

    let new_filename = format!(
        "{}_{}.{}",
        file_path.file_stem().unwrap().to_str().unwrap(),
        hash_string,
        file_path.extension().unwrap().to_str().unwrap()
    );

    debug!(format!("new filename is {}", &new_filename));
    fs::write(&new_filename, file_to_hash)?;
    debug!(format!("wrote file {}", &new_filename));
    println!("{}", new_filename);

    Ok(new_filename)
}

fn main() {
    let args = Cli::from_args();

    debug!("args parsed");
    debug!(&args.file_path);

    let base: &Path;
    if args.base_path.is_none() {
        base = args.file_path.parent().unwrap();
    } else {
        base = args.base_path.as_ref().unwrap();
    }

    debug!(format!("base path is {}", base.display()));
    env::set_current_dir(base).unwrap();

    let now = Instant::now();

    let contents = fs::read_to_string(&args.file_path).unwrap();
    debug!(format!(
        "read contents of file {}",
        &args.file_path.display()
    ));

    let mut output = vec![];

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("script[src]", |el| {
                    let src = el.get_attribute("src").expect("src is required");
                    let src_path_string = get_file_path(&src, base);
                    let src_path = Path::new(&src_path_string);
                    debug!(format!(
                        "found script with src '{}' and path '{}'",
                        &src, &src_path_string
                    ));

                    if let Ok(new_filename) = create_hash_file(src_path) {
                        el.set_attribute("src", &new_filename).unwrap();

                        if !args.keep {
                            fs::remove_file(src_path)?;
                        }
                    }

                    Ok(())
                }),
                element!("link[rel=stylesheet][href]", |el| {
                    let href = el.get_attribute("href").expect("href is required");
                    let href_path_string = get_file_path(&href, base);
                    let href_path = Path::new(&href_path_string);
                    debug!(format!(
                        "found link with href '{}' and path '{}'",
                        &href, &href_path_string
                    ));

                    if let Ok(new_filename) = create_hash_file(href_path) {
                        el.set_attribute("href", &new_filename).unwrap();

                        if !args.keep {
                            fs::remove_file(href_path)?;
                        }
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

    if args.show_time {
        println!("{:#?}", now.elapsed());
    }
}
