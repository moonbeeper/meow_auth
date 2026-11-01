<script lang="ts" module>
    import { z } from "zod";
    export const schema = z.object({
        scopes: z.record(z.string(), z.boolean()).default({})
    });
</script>

<script lang="ts">
    import { invalidate } from "$app/navigation";
    import { IconDeviceFloppyFilled } from "@tabler/icons-svelte-runes";
    import { Control, Field } from "formsnap";
    import { untrack } from "svelte";
    import { slide } from "svelte/transition";
    import { superForm, defaults, setError } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    import { isOk } from "#lib/api/ignoreThisPlease.js";
    import {
        createAdminEditUserApplication,
        createAvailableScopesOauthApplication
    } from "#lib/api/index.js";
    import type { OauthApplication } from "#lib/api/model/oauthApplication.js";
    import Button from "#lib/components/button.svelte";
    import CheckboxBox from "#lib/components/form/checkbox_box.svelte";
    import LoadingBox from "#lib/components/loadingBox.svelte";
    import SettingsPanel from "#lib/components/settingsPanel.svelte";

    type Props = {
        item: OauthApplication;
        userId: string;
    };

    let { item, userId }: Props = $props();

    const updateApp = createAdminEditUserApplication();

    let savedScopes = $state(untrack(() => item.scopes));
    const availableScopesQuery = createAvailableScopesOauthApplication(() => ({
        id: item.id
    }));
    let availableScopes = $derived.by(() => {
        const res = availableScopesQuery.data;
        if (!res || res.status !== 200) return [];
        return res.data;
    });
    let scopesLoading = $derived(availableScopesQuery.isPending);
    let scopesReady = $state(false);

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

            const res = await updateApp.mutateAsync({
                id: userId,
                cid: item.id,
                data: {
                    name: item.name,
                    redirect_uri: item.redirect_uri,
                    public: item.public,
                    scopes: scopes
                }
            });

            if (!isOk(res)) {
                console.warn("failed updated scopes");
                setError(form, "", res.data.message ?? "Failed to update scopes :(");
                return;
            }

            savedScopes = scopes;

            if (selected_scopes.length == 0) {
                form.data.scopes["profile"] = true;
            }

            rawForm.reset({
                data: {
                    scopes: form.data.scopes
                },
                newState: form.data.scopes
            });
            invalidate((url) => url.pathname === "/me/my-apps");
        }
    });

    const { form, enhance, delayed } = rawForm;

    let scopeDirty = $derived.by(() => {
        if (!scopesReady) return false;

        const saved = new Set(savedScopes.split(" ").filter(Boolean));
        return availableScopes.some((scope) => {
            const checked = $form.scopes[scope.identifier] ?? false;
            return checked != saved.has(scope.identifier);
        });
    });

    let titleScopes = $derived.by(() => {
        return scopeDirty ? "Scopes" + "*" : "Scopes";
    });

    $effect(() => {
        if (scopesLoading || scopesReady) return;
        if (availableScopesQuery.isError) return;
        if (!availableScopesQuery.data) return;

        const current = new Set(savedScopes.split(" ").filter(Boolean));
        $form.scopes = Object.fromEntries(
            availableScopes.map((scope) => [scope.identifier, current.has(scope.identifier)])
        );
        scopesReady = true;
    });
</script>

<SettingsPanel
    title={titleScopes}
    description="These are the permissions this application can request from users"
    contentSpacing={2}
>
    <form class="form" use:enhance>
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
                    </Control>
                </Field>
            {/each}
        {/if}

        {#if scopeDirty}
            <div transition:slide>
                <Button
                    primary
                    disabled={$delayed}
                    loading={$delayed}
                    shouldFill
                    symbol_prefix={{ symbol: IconDeviceFloppyFilled }}
                >
                    Save scopes
                </Button>
            </div>
        {/if}
    </form>
</SettingsPanel>

<style lang="scss">
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
