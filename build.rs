// https://doc.rust-lang.org/cargo/reference/build-scripts.html

use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::Path,
    process::Command,
    result::Result,
};

use heck::ToPascalCase;
use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rayon::prelude::*;
use regex::Regex;
use syn::parse_file;
use walkdir::WalkDir;

fn main() -> io::Result<()> {
    let icons_dir = "lucide/icons";
    let out_dir = env::var("OUT_DIR").unwrap();
    let dist = Path::new(&out_dir).join("icons.rs");
    let mut file = File::create(dist)?;

    check_leptosfmt_installed()?;

    writeln!(file, "use leptos::prelude::*;\n")?;

    let regexp = Regex::new(r"(?s)<svg[^>]*>(.*?)</svg>").unwrap();
    let walkdir = WalkDir::new(icons_dir)
        .into_iter()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    let formatted_codes: Vec<String> = walkdir
        .par_iter()
        .filter_map(|entry| {
            if entry.file_type().is_file() {
                let path = entry.path();
                let extension = path.extension()?;

                if extension == "svg" {
                    let file_stem = path.file_stem()?.to_str()?;
                    let component_name = format_ident!("{}", file_stem.to_pascal_case());
                    let svg = fs::read_to_string(path).ok()?;
                    let svg_children: TokenStream = regexp
                        .captures(&svg)
                        .and_then(|captures| captures.get(1))
                        .map(|m| m.as_str())?
                        .parse()
                        .ok()?;
                    let component_code = generate_component(&component_name, &svg_children);
                    let component_string = component_code.to_string();
                    let syntax_tree = parse_file(&component_string).ok()?;
                    let prettyplease_formatted_code = unparse(&syntax_tree);

                    let mut child = Command::new("leptosfmt")
                        .arg("--stdin")
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .spawn()
                        .ok()?;

                    if let Some(mut stdin) = child.stdin.take() {
                        stdin
                            .write_all(prettyplease_formatted_code.as_bytes())
                            .ok()?;
                    }

                    let output = child.wait_with_output().ok()?;
                    let formatted_code = String::from_utf8(output.stdout).ok()?;

                    return Some(formatted_code);
                }
            }

            None
        })
        .collect();

    for code in formatted_codes {
        writeln!(file, "{}", code)?;
    }

    Ok(())
}

fn check_leptosfmt_installed() -> io::Result<()> {
    match Command::new("leptosfmt").arg("--version").output() {
        Ok(_) => Ok(()),
        Err(_) =>
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "leptosfmt not found. Please install it with: cargo install leptosfmt",
            )),
    }
}

fn generate_component(
    component_name: &proc_macro2::Ident,
    svg_children: &TokenStream,
) -> TokenStream {
    quote! {
        #[component]
        pub fn #component_name(
            #[prop(into, optional)] class: Signal<String>
        ) -> impl IntoView {
            view! {
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class=class
                >
                    #svg_children
                </svg>
            }
        }
    }
}
