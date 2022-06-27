use crate::mj_lib::Mathjax;
use fancy_regex::Regex;
use clap::{App, Arg, ArgMatches};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use semver::{Version, VersionReq};
use std::io;
use std::process;

pub fn make_app() -> App<'static> {
    App::new("mathjax-inline-preprocessor")
        .about("A mdbook preprocessor which transforms inline mathjax delimiters into mdbook-supported ones.")
        .subcommand(
            App::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    
    let preprocessor = Mathjax::new();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = pre.supports_renderer(renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

mod mj_lib {
    use super::*;
    pub struct Mathjax;

    impl Mathjax {
        pub fn new() -> Mathjax {
            Mathjax
        }
    }

    impl Preprocessor for Mathjax {
        fn name(&self) -> &str {
            "inline-mathjax"
        }

        fn run(&self, _: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
            // Match all '$' which are not preceded by a '\' or another '$', and which are not followed by a '$'
            // The latter is used to avoid matching '$$' blocks, which mdbook already supports.
            let inline_mathjax = Regex::new(r#"(?<!\\)(?<!\$)\$(?!\$)"#).expect("Failed regex!");
            book.for_each_mut(|section: &mut mdbook::BookItem| {
                if let mdbook::BookItem::Chapter(ref mut ch) = *section {
                    let mut i: u32 = 1;
                    let mut edited = String::from(&ch.content);
                    for m in inline_mathjax.find_iter(&ch.content) {
                        if let Ok(_) = m {
                            // If this is a beginning $, replace with \\(
                            if i % 2 == 1 {
                                
                                edited = inline_mathjax.replace(&edited, "\\\\( ").to_string();
                            }
                            // If this is an ending $, replace with \\)
                            else {
                                edited = inline_mathjax.replace(&edited, " \\\\)").to_string();
                            }
                            i+=1;
                        }
                    }
                    ch.content = edited;
                }
            });
            Ok(book)
        }

        fn supports_renderer(&self, renderer: &str) -> bool {
            renderer != "not-supported"
        }
    }
}