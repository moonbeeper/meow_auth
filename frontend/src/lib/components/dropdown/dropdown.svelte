<script lang="ts">
    import { DropdownMenu, type WithoutChild } from "bits-ui";
    import type { Snippet } from "svelte";
    import { fly, scale } from "svelte/transition";

    type Props = {
        contentProps?: WithoutChild<DropdownMenu.ContentProps>;
        triggerProps?: WithoutChild<DropdownMenu.TriggerProps>;
        trigger: Snippet<[{ props: Record<string, unknown> }]>;
        children: Snippet;
    };

    type DropdownProps = WithoutChild<DropdownMenu.RootProps> & Props;

    let {
        open = $bindable(false),
        children,
        trigger,
        contentProps,
        triggerProps,
        ...restProps
    }: DropdownProps = $props();
</script>

<DropdownMenu.Root bind:open {...restProps}>
    <DropdownMenu.Trigger {...triggerProps}>
        {#snippet child({ props })}
            {@render trigger({ props })}
        {/snippet}
    </DropdownMenu.Trigger>
    <DropdownMenu.Portal>
        <DropdownMenu.Content
            forceMount
            sideOffset={8}
            collisionPadding={12}
            class="dropdown--content"
            {...contentProps}
        >
            {#snippet child({ props, wrapperProps, open })}
                {#if open}
                    <div {...wrapperProps}>
                        <div transition:scale={{ duration: 100, start: 0.8 }} {...props}>
                            {@render children()}
                        </div>
                    </div>
                {/if}
            {/snippet}
        </DropdownMenu.Content>
    </DropdownMenu.Portal>
</DropdownMenu.Root>

<style lang="scss">
    :global(.dropdown--content) {
        background: var(--color-body);
        box-shadow: rgba(0, 0, 0, 0.16) 0px 1px 4px;
        border: 1px solid var(--color-iron-medium);
        min-inline-size: 18ch;
        // fits inside the MOBILE s from the device screen thingy inspector magical uhh device screens
        max-inline-size: 30ch;
        padding: calc(var(--spacing) * 1.2) calc(var(--spacing) * 1.3);
        display: flex;
        flex-direction: column;
        gap: calc(var(--spacing) * 1.2);
        outline: none;
        border-radius: var(--typical-radius);
        overflow: hidden;
        z-index: 20;
    }
</style>
