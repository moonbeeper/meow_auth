<script lang="ts">
    import { Checkbox, type WithoutChild } from "bits-ui";
    import type { HTMLInputAttributes } from "svelte/elements";

    import { getId } from "./idShenanigans";

    type Props = {
        size?: "small" | "medium" | "large";
    };

    export type FormProps = WithoutChild<Checkbox.RootProps> & Props;

    let {
        id = getId(),
        checked = $bindable(false),
        ref = $bindable(null),
        size = "medium",
        ...rest
    }: FormProps = $props();

    let actual_size = $derived.by(() => {
        return "checkbox--" + size;
    });
</script>

<Checkbox.Root class={["checkbox", actual_size]} bind:checked {id} bind:ref {...rest}>
    {#snippet children({ checked, indeterminate })}
        <span class="content">
            {#if indeterminate}
                -
            {:else if checked}
                V
            {/if}
        </span>
    {/snippet}
</Checkbox.Root>

<style lang="scss">
    :global(.checkbox) {
        --checkbox-size-multiplier: 1;
        --checkbox-hover-brightness: 0.9;
        all: unset;
        position: relative;
        display: inline-flex;
        justify-content: center;
        align-items: center;
        flex-shrink: 0;
        color: var(--color-iron-lightest);
        height: calc(1rem * var(--checkbox-size-multiplier));
        width: calc(1rem * var(--checkbox-size-multiplier));
        border-radius: calc(var(--spacing) * 1.25 * var(--checkbox-size-multiplier));
        // transition-property: transform;
        // transition-duration: 0.1s;
        // transition-timing-function: ease-out;
        // transform: scale(1);

        @media (any-hover: hover) {
            &:hover {
                filter: brightness(var(--checkbox-hover-brightness));
            }
        }

        @media (prefers-color-scheme: dark) {
            --checkbox-hover-brightness: 1.15;
            color: var(--color-iron-darkest);
        }

        // &:active {
        //     transform: scale(0.99);
        // }
    }

    :global(.checkbox[data-state="unchecked"]) {
        box-shadow: inset 0 0 0 1px var(--color-iron-medium);
    }

    :global(.checkbox[data-state="checked"]) {
        background: var(--color-accent-medium);
    }

    :global(.checkbox--small) {
        --checkbox-size-multiplier: 0.75;
    }

    :global(.checkbox--large) {
        --checkbox-size-multiplier: 1.25;
    }

    .content {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        pointer-events: none;
        user-select: none;
    }
</style>
