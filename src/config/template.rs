pub const CONFIG_TEMPLATE: &str = include_str!("../../assets/config-template.toml");

#[cfg(test)]
mod tests {
    use super::CONFIG_TEMPLATE;
    use crate::config::file::ConfigFile;

    #[test]
    fn template_parses_as_config_file() {
        let parsed: ConfigFile = toml::from_str(CONFIG_TEMPLATE)
            .expect("committed template must parse");
        // We specifically want [engine] to come through so the template stays
        // representative of real overrides.
        let engine = parsed.engine.expect("template should define [engine]");
        assert_eq!(engine.render_throttle_ms, Some(16));
        assert_eq!(engine.tick_interval_ms, Some(250));
    }
}
