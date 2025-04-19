use anyhow::{Context, Result, anyhow};
use clap::parser::ValueSource;
use clap::{Arg, ArgAction, Command, value_parser};
use file_utils::translate_to_output_path;
use render::RenderOptions;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Display;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::{collections::btree_map::Entry, fs::File};

mod diagnostics;
mod dirgraph;
mod dirgraphsvg;
mod file_utils;
mod gsn;
mod outputs;
mod render;
mod yaml_fix;

use diagnostics::Diagnostics;
use dirgraphsvg::escape_text;
use gsn::{FindModuleByPath, GsnDocument, GsnNode, Module, ModuleInformation, Origin};

const MODULE_INFORMATION_NODE: &str = "module";

///
/// Main entry point.
///
///
fn main() -> Result<()> {
    let mut command = build_command_options();
    let matches = command.clone().get_matches();

    let mut diags = Diagnostics::default();

    let inputs: Vec<String> = matches
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
    if matches.value_source("INPUT") == Some(ValueSource::DefaultValue)
        && !Path::new(inputs.first().unwrap()).exists()
    // unwrap ok, since default provided
    {
        command.print_help()?;
        Err(anyhow!("index.gsn.yaml not found."))
    } else {
        // unwrap ok, since default value provided.
        let output_directory = matches.get_one::<String>("OUTPUT_DIRECTORY").unwrap();

        let mut nodes = BTreeMap::<String, GsnNode>::new();

        // Module name to module mapping
        let mut modules: BTreeMap<String, Module> = BTreeMap::new();

        // Closure is important here, otherwise main is left with ? operator
        let read_and_check = || -> Result<()> {
            read_inputs(
                &inputs,
                &mut nodes,
                &mut modules,
                &mut diags,
                output_directory,
            )?;
            // Validate
            validate_and_check(&mut nodes, &modules, &mut diags, &excluded_modules, &layers)
        }();
        // Ignore error, if errors are found, this is handled in output_messages
        match read_and_check {
            Err(e) if e.is::<ValidationOrCheckError>() => Ok(()),
            Err(e) => Err(e),
            Ok(_) => {
                if !matches.get_flag("CHECK_ONLY") {
                    // Create output directory
                    if !std::path::Path::new(&output_directory).exists() {
                        std::fs::create_dir_all(output_directory).with_context(|| {
                            format!("Could not create output directory {output_directory}")
                        })?;
                    }
                    let embed_stylesheets = matches.get_flag("EMBED_CSS");
                    let mut stylesheets = matches
                        .get_many::<String>("STYLESHEETS")
                        .into_iter()
                        .flatten()
                        .cloned()
                        .collect::<Vec<_>>();
                    // Append stylesheets from modules
                    stylesheets.append(
                        &mut modules
                            .iter()
                            .flat_map(|m| m.1.meta.stylesheets.to_owned())
                            .collect::<Vec<_>>(),
                    );
                    // Copy stylesheets if necessary and prepare paths
                    copy_and_prepare_stylesheets(
                        &mut stylesheets,
                        embed_stylesheets,
                        output_directory,
                    )?;
                    let mut render_options = RenderOptions::new(
                        &matches,
                        stylesheets,
                        embed_stylesheets,
                        output_directory,
                    );
                    // Add missing nodes that may not exist because references checks have been excluded
                    add_missing_nodes_and_modules(&mut nodes, &mut modules, &mut render_options);
                    // Output views
                    print_outputs(&nodes, &modules, &render_options)?;
                }
                if matches.get_flag("STATISTICS") {
                    outputs::render_statistics(&nodes, &modules);
                }
                if let Some(yaml_dir) = matches.get_one::<String>("YAMLDUMP") {
                    outputs::render_yaml_docs(&nodes, &modules, yaml_dir)?;
                }
                Ok(())
            }
        }?;

        // Output diagnostic messages
        output_messages(&diags)
    }
}

#[derive(PartialEq, Debug)]
struct ValidationOrCheckError {}

impl Display for ValidationOrCheckError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

impl Error for ValidationOrCheckError {}

///
/// Create clap command line arguments
///
///
fn build_command_options() -> Command {
    clap::command!()
        .arg(
            Arg::new("INPUT")
                .help("Sets the input file(s) to use.")
                .action(ArgAction::Append)
                .default_values(["index.gsn.yaml"]),
        )
        .arg(
            Arg::new("CHECK_ONLY")
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
                .conflicts_with_all(["CHECK_ONLY", "NO_COMPLETE_VIEW"])
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
                .conflicts_with_all(["CHECK_ONLY", "NO_ARCHITECTURE_VIEW"])
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
            Arg::new("EVIDENCE")
                .help("Output list of all evidence to file with name <EVIDENCE>.")
                .short('e')
                .long("evidence")
                .action(ArgAction::Set)
                .default_value("evidence.md")
                .conflicts_with_all(["CHECK_ONLY", "NO_EVIDENCE"])
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("NO_EVIDENCE")
                .help("Do not output list of all evidence.")
                .short('E')
                .long("no-evidence")
                .action(ArgAction::SetTrue)
                .conflicts_with("EVIDENCE")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("OUTPUT_DIRECTORY")
                .help("Emit all output files to directory <OUTPUT_DIRECTORY>.")
                .short('o')
                .long("output-dir")
                .action(ArgAction::Set)
                .conflicts_with("CHECK_ONLY")
                .default_value(".")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("STATISTICS")
                .help("Output statistics on inputs.")
                .long("statistics")
                .action(ArgAction::SetTrue)
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("YAMLDUMP")
                .help("Output parsed YAML files to single file <YAMLFILE>.")
                .long("restructure-yaml")
                .action(ArgAction::Set)
                .default_value("gsn2x_restructured.yaml")
                .help_heading("OUTPUT"),
        )
        .arg(
            Arg::new("LAYERS")
                .help("Output additional layer. Can be used multiple times.")
                .short('l')
                .long("layer")
                .action(ArgAction::Append)
                .use_value_delimiter(true)
                .conflicts_with("CHECK_ONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("STYLESHEETS")
                .help("Links a stylesheet in SVG output. Can be used multiple times.")
                .short('s')
                .long("stylesheet")
                .action(ArgAction::Append)
                .conflicts_with("CHECK_ONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("EMBED_CSS")
                .help("Embed stylesheets instead of linking them.")
                .short('t')
                .long("embed-css")
                .action(ArgAction::SetTrue)
                .conflicts_with("CHECK_ONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("MASKED_MODULE")
                .help("Do not show this module in views.")
                .short('m')
                .long("mask")
                .action(ArgAction::Append)
                .conflicts_with("CHECK_ONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("NO_LEGEND")
                .help("Do not output a legend based on module information.")
                .short('G')
                .long("no-legend")
                .action(ArgAction::SetTrue)
                .conflicts_with("CHECK_ONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("FULL_LEGEND")
                .help("Output a legend based on all module information.")
                .short('g')
                .long("full-legend")
                .action(ArgAction::SetTrue)
                .conflicts_with("CHECK_ONLY")
                .help_heading("OUTPUT MODIFICATION"),
        )
        .arg(
            Arg::new("CHAR_WRAP")
                .help("Define the number of characters after which a line of text is wrapped.")
                .short('w')
                .long("wrap")
                .action(ArgAction::Set)
                .value_parser(value_parser!(u32))
                .conflicts_with("CHECK_ONLY")
                .help_heading("OUTPUT MODIFICATION"),
            // Intentionally no default value, to allow formatting via YAML.
        )
}

///
/// Add missing nodes that were referenced, but excluded f
///
fn add_missing_nodes_and_modules(
    nodes: &mut BTreeMap<String, GsnNode>,
    modules: &mut BTreeMap<String, Module>,
    render_options: &mut RenderOptions,
) {
    let mut add_nodes = vec![];
    for (_, node) in nodes.iter() {
        let ref_nodes: Vec<_> = node
            .supported_by
            .iter()
            .chain(node.in_context_of.iter())
            .collect();
        for ref_node in ref_nodes {
            if !nodes.contains_key(ref_node) {
                add_nodes.push(ref_node.to_owned());
            }
        }
    }
    for node in add_nodes {
        let mut gsn_node = GsnNode {
            module: "Unknown".to_owned(),
            ..Default::default()
        };
        gsn_node.fix_node_type(&node);
        nodes.insert(node.to_owned(), gsn_node);
        render_options.masked_elements.push(node);
    }
    let _ = modules.insert(
        "Unknown".to_owned(),
        Module {
            orig_file_name: "".to_owned(),
            meta: ModuleInformation::new("Unknown".to_owned()),
            origin: Origin::Excluded,
            canonical_path: None,
            output_path: None,
        },
    );
}

///
/// Read inputs
///
///
fn read_inputs(
    inputs: &[String],
    nodes: &mut BTreeMap<String, GsnNode>,
    modules: &mut BTreeMap<String, Module>,
    diags: &mut Diagnostics,
    output_directory: &str,
) -> Result<()> {
    let mut copied_inputs: Vec<String> = inputs.iter().map(|i| (i.replace('\\', "/"))).collect();
    let mut first_run = true;
    'outer: loop {
        let mut additional_inputs = vec![];
        for input in &copied_inputs {
            let reader =
                BufReader::new(File::open(input).context(format!("Failed to open file {input}"))?);

            let mut n: BTreeMap<String, GsnDocument> = serde_yml::from_reader(reader)
            .map(|n: yaml_fix::YamlFixMap<String, GsnDocument>| n.into_inner())
            .map_err(|e| {
                anyhow!(format!(
                    "No valid GSN element can be found starting from line {}.\n\
                     This typically means that the YAML is completely invalid or \n\
                     the `text:` attribute is missing for an element.\n\
                     Please see the documentation for details (https://jonasthewolf.github.io/gsn2x/troubleshooting.html).\n\
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
                    let module_name = escape_text(&input.to_owned());
                    ModuleInformation::new(module_name)
                }
            };

            // Add filename and module name to module list
            let module = meta.name.to_owned();
            let pb = PathBuf::from(input)
                .canonicalize()
                .with_context(|| format!("Failed to open file {input}."))?;
            let module_name_exists = modules.find_module_by_path(&pb).is_some();
            // Check for duplicate module name
            match modules.entry(module.to_owned()) {
                Entry::Vacant(e) if !module_name_exists => {
                    e.insert(Module {
                        orig_file_name: input.to_owned().to_owned(),
                        meta: meta.clone(),
                        origin: if first_run {
                            Origin::CommandLine
                        } else {
                            Origin::File(input.to_owned())
                        },
                        canonical_path: Some(pb),
                        output_path: translate_to_output_path(output_directory, input, Some("svg"))
                            .ok(),
                    });
                    check_and_add_nodes(n, nodes, &module, diags, input, meta.char_wrap);
                    // Remember additional files to read
                    let imported_files = get_uses_files(&meta, input, diags);
                    additional_inputs.extend(imported_files.to_vec());
                }
                Entry::Vacant(_) => {
                    unreachable!()
                }
                Entry::Occupied(e) => {
                    diags.add_error(
                        Some(&module),
                        format!(
                            "C06: Module in {} was already present in {} provided by {}.",
                            input,
                            e.get().orig_file_name,
                            e.get().origin,
                        ),
                    );
                    // A circle may be detected, conservatively bail out completely.
                    break 'outer Err(ValidationOrCheckError {}.into());
                }
            }
        }
        if additional_inputs.is_empty() {
            break Ok(());
        } else {
            copied_inputs.clear();
            copied_inputs.append(&mut additional_inputs);
        }
        first_run = false;
    }
}

///
/// Get files that are marked as "uses" by current module.
///
///
fn get_uses_files(
    meta: &ModuleInformation,
    input: &String,
    diags: &mut Diagnostics,
) -> Vec<String> {
    let imported_files: Vec<String> = meta
        .uses
        .iter()
        .filter_map(|r| match PathBuf::from(r) {
            x if x.is_relative() => PathBuf::from(input).parent().map(|p| {
                let mut new_r = p.to_path_buf();
                new_r.push(r);
                new_r.to_string_lossy().to_string()
            }),
            x if x.is_absolute() => Some(r.to_owned()),
            _ => {
                diags.add_warning(
                    Some(&meta.name),
                    format!("Could not identify used file {r} in module; ignoring it."),
                );
                None
            }
        })
        .map(|i| i.replace('\\', "/"))
        .collect();
    imported_files
}

///
/// Check and potentially add nodes of new module
///
///
fn check_and_add_nodes(
    mut n: BTreeMap<String, GsnDocument>,
    nodes: &mut BTreeMap<String, GsnNode>,
    module: &String,
    diags: &mut Diagnostics,
    input: &String,
    char_wrap: Option<u32>,
) {
    // Check for duplicates, since they might be in separate files.
    let node_names: Vec<String> = n.keys().cloned().collect();
    for node_name in node_names {
        if let Some((k, v)) = n.remove_entry(&node_name) {
            match nodes.entry(k.to_owned()) {
                Entry::Vacant(e) => match v {
                    GsnDocument::GsnNode(mut x) => {
                        // Remember module for node
                        module.clone_into(&mut x.module);
                        x.fix_node_type(&k);
                        // Sort all edges lexicographically
                        x.supported_by.sort();
                        x.in_context_of.sort();
                        x.challenges.sort();
                        if x.char_wrap.is_none() {
                            x.char_wrap = char_wrap;
                        }
                        e.insert(x);
                    }
                    _ => unreachable!(), // There can be only one MetaNode
                },
                Entry::Occupied(e) => {
                    diags.add_error(
                        Some(module),
                        format!(
                            "C07: Element {} in {} was already present in {}.",
                            k,
                            input,
                            e.get().module,
                        ),
                    );
                    break;
                }
            }
        }
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
    modules: &BTreeMap<String, Module>,
    diags: &mut Diagnostics,
    excluded_modules: &[&str],
    layers: &[&str],
) -> Result<()> {
    // Compiler complains if this is not a closure, but a simple block
    if nodes.is_empty() {
        diags.add_error(None, "No input elements are found.".to_owned());
        Err(ValidationOrCheckError {}.into())
    } else if let Some(empty_modules) = {
        let empty_modules = modules
            .keys()
            .filter(|m| nodes.values().filter(|n| &&n.module == m).count() == 0)
            .cloned()
            .collect::<Vec<_>>();
        if empty_modules.is_empty() {
            None
        } else {
            Some(empty_modules)
        }
    } {
        for empty_module in empty_modules {
            diags.add_error(
                Some(&empty_module),
                "The module does not contain elements.".to_owned(),
            );
        }
        Err(ValidationOrCheckError {}.into())
    } else {
        let result = || -> Result<(), ()> {
            for module_info in modules.values() {
                // Validation for well-formedness is done unconditionally.
                gsn::validation::validate_module(
                    diags,
                    &module_info.meta.name,
                    module_info,
                    nodes,
                )?;
            }
            gsn::extend_modules(diags, nodes, modules)?;
            gsn::check::check_nodes(diags, nodes, excluded_modules)?;
            gsn::check::check_layers(diags, nodes, layers)
        }();
        result.map_err(|_| ValidationOrCheckError {}.into())
    }
}

///
/// Print outputs
///
///
///
///
fn print_outputs(
    nodes: &BTreeMap<String, GsnNode>,
    modules: &BTreeMap<String, Module>,
    render_options: &RenderOptions,
) -> Result<()> {
    let output_path = render_options.output_directory.to_owned();
    if !render_options.skip_argument {
        for (_, module) in modules.iter().filter(|(m, _)| *m != "Unknown") {
            let output_path = Path::new(module.output_path.as_ref().unwrap()); // unwrap ok, since we set it for each module.
            if !&output_path.parent().unwrap().exists() {
                // Create output directory; unwraps are ok, since file always have a parent
                std::fs::create_dir_all(output_path.parent().unwrap()).with_context(|| {
                    format!(
                        "Could not create directory {} for {}",
                        output_path.display(),
                        module.orig_file_name
                    )
                })?;
            }
            let mut output_file = Box::new(File::create(output_path).context(format!(
                "Failed to open output file {}",
                output_path.display()
            ))?) as Box<dyn std::io::Write>;

            print!("Rendering \"{}\": ", output_path.display());
            render::render_argument(
                &mut output_file,
                &module.meta.name,
                modules,
                nodes,
                render_options,
            )?;
        }
    }
    // Output directory is already created. No need to add that.
    if modules.iter().filter(|(m, _)| *m != "Unknown").count() > 1 {
        if let Some(architecture_filename) = &render_options.architecture_filename {
            let arch_output_path =
                translate_to_output_path(&output_path, architecture_filename, None)?;
            let mut output_file = File::create(&arch_output_path)
                .context(format!("Failed to open output file {arch_output_path}"))?;
            let dependencies = crate::gsn::calculate_module_dependencies(nodes);
            print!("Rendering \"{arch_output_path}\": ");
            render::render_architecture(
                &mut output_file,
                modules,
                dependencies,
                render_options,
                &arch_output_path,
            )?;
        }
        if let Some(complete_filename) = &render_options.complete_filename {
            let output_path = translate_to_output_path(&output_path, complete_filename, None)?;
            let mut output_file = File::create(&output_path)
                .context(format!("Failed to open output file {output_path}"))?;
            print!("Rendering \"{output_path}\": ");
            render::render_complete(&mut output_file, nodes, render_options)?;
        }
    }
    if let Some(evidence_filename) = &render_options.evidence_filename {
        let output_path = translate_to_output_path(&output_path, evidence_filename, None)?;
        let mut output_file = File::create(&output_path)
            .context(format!("Failed to open output file {output_path}"))?;
        print!("Writing evidence \"{output_path}\": ");
        outputs::render_evidence(&mut output_file, nodes, render_options)?;
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
/// Copy the stylesheets if they need to be in the output directory
/// If they actually reference a URL, make the stylesheet reference a url.
/// Don't do anything if the stylesheet is anyway embedded.
///
pub(crate) fn copy_and_prepare_stylesheets(
    stylesheets: &mut [String],
    embed_stylesheets: bool,
    output_directory: &str,
) -> Result<()> {
    for stylesheet in stylesheets {
        let new_name = if file_utils::is_url(stylesheet) {
            // Stylesheets provided as a URL, are neither copied nor embedded
            format!("url({stylesheet})")
        } else if embed_stylesheets {
            // No need to transform path when embedding stylesheets
            stylesheet.to_owned()
        } else {
            // Copy stylesheet to output path

            let css_path = PathBuf::from(&stylesheet).canonicalize()?;
            let mut out_path = PathBuf::from(output_directory).canonicalize()?;
            out_path.push(css_path.file_name().ok_or(anyhow!(
                "Could not identify stylesheet filename in {}",
                stylesheet
            ))?);
            if css_path != out_path {
                std::fs::copy(&css_path, &out_path).with_context(|| {
                    format!(
                        "Could not copy stylesheet from {} to {}",
                        css_path.display(),
                        &out_path.display()
                    )
                })?;
            }
            out_path.to_string_lossy().to_string().to_owned()
        };
        new_name.clone_into(stylesheet);
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
