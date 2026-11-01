<script lang="ts" module>
    import { z } from "zod";

    const schema = z.object({
        display_name: z.string().min(3).max(50),
        email: z.email()
    });
</script>

<script lang="ts">
    import Avatar from "$comps/avatar.svelte";
    import Button from "$comps/button.svelte";
    import Input from "$comps/form/input.svelte";
    import InputError from "$comps/form/inputError.svelte";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import UploadBox from "$comps/uploadBox.svelte";
    import { IconDeviceFloppyFilled, IconTrashXFilled } from "@tabler/icons-svelte-runes";
    import { Control, Field } from "formsnap";
    import { slide } from "svelte/transition";
    import { defaults, setError, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    import { isOk } from "#lib/api/ignoreThisPlease.js";
    import { createAdminEditUser } from "#lib/api/index.js";
    import { auth } from "#lib/auth/auth.svelte.js";

    import type { LayoutProps } from "./$types";
    import FlagsPanel from "./flagsPanel.svelte";

    let { data }: LayoutProps = $props();
    let userInfo = $derived(data.user);

    const updateProfile = createAdminEditUser();
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

            const res = await updateProfile.mutateAsync({
                id: userInfo.id,
                data: { name: form.data.display_name, email: form.data.email }
            });

            if (!isOk(res)) {
                console.warn("failed to update user");
                setError(form, "display_name", res.data.message ?? "Failed to update user :(");
            }

            rawForm.reset({
                data: form.data,
                newState: form.data
            });
        }
    });

    const { form, enhance, delayed, isTainted, tainted } = rawForm;

    let formFilled = $state(false);

    $effect(() => {
        const user = auth.user;
        if (!formFilled && user) {
            formFilled = true;
            rawForm.reset({
                data: { display_name: user.login },
                newState: {
                    display_name: user.login
                }
            });
        }
    });

    let taintedInfo = $derived.by(() => {
        return isTainted($tainted?.display_name || $tainted?.email);
    });

    $effect(() => {
        rawForm.reset({
            data: { display_name: userInfo.login, email: userInfo.email },
            newState: { display_name: userInfo.login, email: userInfo.email }
        });
    });
</script>

<SettingsPanel
    title="Profile"
    description="The identity an user sees when they sign in to other apps"
>
    <form class="form" use:enhance>
        <div class="profile-form">
            <!-- <div class="avatar2 aa">?</div> -->
            <Avatar />
            <UploadBox />
        </div>
        <div class="flex flex-col gap-2 text-start">
            <Field form={rawForm} name="display_name">
                <Control>
                    {#snippet children({ props })}
                        <label>Name</label>
                        <Input
                            bind:value={$form.display_name}
                            placeholder={$form.display_name}
                            type="text"
                            disabled={$delayed}
                            {...props}
                        />
                    {/snippet}

                    <!-- <input class="input" value="Cow69" placeholder="Name" type="text" /> -->
                </Control>
                <InputError />
            </Field>
        </div>
        <div class="flex flex-col gap-2 text-start">
            <Field form={rawForm} name="email">
                <Control>
                    {#snippet children({ props })}
                        <label>Email</label>
                        <Input
                            bind:value={$form.email}
                            placeholder={$form.email}
                            type="email"
                            disabled={$delayed}
                            {...props}
                        />
                    {/snippet}

                    <!-- <input class="input" value="Cow69" placeholder="Name" type="text" /> -->
                </Control>
                <InputError />
            </Field>
        </div>
        {#if taintedInfo}
            <div transition:slide>
                <Button
                    primary
                    disabled={$delayed}
                    loading={$delayed}
                    shouldFill
                    symbol_prefix={{ symbol: IconDeviceFloppyFilled }}
                >
                    Save changes
                </Button>
            </div>
        {/if}
    </form>
</SettingsPanel>
<FlagsPanel user={userInfo} onSaved={() => {}} />

<SettingsPanel
    title="Delete account"
    description="Delete your account and everything associated with it, including authorized apps and created apps."
    negative
>
    <Button negative symbol_prefix={{ symbol: IconTrashXFilled }}>Delete account</Button>
</SettingsPanel>

<style lang="scss">
    .email-button {
        font-size: var(--text-small);
        white-space: nowrap;
        padding-inline-start: calc(var(--spacing) * 3);
    }

    .form {
        display: contents;
    }

    .profile-form {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: calc(var(--spacing) * 4);
        @media (max-width: 768px) {
            flex-direction: column;
        }
    }
</style>
