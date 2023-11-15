use anyhow::{anyhow, Context, Result};
use clap::{value_parser, Arg, ArgAction};
use file_utils::{prepare_and_check_input_paths, set_extension, translate_to_output_path};
use render::RenderOptions;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

mod diagnostics;
mod dirgraph;
mod dirgraphsvg;
mod file_utils;
mod gsn;
mod render;
mod yaml_fix;

use diagnostics::Diagnostics;
use dirgraphsvg::escape_text;
use gsn::{GsnDocument, GsnNode, Module, ModuleInformation};

const MODULE_INFORMATION_NODE: &str = "module";

///
/// Main entry point.
///
///
fn main() -> Result<()> {
    let app = clap::command!()
        .arg(
            Arg::new("INPUT")
                .help("Sets the input file(s) to use. Only relative paths are accepted.")
                .action(ArgAction::Append)
                .required(true),
        )
        .arg(
            Arg::new("CHECKONLY")
                .help("Only check the input file(s), but do not output graphs.")
                .short('c')
                .long("check")
                .action(ArgAction::SetTrue)
                .help_heading("CHECKS"),
        )
        .arg(
            Arg::new("EXCLUDED_MODULE")
                .help("Exclude this module from reference checks.")
                .short('x')
                .long("exclude")
                .action(ArgAction::Append)
                .help_heading("CHECKS"),
        )
        .arg(
            Arg::new("NO_ARGUMENT_VIEW")
                .help("Do not output of argument view for provided input files.")
                .short('N')
                .long("no-arg")
                .action(ArgAction::SetTrue)
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("COMPLETE_VIEW")
                .help("Output the complete view to file with name <COMPLETE_VIEW>.")
                .short('f')
                .long("full")
                .action(ArgAction::Set)
                .default_value("complete.svg")
                .conflicts_with_all(["CHECKONLY", "NO_COMPLETE_VIEW"])
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("NO_COMPLETE_VIEW")
                .help("Do not output the complete view.")
                .short('F')
                .long("no-full")
                .action(ArgAction::SetTrue)
                .conflicts_with("COMPLETE_VIEW")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("ARCHITECTURE_VIEW")
                .help("Output the architecture view to file with name <ARCHITECTURE_VIEW>.")
                .short('a')
                .long("arch")
                .action(ArgAction::Set)
                .default_value("architecture.svg")
                .conflicts_with_all(["CHECKONLY", "NO_ARCHITECTURE_VIEW"])
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("NO_ARCHITECTURE_VIEW")
                .help("Do not output the architecture view.")
                .short('A')
                .long("no-arch")
                .action(ArgAction::SetTrue)
                .conflicts_with("ARCHITECTURE_VIEW")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("EVIDENCES")
                .help("Output list of all evidences to file with name <EVIDENCES>.")
                .short('e')
                .long("evidences")
                .action(ArgAction::Set)
                .default_value("evidences.md")
                .conflicts_with_all(["CHECKONLY", "NO_EVIDENCES"])
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("NO_EVIDENCES")
                .help("Do not output list of all evidences.")
                .short('E')
                .long("no-evidences")
                .action(ArgAction::SetTrue)
                .conflicts_with("EVIDENCES")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("OUTPUT_DIRECTORY")
                .help("Emit all output files to directory <OUTPUT_DIRECTORY>.")
                .short('o')
                .long("output-dir")
                .action(ArgAction::Set)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("LAYERS")
                .help("Output additional layer. Can be used multiple times.")
                .short('l')
                .long("layer")
                .action(ArgAction::Append)
                .use_value_delimiter(true)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("STYLESHEETS")
                .help("Links a stylesheet in SVG output. Can be used multiple times.")
                .short('s')
                .long("stylesheet")
                .action(ArgAction::Append)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("EMBED_CSS")
                .help("Embed stylehseets instead of linking them.")
                .short('t')
                .long("embed-css")
                .action(ArgAction::SetTrue)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        // .arg(
        //     Arg::new("MASK_MODULE")
        //         .help("Do not unroll this module in the complete view.")
        //         .short('m')
        //         .long("mask")
        //         .multiple_occurrences(true)
        //         .takes_value(true)
        //         .requires("COMPLETE_VIEW")
        //         .help_heading("OUTPUT MODIFICATION"),
        // )
        .arg(
            Arg::new("NO_LEGEND")
                .help("Do not output a legend based on module information.")
                .short('G')
                .long("no-legend")
                .action(ArgAction::SetTrue)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("FULL_LEGEND")
                .help("Output a legend based on all module information.")
                .short('g')
                .long("full-legend")
                .action(ArgAction::SetTrue)
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("WORD_WRAP")
                .help("Define the number of characters after which a line of text is wrapped.")
                .short('w')
                .long("wrap")
                .action(ArgAction::Set)
                .value_parser(value_parser!(u32))
                .conflicts_with("CHECKONLY")
                .help_heading("OUTPUT MODIFICATION"),
        );
    let matches = app.get_matches();
    let mut inputs: Vec<String> = matches
        .get_many::<String>("INPUT")
        .into_iter()
        .flatten()
        .cloned()
        .collect();
    let layers = matches
        .get_many::<String>("LAYERS")
        .into_iter()
        .flatten()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>();
    let excluded_modules = matches
        .get_many::<String>("EXCLUDED_MODULE")
        .into_iter()
        .flatten()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>();

    let mut diags = Diagnostics::default();
    let mut nodes = BTreeMap::<String, GsnNode>::new();

    // Module name to module mapping
    let mut modules: HashMap<String, Module> = HashMap::new();

    // Read input
    let common_ancestors = prepare_and_check_input_paths(&mut inputs)?;
    read_inputs(&inputs, &mut nodes, &mut modules, &mut diags)?;

    // Validate
    validate_and_check(&mut nodes, &modules, &mut diags, &excluded_modules, &layers);

    if diags.errors == 0 && !matches.get_flag("CHECKONLY") {
        let embed_stylesheets = matches.get_flag("EMBED_CSS");
        let output_directory = matches.get_one::<String>("OUTPUT_DIRECTORY");
        let mut stylesheets = matches
            .get_many::<String>("STYLESHEETS")
            .into_iter()
            .flatten()
            .map(|css| css.to_owned())
            .collect::<Vec<_>>();
        // Copy stylesheets if necessary and prepare paths
        copy_and_prepare_stylesheets(&mut stylesheets, embed_stylesheets, &output_directory)?;
        let render_options =
            RenderOptions::new(&matches, stylesheets, embed_stylesheets, output_directory);
        // Output views
        print_outputs(nodes, &modules, &render_options, common_ancestors)?;
    }
    // Output diagnostic messages
    output_messages(&diags)
}

///
/// Read inputs
///
///
fn read_inputs(
    inputs: &[String],
    nodes: &mut BTreeMap<String, GsnNode>,
    modules: &mut HashMap<String, Module>,
    diags: &mut Diagnostics,
) -> Result<()> {
    for input in inputs {
        let reader =
            BufReader::new(File::open(input).context(format!("Failed to open file {input}"))?);

        let mut n: BTreeMap<String, GsnDocument> = serde_yaml::from_reader(reader)
            .map(|n: yaml_fix::YamlFixMap<String, GsnDocument>| n.into_inner())
            .map_err(|e| {
                anyhow!(format!(
                    "No valid GSN element can be found starting from line {}.\n\
                     This typically means that the YAML is completely invalid, or \n\
                     the `text:` attribute is missing for an element.\n\
                     Original error message: {}.",
                    e.location()
                        .map(|e| e.line().to_string())
                        .unwrap_or_else(|| "unknown".to_owned()),
                    e
                ))
            })
            .context(format!("Failed to parse YAML from file {input}"))?;
        let meta: ModuleInformation = match n.remove_entry(MODULE_INFORMATION_NODE) {
            Some((_, GsnDocument::ModuleInformation(x))) => x,
            _ => {
                let module_name = escape_text(input);
                ModuleInformation {
                    name: module_name.to_owned(),
                    brief: None,
                    extends: None,
                    additional: BTreeMap::new(),
                }
            }
        };
        // Add filename and module name to module list
        let module = meta.name.to_owned();

        if let std::collections::hash_map::Entry::Vacant(e) = modules.entry(module.to_owned()) {
            e.insert(Module {
                relative_module_path: input.to_owned().to_owned(),
                meta,
            });
        } else {
            diags.add_error(
                Some(&module),
                format!(
                    "C06: Module name {} in {} was already present in {}.",
                    module,
                    input,
                    modules.get(&module).unwrap().relative_module_path // unwrap is ok, otherwise Entry would not have been Vacant
                ),
            );
        }

        // Check for duplicates, since they might be in separate files.
        let node_names: Vec<String> = n.keys().cloned().collect();
        for node_name in node_names {
            if let Some((k, v)) = n.remove_entry(&node_name) {
                if let std::collections::btree_map::Entry::Vacant(e) = nodes.entry(k.to_owned()) {
                    match v {
                        GsnDocument::GsnNode(mut x) => {
                            // Remember module for node
                            x.module = module.to_owned();
                            x.fix_node_type(&k);
                            e.insert(x);
                        }
                        _ => unreachable!(), // There can be only one MetaNode
                    }
                } else {
                    diags.add_error(
                        Some(&module),
                        format!(
                            "C07: Element {} in {} was already present in {}.",
                            k,
                            input,
                            nodes.get(&k).unwrap().module // unwrap is ok, otherwise Entry would not have been Vacant
                        ),
                    );
                    break;
                }
            }
        }
    }
    if nodes.is_empty() {
        Err(anyhow!("No input elements found."))
    } else {
        Ok(())
    }
}

///
/// Validate and check modules
///
///
///
///
fn validate_and_check(
    nodes: &mut BTreeMap<String, GsnNode>,
    modules: &HashMap<String, Module>,
    diags: &mut Diagnostics,
    excluded_modules: &[&str],
    layers: &Vec<&str>,
) {
    for (module_name, module_info) in modules {
        // Validation for well-formedness is done unconditionally.
        gsn::validation::validate_module(diags, module_name, module_info, nodes);
        if diags.errors > 0 {
            break;
        }
    }
    if diags.errors == 0 {
        gsn::extend_modules(diags, nodes, modules);
        gsn::check::check_nodes(diags, nodes, excluded_modules);
        if !layers.is_empty() {
            gsn::check::check_layers(diags, nodes, layers);
        }
    }
}

///
/// Print outputs
///
///
///
///
fn print_outputs(
    nodes: BTreeMap<String, GsnNode>,
    modules: &HashMap<String, Module>,
    render_options: &RenderOptions,
    common_ancestors: String,
) -> Result<()> {
    let output_path = render_options
        .output_directory
        .to_owned()
        .unwrap_or(".".to_owned());
    if !render_options.skip_argument {
        for (module_name, module) in modules {
            let output_path = set_extension(
                &translate_to_output_path(&output_path, &module.relative_module_path, None)?,
                "svg",
            );
            let mut output_file = Box::new(
                File::create(&output_path)
                    .context(format!("Failed to open output file {output_path}"))?,
            ) as Box<dyn std::io::Write>;

            render::render_argument(
                &mut output_file,
                module_name,
                modules,
                &nodes,
                render_options,
            )?;
        }
    }
    if modules.len() > 1 {
        if let Some(architecture_filename) = &render_options.architecture_filename {
            let arch_output_path = translate_to_output_path(
                &output_path,
                architecture_filename,
                Some(&common_ancestors),
            )?;
            let mut output_file = File::create(&arch_output_path)
                .context(format!("Failed to open output file {arch_output_path}"))?;
            let deps = crate::gsn::calculate_module_dependencies(&nodes);
            render::render_architecture(
                &mut output_file,
                modules,
                deps,
                render_options,
                &arch_output_path,
                &output_path,
            )?;
        }
        if let Some(complete_filename) = &render_options.complete_filename {
            let output_path =
                translate_to_output_path(&output_path, complete_filename, Some(&common_ancestors))?;
            let mut output_file = File::create(&output_path)
                .context(format!("Failed to open output file {output_path}"))?;
            render::render_complete(&mut output_file, &nodes, render_options)?;
        }
    }
    if let Some(evidences_filename) = &render_options.evidences_filename {
        let output_path =
            translate_to_output_path(&output_path, evidences_filename, Some(&common_ancestors))?;
        let mut output_file = File::create(&output_path)
            .context(format!("Failed to open output file {output_path}"))?;
        render::render_evidences(&mut output_file, &nodes, render_options)?;
    }
    Ok(())
}

///
/// Render to dot-file if not only validation is active.
/// Output summary of warnings and errors.
///
fn output_messages(diags: &Diagnostics) -> Result<()> {
    for msg in &diags.messages {
        eprintln!("{msg}");
    }
    if diags.errors == 0 {
        if diags.warnings > 0 {
            eprintln!("Warning: {} warnings detected.", diags.warnings);
        }
        Ok(())
    } else {
        Err(anyhow!(
            "{} errors and {} warnings detected.",
            diags.errors,
            diags.warnings
        ))
    }
}

///
///
///
///
pub(crate) fn copy_and_prepare_stylesheets(
    stylesheets: &mut [String],
    embed_stylesheets: bool,
    output_directory: &Option<&String>,
) -> Result<()> {
    for stylesheet in stylesheets {
        let new_name = if stylesheet.starts_with("http://")
            || stylesheet.starts_with("https://")
            || stylesheet.starts_with("file://")
        {
            // Stylesheets provided as a URL, are neither copied nor embedded
            format!("url({stylesheet})")
        } else if embed_stylesheets {
            // No need to transform path when embedding stylesheets
            stylesheet.to_owned()
        } else if let Some(output_directory) = output_directory {
            // Copy stylesheet to output path
            let css_path = PathBuf::from(&stylesheet);
            let mut out_path = PathBuf::from(output_directory);
            std::fs::create_dir_all(&out_path)?;
            out_path.push(css_path.file_name().ok_or(anyhow!(
                "Could not identify stylesheet filename in {}",
                stylesheet
            ))?);
            std::fs::copy(&css_path, &out_path).with_context(|| {
                format!(
                    "Could not copy stylesheet from {} to {}",
                    css_path.display(),
                    &out_path.display()
                )
            })?;
            out_path.to_string_lossy().to_string().to_owned()
        } else {
            stylesheet.to_owned()
        };
        *stylesheet = new_name.to_owned();
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use crate::diagnostics::Diagnostics;

    #[test]
    fn check_output_messages_errors() {
        let d = Diagnostics {
            warnings: 2,
            errors: 3,
            ..Default::default()
        };
        let res = crate::output_messages(&d);
        assert!(res.is_err());
        assert_eq!(
            format!("{:?}", res),
            "Err(3 errors and 2 warnings detected.)"
        );
    }

    #[test]
    fn check_output_messages_warnings() {
        let d = Diagnostics {
            warnings: 5,
            errors: 0,
            ..Default::default()
        };
        let res = crate::output_messages(&d);
        assert!(res.is_ok());
        assert_eq!(format!("{:?}", res), "Ok(())");
    }

    #[test]
    fn check_output_messages_no() {
        let d = Diagnostics {
            warnings: 0,
            errors: 0,
            ..Default::default()
        };
        let res = crate::output_messages(&d);
        assert!(res.is_ok());
        assert_eq!(format!("{:?}", res), "Ok(())");
    }
}
