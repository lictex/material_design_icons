use anyhow::*;
use heck::*;
use quote::*;
use std::collections::HashMap;

fn main() {
    checkout().unwrap()
}

fn checkout() -> Result<()> {
    let out_dir = std::env::var("OUT_DIR")?;
    let version = "9363cbecde77f1de821e5799457e89b6b2e82c26";

    let git_dir = std::path::Path::new(&out_dir)
        .join("../../material_design_icons")
        .join(&version);

    fn git_reset(version: &str, git_dir: &std::path::PathBuf) -> std::result::Result<(), Error> {
        std::process::Command::new("git")
            .args(&["reset", "--hard", &version])
            .current_dir(git_dir)
            .spawn()?
            .wait()?
            .success()
            .then_some(())
            .ok_or(anyhow!("git reset failed"))
    }

    let git_ready = git_dir
        .exists()
        .then(|| git_reset(version, &git_dir))
        .and_then(Result::ok)
        .is_some();
    if !git_ready {
        let _ = std::fs::remove_dir_all(&git_dir);
        std::fs::create_dir_all(&git_dir)?;
        std::process::Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                "https://github.com/Templarian/MaterialDesign",
                "./",
            ])
            .current_dir(&git_dir)
            .spawn()?
            .wait()?
            .success()
            .then_some(())
            .ok_or(anyhow!("git clone failed"))?
    }
    git_reset(version, &git_dir)?;

    let tokens = search_icons(git_dir.join("svg"))?
        .into_iter()
        .map(|(name, path)| {
            let name = syn::Ident::new_raw(&name, proc_macro2::Span::call_site());
            let path = path.to_string_lossy();
            let doc = format!(
                "![{}](file://{})",
                name.to_string().to_title_case(),
                path.replace(" ", "%20")
            );
            quote! {
                #[doc = #doc]
                pub const #name: &'static [u8] = include_bytes!(#path);
            }
        })
        .reduce(|mut a, b| {
            a.extend(b);
            a
        })
        .unwrap_or_default();

    std::fs::write(
        std::path::Path::new(&out_dir).join("icons.rs"),
        tokens.to_string(),
    )?;
    Ok(())
}

fn search_icons<P: AsRef<std::path::Path>>(path: P) -> Result<HashMap<String, std::path::PathBuf>> {
    let mut result = HashMap::new();
    for entry in std::fs::read_dir(path)?.filter_map(Result::ok) {
        if entry.file_type()?.is_dir() {
            result.extend(search_icons(entry.path())?);
            continue;
        }
        if let Some(s) = entry
            .file_name()
            .to_str()
            .and_then(|f| f.strip_suffix(".svg"))
        {
            #[cfg(feature = "snake_case")]
            result.insert(s.to_snake_case(), entry.path());
            #[cfg(feature = "screaming_snake_case")]
            result.insert(s.to_shouty_snake_case(), entry.path());
            #[cfg(feature = "upper_camel_case")]
            result.insert(s.to_upper_camel_case(), entry.path());
            #[cfg(feature = "lower_camel_case")]
            result.insert(s.to_lower_camel_case(), entry.path());
        }
    }
    Ok(result)
}
