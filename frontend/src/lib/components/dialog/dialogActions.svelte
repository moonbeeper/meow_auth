<script lang="ts">
    import type { Snippet } from "svelte";

    type Props = {
        children: Snippet;
    };

    let { children }: Props = $props();
</script>

<div class="actions">
    {@render children()}
</div>

<style lang="scss">
    .actions {
        display: flex;
        gap: calc(var(--spacing) * 2);
        padding: var(--dialog-content-padding);
        padding-block: calc(var(--spacing) * 2);
        justify-content: flex-end;
        flex-shrink: 0;
        flex-wrap: wrap;
        border-block-start: 1px solid var(--color-iron-medium);
    }
</style>
