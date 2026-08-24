<script lang="ts">
    import Button from "$comps/button.svelte";
    import { Dialog } from "bits-ui";

    type Props = {
        title?: string;
    };

    let { title = "wawa prompt" }: Props = $props();
</script>

<div class="dialog--header">
    <Dialog.Title>{title}</Dialog.Title>
    <Dialog.Close>
        {#snippet child({ props })}
            <Button className="dialog--close-icon" {...props}>X</Button>
        {/snippet}
    </Dialog.Close>
</div>

<style lang="scss">
    .dialog--header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding-inline: calc(var(--spacing) * 2);
        padding-inline-start: calc(var(--spacing) * 4);
        padding-block: calc(var(--spacing) * 2);
        // padding: calc(var(--spacing) * 2);
        border-block-end: 1px solid var(--color-iron-medium);

        font-size: var(--text-normal);
        font-weight: 600;
    }

    :global(.dialog--close-icon) {
        --button-padding: calc(var(--spacing) * 2) calc(var(--spacing) * 3);
    }
</style>
