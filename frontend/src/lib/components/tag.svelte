<script lang="ts">
    import type { Snippet } from "svelte";

    let { children }: { children: Snippet } = $props();
</script>

<span class="tag">
    {@render children()}
</span>

<style lang="scss">
    .tag {
        display: inline-block;
        padding-inline: calc(var(--spacing) * 1.5);
        padding-block: calc(var(--spacing) * 0.5);
        font-size: var(--text-smallest);
        background: var(--color-accent-lighter);
        border-radius: var(--typical-radius);
        color: var(--color-accent-dark);
        white-space: nowrap;
        // min-inline-size: 0.75rem;
        text-align: center;
        font-weight: 500;
    }
</style>
