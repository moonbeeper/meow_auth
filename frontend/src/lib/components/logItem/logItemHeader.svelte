<script lang="ts">
    // "Complex binding patterns require an initialization value" when lang="ts" is not set... OH CMON!
    import Tag from "$comps/tag.svelte";
    import type { Snippet } from "svelte";

    let {
        title = "space birb",
        tag,
        who,
        when,
        children
    }: { title: string; tag?: string; who?: string; when?: string; children?: Snippet } = $props();
</script>

<div class="header">
    <div class="title">
        <span>{title}</span>
        {#if tag}<Tag>{tag}</Tag>{/if}
    </div>

    {#if children}
        {@render children()}
    {:else if who || when}
        <div class="who-when">
            {#if who}<span>{who}</span>{/if}
            {#if who && when}<span>·</span>{/if}
            {#if when}
                <span>{when}</span>
            {/if}
        </div>
    {/if}
</div>

<style lang="scss">
    .header {
        display: flex;
        flex-direction: column;
        min-inline-size: 0;
        flex: 1;
        font-size: var(--text-small);
    }

    .title {
        display: flex;
        gap: calc(var(--spacing) * 2);
        align-items: center;
        font-weight: 500;
    }

    .who-when {
        display: flex;
        gap: calc(var(--spacing) * 1);
        font-size: var(--text-smaller);
    }
</style>
