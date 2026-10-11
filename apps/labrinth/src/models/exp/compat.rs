//! Compatibility utilities for V3 API.

use crate::models::exp::ProjectSerial;

const MODPACK: &str = "modpack";
const SCHEMATIC: &str = "schematic";
const ENSHROUDED_SERVER: &str = "server";

/// Adjusts V3 project types based on a project's components.
///
/// The experimental API does not have a concept of project types; instead, a
/// project's "type" is implicit based on what components it has.
/// To reflect this in the V3 API, we manually add `project_types` values
/// for compatibility with stuff like searching.
pub fn correct_project_types(
    components: &ProjectSerial,
    project_types: &mut Vec<String>,
) {
    if components.schematic.is_some() {
        project_types.retain(|project_type| project_type != "mod");
        project_types.push(SCHEMATIC.into());
    }

    if components.enshrouded_server.is_some() {
        project_types.retain(|project_type| project_type != MODPACK);
        project_types.push(ENSHROUDED_SERVER.into());
    }
}

#[cfg(test)]
mod tests {
    use super::{ENSHROUDED_SERVER, MODPACK, correct_project_types};
    use crate::models::exp::{
        ProjectSerial,
        enshrouded::{EnshroudedServerProject, SchematicProject},
    };

    #[test]
    fn leaves_project_types_unchanged_without_server_component() {
        let components = ProjectSerial::default();
        let mut project_types = vec!["mod".to_string(), MODPACK.to_string()];
        let expected = project_types.clone();

        correct_project_types(&components, &mut project_types);

        assert_eq!(project_types, expected);
    }

    #[test]
    fn replaces_mod_type_for_schematic_projects() {
        let components = ProjectSerial {
            schematic: Some(SchematicProject {}),
            ..ProjectSerial::default()
        };
        let mut project_types = vec!["mod".to_string()];

        correct_project_types(&components, &mut project_types);

        assert_eq!(project_types, vec!["schematic".to_string()]);
    }

    #[test]
    fn replaces_modpack_type_for_enshrouded_server_projects() {
        let components = ProjectSerial {
            enshrouded_server: Some(EnshroudedServerProject {
                address: "server.shroudedit.com".to_string(),
                query_port: 15637,
                region: Some("europe".to_string()),
                languages: vec!["de".to_string()],
                voice_chat_enabled: false,
                voice_chat_mode: Default::default(),
                text_chat_enabled: false,
                user_groups: Vec::new(),
            }),
            ..ProjectSerial::default()
        };
        let mut project_types = vec![MODPACK.to_string()];

        correct_project_types(&components, &mut project_types);

        assert_eq!(project_types, vec![ENSHROUDED_SERVER.to_string()]);
    }
}
