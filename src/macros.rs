#[macro_export]
macro_rules! slash_command_permissions {
    ($name:ident: $allow:literal for User($id:literal)) => {
        slash_command_permissions!(@INNER $name: User ($id): $allow);
    };
    ($name:ident: $allow:literal for Role($id:literal)) => {
        slash_command_permissions!(@INNER $name: Role ($id): $allow);
    };
    (@INNER $name:ident: $variant:ident ($id:literal): $allow:literal) => {
        #[allow(non_camel_case_types)]
        struct $name;

        impl $name {
            pub fn apply(p: &mut ::serenity::builder::CreateApplicationCommandPermissionsData) -> &mut ::serenity::builder::CreateApplicationCommandPermissionsData {
                p.create_permission(|p| {
                    p.kind(::serenity::model::interactions::application_command::ApplicationCommandPermissionType::$variant)
                        .id($id)
                        .permission($allow)
                })
            }
        }
    };
}
