<script lang="ts">
    import type { Snippet } from "svelte";

    let { children }: { children: Snippet } = $props();
</script>

<div class="actions">
    {@render children()}
</div>

<style lang="scss">
    .actions {
        display: flex;
        gap: calc(var(--spacing) * 2);
        align-items: center;
        flex-shrink: 0;
    }
</style>
