mod convert;
mod parse;

pub fn convert(
    config: &crate::config::Config,
    input: &std::path::Path,
    output: &std::path::Path,
    is_appending: bool,
    stedsnavn_gml: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    convert::convert_all(config, input, output, is_appending, stedsnavn_gml)
}
