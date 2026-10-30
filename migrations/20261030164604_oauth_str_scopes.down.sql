-- Add down migration script here
alter table oauth_applications drop column client_type;
drop type if exists oauth_applications_client_type;

-- alter table oauth_applications add column scopes bigint not null default '';
alter table oauth_applications alter column scopes type bigint using (
    (case when 'openid' = any(string_to_array(scopes, ' ')) then 1 else 0 end) |
    (case when 'profile' = any(string_to_array(scopes, ' ')) then 2 else 0 end) |
    (case when 'email' = any(string_to_array(scopes, ' ')) then 4 else 0 end)
);

alter table oauth_authorizations alter column scopes type bigint using (
    (case when 'openid' = any(string_to_array(scopes, ' ')) then 1 else 0 end) |
    (case when 'profile' = any(string_to_array(scopes, ' ')) then 2 else 0 end) |
    (case when 'email' = any(string_to_array(scopes, ' ')) then 4 else 0 end)
);

alter table oauth_pending_authorizations alter column old_scopes type bigint using (
    (case when 'openid' = any(string_to_array(old_scopes, ' ')) then 1 else 0 end) |
    (case when 'profile' = any(string_to_array(old_scopes, ' ')) then 2 else 0 end) |
    (case when 'email' = any(string_to_array(old_scopes, ' ')) then 4 else 0 end)
);

alter table oauth_pending_authorizations alter column requested_scopes type bigint using (
    (case when 'openid' = any(string_to_array(requested_scopes, ' ')) then 1 else 0 end) |
    (case when 'profile' = any(string_to_array(requested_scopes, ' ')) then 2 else 0 end) |
    (case when 'email' = any(string_to_array(requested_scopes, ' ')) then 4 else 0 end)
);

alter table oauth_pending_tokens alter column scopes type bigint using (
    (case when 'openid' = any(string_to_array(scopes, ' ')) then 1 else 0 end) |
    (case when 'profile' = any(string_to_array(scopes, ' ')) then 2 else 0 end) |
    (case when 'email' = any(string_to_array(scopes, ' ')) then 4 else 0 end)
);

alter table oauth_tokens alter column scopes type bigint using (
    (case when 'openid' = any(string_to_array(scopes, ' ')) then 1 else 0 end) |
    (case when 'profile' = any(string_to_array(scopes, ' ')) then 2 else 0 end) |
    (case when 'email' = any(string_to_array(scopes, ' ')) then 4 else 0 end)
);
