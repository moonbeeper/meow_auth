<script lang="ts">
    import { Accordion } from "bits-ui";
    import type { Snippet } from "svelte";

    type Props = {
        children: Snippet;
        collapsible?: boolean;
    };

    let { children, collapsible }: Props = $props();
</script>

{#if collapsible}
    <div class="items">
        <Accordion.Root type="single">
            {@render children()}
        </Accordion.Root>
    </div>
{:else}
    <div class="items">
        {@render children()}
    </div>
{/if}

<style lang="scss">
    .items {
        border-radius: var(--typical-radius);
        border: 1px solid var(--color-iron-dark);
        overflow: hidden;
    }
</style>
