-- Add up migration script here
alter type webauthn_challenge_kind add value if not exists 'reauthenticate';
