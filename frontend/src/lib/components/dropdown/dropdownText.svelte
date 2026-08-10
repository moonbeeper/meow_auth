<script lang="ts">
    import { DropdownMenu, type WithoutChildren } from "bits-ui";
    import type { Snippet } from "svelte";

    type Props = {
        children: Snippet;
    };
    type DropdownTextProps = Props;

    let { children }: DropdownTextProps = $props();
</script>

<div class="dropdown--text">
    {@render children()}
</div>

<style lang="scss">
    .dropdown--text {
        display: flex;
        flex-direction: column;
        text-align: start;
        padding: calc(var(--spacing) * 2) 1rem;
    }
</style>
