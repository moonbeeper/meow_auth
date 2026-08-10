<script lang="ts">
    import { DropdownMenu, type WithoutChildren } from "bits-ui";
    import type { Snippet } from "svelte";

    type Props = {
        children: Snippet;
    };
    type DropdownItemProps = WithoutChildren<DropdownMenu.ItemProps> & Props;

    let { children, ...rest }: DropdownItemProps = $props();
</script>

<DropdownMenu.Item class="dropdown--item" {...rest}>
    {@render children()}
</DropdownMenu.Item>

<style lang="scss">
    :global(.dropdown--item) {
        --dropitem-hover-brightness: 0.92;
        --dropitem-font-size: var(--text-normal);
        border-radius: calc(var(--typical-radius) / 2);
        padding: calc(var(--spacing) * 2) 1rem;
        /* accent-color: var(--color-accent-medium); */
        color: var(--dropitem-color, inherit);
        background: var(--dropitem-background, var(--color-body));
        cursor: pointer;
        font-size: var(--dropitem-font-size);
        font-weight: 500;
        display: inline-flex;
        transition-property: background-color, outline, border, opacity, color;
        transition-duration: 0.1s;
        transition-timing-function: ease-out;
        align-items: center;
        // makes so the line height does not change when the button is a link
        line-height: 1.1;
        gap: calc(var(--spacing) * 2);
        outline: 0;

        @media (any-hover: hover) {
            &:hover {
                filter: brightness(var(--dropitem-hover-brightness));
            }
        }

        @media (prefers-color-scheme: dark) {
            --dropitem-hover-brightness: 1.3;
        }
    }
</style>
