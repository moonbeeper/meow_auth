<script lang="ts">
    import type { Snippet } from "svelte";

    import Checkbox, { type FormProps } from "./checkbox.svelte";
    import { getId } from "./idShenanigans";

    type Props = { id?: string; size?: FormProps["size"]; children: Snippet; checked?: boolean };

    let {
        id = getId(),
        size = "medium",
        children,
        checked = $bindable(false),
        ...rest
    }: Props = $props();
</script>

<label for={id} class="container" {...rest}>
    <Checkbox {id} {size} bind:checked />

    <div class="content">
        {@render children()}
    </div>
</label>

<style lang="scss">
    .container {
        --checkbox-box-hover-color: var(--color-accent-medium);
        display: inline-flex;
        align-items: center;
        cursor: pointer;
        gap: calc(var(--spacing) * 3);
        border-radius: calc(var(--spacing) * 2);
        padding-inline: calc(var(--spacing) * 3);
        padding-block: calc(var(--spacing) * 2);
        border: var(--checkbox-box-border-width, 0) solid
            var(--checkbox-box-border-color, transparent);

        transition-property: background-color, background;
        transition-duration: 0.1s;
        transition-timing-function: ease-out;

        @media (any-hover: hover) {
            &:hover {
                background: color-mix(in oklab, var(--checkbox-box-hover-color), transparent 60%);
            }
        }

        @media (prefers-color-scheme: dark) {
            --checkbox-box-hover-color: var(--color-accent-dark);
        }

        &[data-fs-invalid],
        &[aria-invalid="true"] {
            --checkbox-box-border-width: 1px;
            --checkbox-box-border-color: var(--color-coral-medium);

            @media (prefers-color-scheme: dark) {
                --checkbox-box-border-color: var(--color-coral-light);
            }
        }
    }

    .content {
        text-align: start;
        flex: 1;
        min-inline-size: 0;
    }
</style>
