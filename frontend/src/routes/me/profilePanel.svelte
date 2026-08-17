<script lang="ts" module>
    import { z } from "zod";

    const schema = z.object({
        display_name: z.string().min(3).max(50)
    });
</script>

<script lang="ts">
    import Avatar from "$comps/avatar.svelte";
    import Button from "$comps/button.svelte";
    import Input from "$comps/form/input.svelte";
    import InputError from "$comps/form/inputError.svelte";
    import InputProxy from "$comps/form/inputProxy.svelte";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import UploadBox from "$comps/uploadBox.svelte";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import queryClient from "$lib/api/tanstackClient";
    import { createChangeUserName, getCurrentUserInfoQueryKey } from "$lib/api/user/user";
    import { auth } from "$lib/auth/auth.svelte";
    import { Control, Field } from "formsnap";
    import { slide } from "svelte/transition";
    import { defaults, setError, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    const changeUserName = createChangeUserName(() => ({
        mutation: {
            onSuccess: () => {
                console.log("successfully updated user name");
                queryClient.invalidateQueries({ queryKey: getCurrentUserInfoQueryKey() });
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

            const res = await changeUserName.mutateAsync({
                data: { name: form.data.display_name }
            });

            if (!isOk(res)) {
                console.warn("failed updated user name");
                setError(form, "display_name", res.data.message ?? "Failed to update name :(");
            }

            rawForm.reset({
                data: { display_name: auth.user?.login },
                newState: {
                    display_name: auth.user?.login
                }
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

    let taintedName = $derived.by(() => {
        return isTainted($tainted?.display_name);
    });
</script>

<SettingsPanel title="Profile" description="Its your identity when you sign in to other apps!">
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
                            placeholder="Name"
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
            <label>Email address</label>
            <InputProxy>
                <Input value="meow@meow.com" placeholder="Email address" readonly type="email" />
                <a href="#main" class="email-button">Change Email</a>
            </InputProxy>
            <!-- <div class="flex input input--proxy">
                <input
                    class="input"
                    value="meow@meow.com"
                    placeholder="Email address"
                    readonly
                    type="email"
                />
                <a href="#main" class="untinify">Change Email</a>
            </div> -->
        </div>
        {#if taintedName}
            <div transition:slide>
                <Button primary disabled={$delayed} loading={$delayed} shouldFill
                    >Save changes</Button
                >
            </div>
        {/if}
    </form>
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
