<script lang="ts" module>
    import { z } from "zod";

    export const schema = z.object({
        display_name: z.string().min(3).max(50),
        redirect_uri: z.url()
    });
    export const schemaScopes = z.object({
        openid: z.boolean(),
        profile: z.boolean(),
        email: z.boolean()
    });
</script>

<script lang="ts">
    import { invalidate } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import CheckboxBox from "$comps/form/checkbox_box.svelte";
    import Input from "$comps/form/input.svelte";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import UploadBox from "$comps/uploadBox.svelte";
    import { createUpdateOauthApplication, getListOauthApplicationsQueryKey } from "$lib/api";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import queryClient from "$lib/api/tanstackClient";
    import { actionDate } from "$lib/common";
    import { OauthScope, OauthScopes } from "$lib/oauthScopes";
    import { Control, Field } from "formsnap";
    import { slide } from "svelte/transition";
    import { defaults, setError, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    import type { PageProps } from "./$types";
    import DeleteAppDialog from "./deleteAppDialog.svelte";
    import RotateSecretDialog from "./rotateSecretDialog.svelte";

    let { data }: PageProps = $props();

    let description = $derived.by(() => {
        return "Created " + actionDate(data.app.created_at) + "\n\n" + data.app.id;
    });

    let openDeleteDialog = $state(false);
    let openRotateDialog = $state(false);
    const updateApp = createUpdateOauthApplication(() => ({
        mutation: {
            onSuccess: () => {
                console.log("successfully updated user name");
                queryClient.invalidateQueries({ queryKey: getListOauthApplicationsQueryKey() });
            }
        }
    }));

    const rawFormInfo = superForm(defaults(zod4(schema)), {
        SPA: true,
        resetForm: false,
        validationMethod: "onsubmit", // makes so the error (data-fs-error) doesnt dissapear after blur
        validators: zod4(schema),
        onUpdate: async ({ form }) => {
            if (!form.valid) {
                console.warn("somehow the form was submitted without being valid");
                return;
            }

            const res = await updateApp.mutateAsync({
                id: data.app.id,
                data: {
                    name: form.data.display_name,
                    redirect_uri: form.data.redirect_uri,
                    public: data.app.public,
                    scopes: data.app.scopes
                }
            });

            if (!isOk(res)) {
                console.warn("failed updated user name");
                setError(form, "display_name", res.data.message ?? "Failed to update name :(");
                return;
            }

            rawFormInfo.reset({
                data: {
                    display_name: form.data.display_name,
                    redirect_uri: form.data.redirect_uri
                },
                newState: {
                    display_name: form.data.display_name,
                    redirect_uri: form.data.redirect_uri
                }
            });

            invalidate((url) => url.pathname === "/me/my-apps");
        }
    });

    const { form, enhance, delayed, isTainted, tainted } = rawFormInfo;
    let infoDirty = $derived.by(() => {
        return isTainted($tainted?.display_name) || isTainted($tainted?.redirect_uri);
    });

    let title = $derived.by(() => {
        return infoDirty ? data.app.name + "*" : data.app.name;
    });

    const rawFormScopes = superForm(defaults(zod4(schemaScopes)), {
        SPA: true,
        resetForm: false,
        validationMethod: "onsubmit", // makes so the error (data-fs-error) doesnt dissapear after blur
        validators: zod4(schemaScopes),
        onUpdate: async ({ form }) => {
            if (!form.valid) {
                console.warn("somehow the form was submitted without being valid");
                return;
            }

            let bits = new OauthScopes(0);
            if (form.data.openid) bits.add(OauthScope.OpenId);
            if (form.data.profile) bits.add(OauthScope.Profile);
            if (form.data.email) bits.add(OauthScope.Email);

            const res = await updateApp.mutateAsync({
                id: data.app.id,
                data: {
                    name: data.app.name,
                    redirect_uri: data.app.redirect_uri,
                    public: data.app.public,
                    scopes: bits.bits
                }
            });

            if (!isOk(res)) {
                console.warn("failed updated user name");
                return;
                // setError(form, "display_name", res.data.message ?? "Failed to update name :(");
            }

            rawFormScopes.reset({
                data: {
                    email: form.data.email,
                    openid: form.data.openid,
                    profile: form.data.profile
                },
                newState: {
                    email: form.data.email,
                    openid: form.data.openid,
                    profile: form.data.profile
                }
            });
            invalidate((url) => url.pathname === "/me/my-apps");
        }
    });

    const { form: formScope, enhance: enhanceScope, delayed: delayedScope } = rawFormScopes;
    let scopeDirty = $derived.by(() => {
        let bits = new OauthScopes(0);
        if ($formScope.openid) bits.add(OauthScope.OpenId);
        if ($formScope.profile) bits.add(OauthScope.Profile);
        if ($formScope.email) bits.add(OauthScope.Email);

        return bits.bits != data.app.scopes;
    });

    let titleScopes = $derived.by(() => {
        return scopeDirty ? "Scopes" + "*" : "Scopes";
    });

    $effect(() => {
        rawFormInfo.reset({
            data: {
                display_name: data.app.name,
                redirect_uri: data.app.redirect_uri
            },
            newState: {
                display_name: data.app.name,
                redirect_uri: data.app.redirect_uri
            }
        });

        let bits = new OauthScopes(data.app.scopes);

        rawFormScopes.reset({
            data: {
                email: bits.has(OauthScope.Email),
                openid: bits.has(OauthScope.OpenId),
                profile: bits.has(OauthScope.Profile)
            },
            newState: {
                email: bits.has(OauthScope.Email),
                openid: bits.has(OauthScope.OpenId),
                profile: bits.has(OauthScope.Profile)
            }
        });
    });
</script>

<DeleteAppDialog item={data.app} bind:open={openDeleteDialog} />
<RotateSecretDialog item={data.app} bind:open={openRotateDialog} />
<SettingsPanel {title} {description}>
    <div class="app--logo">
        <img
            class="app--logo-img"
            src="https://github.com/identicons/app/oauth_app/3137776"
            alt="app logo"
        />
        <UploadBox />
    </div>
    <form class="form" use:enhance>
        <div class="flex flex-col gap-2 text-start">
            <Field form={rawFormInfo} name="display_name">
                <Control>
                    {#snippet children({ props })}
                        <label>Name</label>
                        <Input
                            placeholder={data.app.name}
                            type="text"
                            disabled={$delayed}
                            bind:value={$form.display_name}
                            {...props}
                        />
                    {/snippet}
                </Control>
            </Field>
        </div>
        <div class="flex flex-col gap-2 text-start">
            <Field form={rawFormInfo} name="redirect_uri">
                <Control>
                    {#snippet children({ props })}
                        <label>Redirect URL</label>
                        <Input
                            placeholder={data.app.redirect_uri}
                            type="text"
                            disabled={$delayed}
                            bind:value={$form.redirect_uri}
                            {...props}
                        />
                    {/snippet}
                </Control>
            </Field>
        </div>
        {#if infoDirty}
            <div transition:slide>
                <Button primary disabled={$delayed} loading={$delayed} shouldFill
                    >Save changes</Button
                >
            </div>
        {/if}
    </form>
</SettingsPanel>

<SettingsPanel
    title={titleScopes}
    description="These are the permissions this application can request from users"
    contentSpacing={2}
>
    <form class="form" use:enhanceScope>
        <CheckboxBox size="large" bind:checked={$formScope.openid}>
            <span class="checkbox-box--title">openid</span>
            <p class="checkbox-box--description">View meow</p>
        </CheckboxBox>
        <CheckboxBox size="large" bind:checked={$formScope.profile}>
            <span class="checkbox-box--title">profile</span>
            <p class="checkbox-box--description">View meow</p>
        </CheckboxBox>
        <CheckboxBox size="large" bind:checked={$formScope.email}>
            <span class="checkbox-box--title">email</span>
            <p class="checkbox-box--description">View meow</p>
        </CheckboxBox>
        {#if scopeDirty}
            <div transition:slide>
                <Button primary disabled={$delayedScope} loading={$delayedScope} shouldFill
                    >Save scopes</Button
                >
            </div>
        {/if}
    </form>
</SettingsPanel>
<SettingsPanel
    title="Danger zone"
    description="These actions are irreversible! Consequences must be acknowledged"
    negative
    contentSpacing={2}
>
    <!-- <Button>Revoke all authorizations</Button> -->
    <Button negative onclick={() => (openRotateDialog = true)}>Rotate secret</Button>
    <Button negative onclick={() => (openDeleteDialog = true)}>Delete this app</Button>
</SettingsPanel>

<style lang="scss">
    .app--logo-img {
        position: relative;
        // width: 96px;
        // height: 96px;
        // margin-inline: auto;
        margin-bottom: calc(var(--spacing) * 2);
        inline-size: 8rem;
        // block-size: 4rem;
        flex-shrink: 0;
        aspect-ratio: 1;
        border-radius: var(--img-radius, var(--typical-radius));
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 600;
        font-size: var(--text-small);
        // overflow: hidden;
        border: 1px solid var(--color-accent-light);

        // subtle as HECK but i like it
        box-shadow: rgba(0, 0, 0, 0.05) 0px 6px 24px 0px;
    }

    .app--logo {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: calc(var(--spacing) * 4);
        @media (max-width: 768px) {
            flex-direction: column;
        }
    }

    .checkbox-box--title {
        font-family: var(--font-mono);
        font-size: var(--text-small);
        font-weight: 500;
    }

    .checkbox-box--description {
        pointer-events: none;
        user-select: none;
        font-size: var(--text-small);
        color: var(--color-iron-darker);
    }
    .form {
        display: contents;
    }
</style>
