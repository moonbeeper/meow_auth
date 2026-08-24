<script lang="ts">
    import type { Snippet } from "svelte";

    type Props = {
        children: Snippet;
    };
    let { children }: Props = $props();
</script>

<div class="content">
    {@render children()}
</div>

<style lang="scss">
    .content {
        display: grid;
        // grid-template-rows: minmax(0, 1fr);
        gap: calc(var(--spacing) * 2);
        padding: var(--dialog-content-padding);
        font-size: var(--text-small);
        overflow-y: auto;
        min-inline-size: 0;
        align-content: start;
    }
</style>
