///
/// Escape module name
///
/// Remove espcially the "."'s, since the module name is used in the template as a key for a map.
/// However, Tera cannot cope with that. The dot is interpreted as a separator for attributes.
///
pub fn escape_module_name(input: &&str) -> String {
    input
        .replace(".", "_")
        .replace("-", "_")
        .replace(" ", "_")
        .replace("/", "_")
        .replace("\\", "_")
        .replace(":", "_")
}
