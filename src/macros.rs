#[macro_export]
macro_rules! slash_command_permissions {
    ($vis:vis $name:ident: $allow:literal for User($id:literal)) => {
        slash_command_permissions!(@INNER $vis $name: User ($id): $allow);
    };
    ($vis:vis $name:ident: $allow:literal for Role($id:literal)) => {
        slash_command_permissions!(@INNER $vis $name: Role ($id): $allow);
    };
    (@INNER $vis:vis $name:ident: $variant:ident ($id:literal): $allow:literal) => {
        #[allow(non_camel_case_types)]
        $vis struct $name;

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

#[macro_export]
macro_rules! slash_command_options {
    ($vis:vis $name:ident: |$v:ident| $body:block) => {
        #[allow(non_camel_case_types)]
        $vis struct $name;

        impl $name {
            pub fn apply($v: &mut ::serenity::builder::CreateApplicationCommandOption) -> &mut ::serenity::builder::CreateApplicationCommandOption $body
        }
    };
}
