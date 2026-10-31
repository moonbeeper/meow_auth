<script lang="ts" module>
    import { z } from "zod";

    export const schema = z.object({
        display_name: z.string().min(3).max(50),
        redirect_uri: z.url(),
        public: z.boolean().default(true),
        scopes: z.record(z.string(), z.boolean()).default({})
    });
</script>

<script lang="ts">
    import { invalidate } from "$app/navigation";
    import Button from "$comps/button.svelte";
    import CheckboxBox from "$comps/form/checkbox_box.svelte";
    import Input from "$comps/form/input.svelte";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import UploadBox from "$comps/uploadBox.svelte";
    import { IconSquareRoundedPlusFilled } from "@tabler/icons-svelte-runes";
    import { Control, Field } from "formsnap";
    import { defaults, setError, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    import { isOk } from "#lib/api/ignoreThisPlease.js";
    import {
        createAvailableScopesOauthApplication,
        createCreateOauthApplication,
        getCurrentUserAuditLogQueryKey,
        getListOauthApplicationsQueryKey
    } from "#lib/api/index.js";
    import type { OauthApplicationDataResponse } from "#lib/api/model/index.js";
    import queryClient from "#lib/api/tanstackClient.js";
    import LoadingBox from "#lib/components/loadingBox.svelte";

    import AfterCreationDialog from "./afterCreationDialog.svelte";

    let openDialog = $state(false);
    let createdApp = $state<OauthApplicationDataResponse | null>(null);

    const availableScopesQuery = createAvailableScopesOauthApplication();

    let availableScopes = $derived.by(() => {
        const res = availableScopesQuery.data;
        if (!res || res.status !== 200) return [];
        return res.data;
    });
    let scopesLoading = $derived(availableScopesQuery.isPending);
    let scopesReady = $state(false);

    const createApp = createCreateOauthApplication(() => ({
        mutation: {
            onSuccess: () => {
                console.log("successfully created app");
                queryClient.invalidateQueries({ queryKey: getListOauthApplicationsQueryKey() });
                queryClient.invalidateQueries({ queryKey: getCurrentUserAuditLogQueryKey() });
            }
        }
    }));

    const rawForm = superForm(defaults(zod4(schema)), {
        SPA: true,
        resetForm: false,
        validationMethod: "onsubmit", // makes so the error (data-fs-error) doesnt dissapear after blur
        validators: zod4(schema),
        dataType: "json",
        onUpdate: async ({ form }) => {
            if (!form.valid) {
                console.warn("somehow the form was submitted without being valid");
                return;
            }

            const selected_scopes = Object.entries(form.data.scopes)
                .filter(([_, checked]) => checked)
                .map(([identifier, _]) => identifier)
                .join(" ");
            const scopes = selected_scopes.length == 0 ? "profile" : selected_scopes;

            const res = await createApp.mutateAsync({
                data: {
                    name: form.data.display_name,
                    redirect_uri: form.data.redirect_uri,
                    public: form.data.public,
                    scopes: scopes
                }
            });

            if (!isOk(res)) {
                console.warn("failed updated scopes");
                setError(form, "display_name", res.data.message ?? "Failed to create application");
                return;
            }

            createdApp = res.data;
            openDialog = true;

            invalidate((url) => url.pathname === "/me/my-apps");
        }
    });

    const { form, enhance, delayed } = rawForm;
    $effect(() => {
        if (scopesLoading || scopesReady) return;
        if (availableScopesQuery.isError) return;
        if (!availableScopesQuery.data) return;

        $form.scopes = Object.fromEntries(
            availableScopes.map((scope) => [scope.identifier, false])
        );
        scopesReady = true;
    });
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
        {#if scopesLoading || !scopesReady}
            <LoadingBox
                what="scopes"
                loaded={scopesLoading || !scopesReady}
                hasError={availableScopesQuery.isError}
            />
        {:else}
            {#each availableScopes as scope (scope.identifier)}
                <Field form={rawForm} name="scopes">
                    <Control>
                        {#snippet children({ props })}
                            <CheckboxBox
                                size="large"
                                bind:checked={$form.scopes[scope.identifier]}
                                {...props}
                            >
                                <span class="checkbox-box--title">{scope.identifier}</span>
                                {#if scope.description}
                                    <p class="checkbox-box--description">{scope.description}</p>
                                {/if}
                            </CheckboxBox>
                        {/snippet}
                    </Control></Field
                >
            {/each}
        {/if}
    </SettingsPanel>
    <SettingsPanel
        title="Create application"
        description="Ah, a new digital child that will have access to user's accounts"
        contentSpacing={2}
    >
        <Button
            primary
            square
            type="submit"
            disabled={$delayed || scopesLoading}
            loading={$delayed}
            symbol_prefix={{ symbol: IconSquareRoundedPlusFilled, size: "medium" }}
        >
            Create application
        </Button>
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
