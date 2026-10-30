-- Add up migration script here
-- I HATE me automagically putting a "," in the end of something like this because of beautiful copy and pasting
-- the line, expecting the thing to remove it. i do not have a lsp or formatter OR anything to tell me
-- "oh a comma, that's not supposed to be there. remove it buh". i have to figure it out by myself saying
-- "oh god WHAT DID I DO WRONG". i hate my stupid feathery thing furr thingy thing space birb head

create type oauth_applications_client_type as enum (
    'first_party',
    'third_party'
);
alter table oauth_applications add column client_type oauth_applications_client_type not null default 'third_party'::oauth_applications_client_type;

alter table oauth_applications alter column scopes type text using (
    concat_ws(' ',
        case when (scopes & 1) != 0 then 'openid' end,
        case when (scopes & 2) != 0 then 'profile' end,
        case when (scopes & 4) != 0 then 'email' end
    )
);

alter table oauth_authorizations alter column scopes type text using (
    concat_ws(' ',
        case when (scopes & 1) != 0 then 'openid' end,
        case when (scopes & 2) != 0 then 'profile' end,
        case when (scopes & 4) != 0 then 'email' end
    )
);

alter table oauth_pending_authorizations alter column old_scopes type text using (
    concat_ws(' ',
        case when (old_scopes & 1) != 0 then 'openid' end,
        case when (old_scopes & 2) != 0 then 'profile' end,
        case when (old_scopes & 4) != 0 then 'email' end
    )
);

alter table oauth_pending_authorizations alter column requested_scopes type text using (
    concat_ws(' ',
        case when (requested_scopes & 1) != 0 then 'openid' end,
        case when (requested_scopes & 2) != 0 then 'profile' end,
        case when (requested_scopes & 4) != 0 then 'email' end
    )
);

alter table oauth_pending_tokens alter column scopes type text using (
    concat_ws(' ',
        case when (scopes & 1) != 0 then 'openid' end,
        case when (scopes & 2) != 0 then 'profile' end,
        case when (scopes & 4) != 0 then 'email' end
    )
);

alter table oauth_tokens alter column scopes type text using (
    concat_ws(' ',
        case when (scopes & 1) != 0 then 'openid' end,
        case when (scopes & 2) != 0 then 'profile' end,
        case when (scopes & 4) != 0 then 'email' end
    )
);
