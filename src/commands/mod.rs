//! Command handlers for cfdns CLI.

mod helpers;
mod init;
mod metadata;
mod query;
mod record;

pub use helpers::{parse_json_data, parse_settings, parse_tags, resolve_record};
pub use init::cmd_init;
pub use metadata::{cmd_comment, cmd_tag};
pub use query::{cmd_count, cmd_get, cmd_list, cmd_search};
pub use record::{cmd_create, cmd_delete, cmd_overwrite, cmd_update};
