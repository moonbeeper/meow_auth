<script lang="ts" module>
    import { z } from "zod";

    export const schema = z.object({
        display_name: z.string().min(3).max(50),
        redirect_uri: z.url(),
        public: z.boolean().default(true),
        openid: z.boolean().default(false),
        profile: z.boolean().default(false),
        email: z.boolean().default(false)
    });
</script>

<script lang="ts">
    import { invalidate } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import CheckboxBox from "$comps/form/checkbox_box.svelte";
    import Input from "$comps/form/input.svelte";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import UploadBox from "$comps/uploadBox.svelte";
    import { Control, Field } from "formsnap";
    import { defaults, setError, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    import { isOk } from "#lib/api/ignoreThisPlease.js";
    import {
        createCreateOauthApplication,
        getListOauthApplicationsQueryKey
    } from "#lib/api/index.js";
    import type { OauthApplicationDataResponse } from "#lib/api/model/index.js";
    import queryClient from "#lib/api/tanstackClient.js";
    import { OauthScope, OauthScopes } from "#lib/oauthScopes.js";

    import AfterCreationDialog from "./afterCreationDialog.svelte";

    let openDialog = $state(false);
    let createdApp = $state<OauthApplicationDataResponse | null>(null);

    const createApp = createCreateOauthApplication(() => ({
        mutation: {
            onSuccess: () => {
                console.log("successfully created app");
                queryClient.invalidateQueries({ queryKey: getListOauthApplicationsQueryKey() });
            }
        }
    }));

    const rawForm = superForm(defaults(zod4(schema)), {
        SPA: true,
        resetForm: false,
        validationMethod: "onsubmit", // makes so the error (data-fs-error) doesnt dissapear after blur
        validators: zod4(schema),
        onUpdate: async ({ form }) => {
            if (!form.valid) {
                console.warn("somehow the form was submitted without being valid");
                return;
            }

            let bits = new OauthScopes(0);
            if (form.data.openid) bits.add(OauthScope.OpenId);
            if (form.data.profile) bits.add(OauthScope.Profile);
            if (form.data.email) bits.add(OauthScope.Email);

            const res = await createApp.mutateAsync({
                data: {
                    name: form.data.display_name,
                    redirect_uri: form.data.redirect_uri,
                    public: form.data.public,
                    scopes: bits.bits
                }
            });

            if (!isOk(res)) {
                console.warn("failed updated user name");
                setError(form, "display_name", res.data.message ?? "Failed to update name :(");
                return;
            }

            createdApp = res.data;
            openDialog = true;

            invalidate((url) => url.pathname === "/me/my-apps");
        }
    });

    const { form, enhance, delayed } = rawForm;
</script>

<AfterCreationDialog bind:open={openDialog} item={createdApp} />
<form class="form" use:enhance>
    <SettingsPanel
        title="Application details"
        description="How will your application be identified to users? This is the name and logo that will be shown to end user when they are asked to authorize your application"
    >
        <div class="app--logo">
            <img
                class="app--logo-img"
                src="https://github.com/identicons/app/oauth_app/3137776"
                alt="app logo"
            />
            <UploadBox />
        </div>
        <div class="flex flex-col gap-2 text-start">
            <Field form={rawForm} name="display_name">
                <Control>
                    {#snippet children({ props })}
                        <label>Name</label>
                        <Input
                            type="text"
                            placeholder="Name"
                            disabled={$delayed}
                            bind:value={$form.display_name}
                            {...props}
                        />
                    {/snippet}
                </Control>
            </Field>
        </div>
        <div class="flex flex-col gap-2 text-start">
            <Field form={rawForm} name="redirect_uri">
                <Control>
                    {#snippet children({ props })}
                        <label>Redirect URL</label>
                        <Input
                            type="text"
                            placeholder="Redirect URL"
                            disabled={$delayed}
                            bind:value={$form.redirect_uri}
                            {...props}
                        />
                    {/snippet}
                </Control>
            </Field>
        </div>
    </SettingsPanel>

    <SettingsPanel
        title="Scopes"
        description="These are the permissions your application can request from users"
        contentSpacing={2}
    >
        <!-- <form class="form" use:enhanceScope> -->
        <CheckboxBox size="large" bind:checked={$form.openid}>
            <span class="checkbox-box--title">openid</span>
            <p class="checkbox-box--description">View meow</p>
        </CheckboxBox>
        <CheckboxBox size="large" bind:checked={$form.profile}>
            <span class="checkbox-box--title">profile</span>
            <p class="checkbox-box--description">View meow</p>
        </CheckboxBox>
        <CheckboxBox size="large" bind:checked={$form.email}>
            <span class="checkbox-box--title">email</span>
            <p class="checkbox-box--description">View meow</p>
        </CheckboxBox>
        <!-- {#if scopeDirty}
            <div transition:slide>
                <Button primary disabled={$delayedScope} loading={$delayedScope} shouldFill
                    >Save changes</Button
                >
            </div>
        {/if}
    </form> -->
    </SettingsPanel>
    <SettingsPanel
        title="Create application"
        description="Ah, a new digital child that will have access to user's accounts"
        contentSpacing={2}
    >
        <Button primary square type="submit" disabled={$delayed} loading={$delayed}
            >Create application</Button
        >
    </SettingsPanel>
</form>

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
