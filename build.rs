// https://doc.rust-lang.org/cargo/reference/build-scripts.html

use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::Path,
    result::Result,
};

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;
use walkdir::WalkDir;

fn main() -> io::Result<()> {
    let icons_dir = "lucide/icons";
    let out_dir = env::var("OUT_DIR").unwrap();
    let dist = Path::new(&out_dir).join("icons.rs");
    let mut file = File::create(dist)?;

    let features_all = env::var("CARGO_FEATURE_ALL").is_ok();
    let any_feature_enabled = features_all ||
        env::vars()
            .any(|(key, _)| key.starts_with("CARGO_FEATURE_") && key != "CARGO_FEATURE_DEFAULT");

    if !any_feature_enabled {
        return Ok(());
    }

    writeln!(file, "use leptos::prelude::*;")?;

    let regexp = Regex::new(r"(?s)<svg[^>]*>(.*?)</svg>").unwrap();
    let walkdir = WalkDir::new(icons_dir).into_iter().filter_map(Result::ok);

    for entry in walkdir {
        if entry.file_type().is_file() {
            let path = entry.path();
            let extension = path.extension().expect("File has no extension");

            if extension == "svg" {
                let file_stem = path.file_stem().unwrap().to_str().unwrap();
                let feature_name = file_stem.replace('_', "-");
                let feature_enabled = features_all ||
                    env::var(format!(
                        "CARGO_FEATURE_{}",
                        feature_name.to_uppercase().replace('-', "_")
                    ))
                    .is_ok();

                if !feature_enabled {
                    continue;
                }

                let component_name = format_ident!("{}", file_stem.to_pascal_case());
                let svg = fs::read_to_string(path).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to read SVG file {}: {}", path.display(), e),
                    )
                })?;
                let svg_children: TokenStream = regexp
                    .captures(&svg)
                    .and_then(|captures| captures.get(1))
                    .map(|m| m.as_str())
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Invalid SVG format in file {}", path.display()),
                        )
                    })?
                    .parse()
                    .map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Failed to parse SVG content: {e}"),
                        )
                    })?;
                let component_code = generate_component(&component_name, &svg_children);

                writeln!(file, "{component_code}")?;
            }
        }
    }

    Ok(())
}

// FIXME: fix the extra blank space in the generated code
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
