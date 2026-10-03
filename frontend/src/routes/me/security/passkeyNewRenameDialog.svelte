<script lang="ts" module>
    import { z } from "zod";
    const schema = z.object({
        name: z
            .string()
            .min(3, "Needs to be least 3 characters long")
            .max(50, "Needs to be at most 50 characters long")
    });
</script>

<script lang="ts">
    import Button from "$comps/button.svelte";
    import * as Dialog from "$comps/dialog";
    import Input from "$comps/form/input.svelte";
    import InputError from "$comps/form/inputError.svelte";
    import { createRenamePasskey, getListPasskeysQueryKey } from "$lib/api";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import queryClient from "$lib/api/tanstackClient";
    import { Control, Field } from "formsnap";
    import { defaults, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    export type Props = {
        id: string;
        actionBool: boolean;
        open: boolean;
        isReady: boolean;
        currentName?: string;
        actionMode: ActionMode;
    };
    export type ActionMode = "add" | "update";

    let {
        id,
        open = $bindable(false),
        actionBool = $bindable(),
        isReady,
        currentName,
        actionMode
    }: Props = $props();

    const requestRenamePasskey = createRenamePasskey(() => ({
        mutation: {
            onSuccess: () => {
                queryClient.invalidateQueries({ queryKey: getListPasskeysQueryKey() });
            }
        }
    }));

    const rawForm = superForm(defaults(zod4(schema)), {
        SPA: true,
        resetForm: true,
        validationMethod: "onsubmit", // makes so the error (data-fs-error) doesnt dissapear after blur
        validators: zod4(schema),
        onUpdate: async ({ form }) => {
            if (!form.valid) {
                console.warn("somehow the form was submitted without being valid");
                return;
            }

            const res = await requestRenamePasskey.mutateAsync({
                id: id,
                data: {
                    name: form.data.name
                }
            });

            if (!isOk(res)) {
                console.error("failed updating passkey name");
                open = false;

                return;
            }

            console.log("updated passkey name successfully");
            open = false;
        }
    });

    const { form, enhance, delayed } = rawForm;

    let message = $derived.by(() => {
        if (actionMode == "update") {
            return "Ah yes, renaming your passkey so it becomes more rememberable. You can always change it again later.";
        }
        return "You can now rename it to something more rememberable or just leave it as is, and you can always change it later.";
    });

    let title = $derived.by(() => {
        if (actionMode == "update") {
            return "Rename your Passkey";
        }
        return "Name your new Passkey";
    });

    let inputPlaceholder = $derived.by(() => {
        if (actionMode == "update") {
            return currentName;
        }
        return "My beautiful passkey 1";
    });

    let cancelButtonText = $derived.by(() => {
        if (actionMode == "update") {
            return "Cancel";
        }
        return "No thanks";
    });

    $effect(() => {
        if (!open && isReady) {
            rawForm.reset();
            actionBool = false;
        }
    });
</script>

<Dialog.Root {title} bind:open>
    {message}

    <form class="form" method="post" use:enhance>
        <Field form={rawForm} name="name">
            <Control>
                {#snippet children({ props })}
                    <Input
                        {...props}
                        type="text"
                        disabled={$delayed}
                        placeholder={inputPlaceholder}
                        bind:value={$form.name}
                    />
                {/snippet}
            </Control>
            <InputError />
        </Field>
    </form>

    {#snippet actions()}
        <Dialog.Close>{cancelButtonText}</Dialog.Close>
        <Button primary onclick={() => rawForm.submit()} loading={$delayed} disabled={$delayed}
            >Rename</Button
        >
    {/snippet}
</Dialog.Root>

<style lang="scss">
    .form {
        display: flex;
        flex-direction: column;
        gap: var(--spacing);
        padding-inline: calc(var(--spacing) * 2);
    }
</style>
