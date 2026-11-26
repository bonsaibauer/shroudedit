//! Compatibility utilities for V3 API.

use crate::models::exp::ProjectSerial;

const MODPACK: &str = "modpack";
const MOD: &str = "mod";
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
	if components.enshrouded_server.is_some() {
		project_types.retain(|project_type| {
			!matches!(project_type.as_str(), MOD | MODPACK | SCHEMATIC | ENSHROUDED_SERVER)
		});
		project_types.push(ENSHROUDED_SERVER.into());
	} else if components.schematic.is_some() {
		project_types.retain(|project_type| {
			!matches!(project_type.as_str(), MOD | MODPACK | SCHEMATIC)
		});
		project_types.push(SCHEMATIC.into());
	} else if components.enshrouded_mod.is_some() {
		project_types.retain(|project_type| {
			!matches!(project_type.as_str(), MOD | MODPACK | SCHEMATIC)
		});
		project_types.push(MOD.into());
	} else if project_types.iter().any(|project_type| project_type == MODPACK) {
		project_types.retain(|project_type| !matches!(project_type.as_str(), MOD | SCHEMATIC));
	} else {
		project_types.retain(|project_type| project_type != SCHEMATIC);
	}
}

#[cfg(test)]
mod tests {
	use super::{ENSHROUDED_SERVER, MOD, MODPACK, SCHEMATIC, correct_project_types};
	use crate::models::exp::{
		ProjectSerial,
		enshrouded::{
			EnshroudedServerProject, EnshroudedVoiceChatMode, ModProject,
			SchematicProject,
		},
	};

	#[test]
	fn removes_schematic_type_from_mod_projects() {
		let components = ProjectSerial::default();
		let mut project_types = vec![MOD.to_string(), SCHEMATIC.to_string()];

		correct_project_types(&components, &mut project_types);

		assert_eq!(project_types, vec![MOD.to_string()]);
	}

	#[test]
	fn keeps_modpack_projects_separate_from_mods_and_schematics() {
		let components = ProjectSerial::default();
		let mut project_types = vec![MOD.to_string(), MODPACK.to_string(), SCHEMATIC.to_string()];

		correct_project_types(&components, &mut project_types);

		assert_eq!(project_types, vec![MODPACK.to_string()]);
	}

	#[test]
	fn keeps_explicit_mod_projects_separate_from_modpacks_and_schematics() {
		let components = ProjectSerial {
			enshrouded_mod: Some(ModProject {}),
			..ProjectSerial::default()
		};
		let mut project_types = vec![MODPACK.to_string(), SCHEMATIC.to_string()];

		correct_project_types(&components, &mut project_types);

		assert_eq!(project_types, vec![MOD.to_string()]);
	}

	#[test]
	fn replaces_mod_type_for_schematic_projects() {
		let components = ProjectSerial {
			schematic: Some(SchematicProject {}),
			..ProjectSerial::default()
		};
		let mut project_types = vec![MOD.to_string(), MODPACK.to_string()];

		correct_project_types(&components, &mut project_types);

		assert_eq!(project_types, vec![SCHEMATIC.to_string()]);
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
				voice_chat_mode: EnshroudedVoiceChatMode::default(),
				text_chat_enabled: false,
				user_groups: Vec::new(),
			}),
			..ProjectSerial::default()
		};
		let mut project_types = vec![MOD.to_string(), MODPACK.to_string(), SCHEMATIC.to_string()];

		correct_project_types(&components, &mut project_types);

		assert_eq!(project_types, vec![ENSHROUDED_SERVER.to_string()]);
	}
}
