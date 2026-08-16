<script lang="ts" module>
    import { z } from "zod";

    const schema = z.object({
        display_name: z.string().min(3).max(50)
    });
</script>

<script lang="ts">
    import { goto } from "$app/navigation";
    import * as AuthBox from "$comps/authBox";
    import Button from "$comps/button.svelte";
    import Input from "$comps/form/input.svelte";
    import InputError from "$comps/form/inputError.svelte";
    import { isOk } from "$lib/api/ignoreThisPlease";
    import queryClient from "$lib/api/tanstackClient";
    import { createChangeUserName, getCurrentUserInfoQueryKey } from "$lib/api/user/user";
    import { Control, Field } from "formsnap";
    import { defaults, setError, superForm } from "sveltekit-superforms";
    import { zod4 } from "sveltekit-superforms/adapters";

    import type { PageProps } from "./$types";

    let { data }: PageProps = $props();

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
                return;
            }

            await goto(data.redirect ?? "/me");
        }
    });

    const { form, enhance, delayed } = rawForm;
</script>

<AuthBox.Root>
    <AuthBox.Header title="Complete your sign up" />
    <form class="form" use:enhance method="post">
        <Field form={rawForm} name="display_name">
            <Control>
                {#snippet children({ props })}
                    <Input
                        type="text"
                        fontSize="large"
                        placeholder="Enter your display name"
                        autocomplete="name"
                        disabled={$delayed}
                        required
                        {...props}
                        bind:value={$form.display_name}
                    />
                {/snippet}
            </Control>
            <InputError />
        </Field>
        <p class="font-medium">It's time to complete your sign up!</p>
        <Button primary fontSize="medium" type="submit" disabled={$delayed} loading={$delayed}>
            Continue
        </Button>
    </form>
</AuthBox.Root>
