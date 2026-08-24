<script lang="ts">
    import Button, { type ButtonOrLinkProps } from "$comps/button.svelte";
    import { Dialog } from "bits-ui";
    import type { Snippet } from "svelte";

    type Props = {
        children: Snippet;
    };

    type DialogCloseProps = ButtonOrLinkProps & Props;
    let { children, loading, ...rest }: DialogCloseProps = $props();
</script>

<Dialog.Close>
    {#snippet child({ props })}
        <Button {...props} {...rest}>
            {@render children()}
        </Button>
    {/snippet}
</Dialog.Close>
