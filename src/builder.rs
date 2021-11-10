use crate::handler::SlashCommandEntry;
use crate::SlashCommandCallback;
use serenity::builder::{CreateApplicationCommand, CreateApplicationCommandPermissionsData};

macro_rules! builder_fn {
    ($name:ident : $ty:ty) => {
        pub fn $name(&mut self, $name: $ty) -> &mut Self {
            self.$name = Some($name);
            self
        }
    };
}

#[derive(Default)]
pub struct SlashCommandBuilder {
    name: Option<&'static str>,
    guilds: Option<&'static [u64]>,
    callback: Option<SlashCommandCallback>,
    create: Option<CreateApplicationCommand>,
    permissions: Option<CreateApplicationCommandPermissionsData>,
}

impl SlashCommandBuilder {
    builder_fn!(name: &'static str);
    builder_fn!(guilds: &'static [u64]);
    builder_fn!(callback: SlashCommandCallback);

    pub fn create_application_command<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateApplicationCommand) -> &mut CreateApplicationCommand,
    {
        let mut create = Default::default();
        f(&mut create);
        self.create = Some(create);
        self
    }

    pub fn create_permissions<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(
            &mut CreateApplicationCommandPermissionsData,
        ) -> &mut CreateApplicationCommandPermissionsData,
    {
        let mut permissions = Default::default();
        f(&mut permissions);
        self.permissions = Some(permissions);
        self
    }

    pub fn build(self) -> (SlashCommandEntry, SlashCommandCallback) {
        macro_rules! check_uninit {
            ($name:ident) => {
                match self.$name {
                    Some($name) => $name,
                    None => panic!(concat!(
                        "'",
                        stringify!($name),
                        "' was not initialized in slash command builder"
                    )),
                }
            };
        }

        let name = check_uninit!(name);
        let guilds = self.guilds;
        let callback = check_uninit!(callback);
        let create = check_uninit!(create);
        let permissions = check_uninit!(permissions);

        (
            SlashCommandEntry {
                name,
                guilds,
                create,
                permissions,
            },
            callback,
        )
    }
}
