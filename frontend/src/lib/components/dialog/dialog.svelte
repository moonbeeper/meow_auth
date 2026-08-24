<script lang="ts">
    import { Dialog, type WithoutChild } from "bits-ui";
    import type { Snippet } from "svelte";
    import { fade, scale } from "svelte/transition";

    import DialogActions from "./dialogActions.svelte";
    import DialogContent from "./dialogContent.svelte";
    import DialogHeader from "./dialogHeader.svelte";

    type Props = {
        trigger?: Snippet<[{ props: Record<string, unknown> }]>;
        children: Snippet;
        actions?: Snippet;
        triggerProps?: WithoutChild<Dialog.TriggerProps>;
        contentProps?: WithoutChild<Dialog.ContentProps>;
        title?: string;
        width?: "normal" | "large";
        fullScreenMobile?: boolean;
        role?: "dialog" | "alertdialog";
    };
    type DialogProps = WithoutChild<Dialog.RootProps> & Props;

    let {
        open = $bindable(false),
        trigger,
        children,
        actions,
        triggerProps,
        contentProps,
        title,
        width = "normal",
        role = "dialog",
        fullScreenMobile = false,
        ...rest
    }: DialogProps = $props();

    let widthClass = $derived.by(() => {
        return "dialog-" + width;
    });
</script>

<Dialog.Root bind:open {...rest}>
    {#if trigger}
        <Dialog.Trigger {...triggerProps}>
            {#snippet child({ props })}
                {@render trigger({ props })}
            {/snippet}
        </Dialog.Trigger>
    {/if}
    <Dialog.Portal>
        <Dialog.Overlay class="dialog--overlay" forceMount>
            {#snippet child({ open, props })}
                {#if open}
                    <div
                        {...props}
                        transition:fade={{
                            duration: 200,
                            easing: (t) => {
                                // cubic out. copied from svelte's transitions (used in the fly transition)
                                const f = t - 1.0;
                                return f * f * f + 1.0;
                            }
                        }}
                    ></div>
                {/if}
            {/snippet}
        </Dialog.Overlay>
        <Dialog.Content
            forceMount
            class={["dialog--content-container", widthClass, { fullScreenMobile }]}
            preventScroll
            {role}
            {...contentProps}
        >
            {#snippet child({ open, props })}
                {#if open}
                    <div {...props} transition:scale={{ duration: 200 }}>
                        <DialogHeader {title} />
                        <DialogContent>
                            {@render children()}
                        </DialogContent>
                        {#if actions}
                            <DialogActions>
                                {@render actions()}
                            </DialogActions>
                        {/if}
                    </div>
                {/if}
            {/snippet}
        </Dialog.Content>
    </Dialog.Portal>
</Dialog.Root>

<style lang="scss">
    :global(.dialog--overlay) {
        position: fixed;
        inset: 0;
        z-index: 150;
        // background: color-mix(in oklab, var(--color-black) 80%, transparent 20%);
        background: linear-gradient(
            180deg,
            color-mix(
                    in oklab,
                    var(--dialog-overlay-accent, var(--color-accent-dark))
                        var(--dialog-overlay-accent-mix, 60%),
                    transparent var(--dialog-overlay-transparent-mix, 40%)
                )
                0,
            color-mix(in oklab, var(--color-black) 80%, transparent 20%)
        );
        backdrop-filter: saturate(1.5) blur(5px);

        @media (prefers-color-scheme: dark) {
            --dialog-overlay-accent: var(--color-accent-dark);
            --dialog-overlay-accent-mix: 30%;
            --dialog-overlay-transparent-mix: 70%;
        }
    }

    :global(.dialog--content-container) {
        --dialog-content-padding: calc(var(--spacing) * 4);
        position: fixed;
        display: grid;
        grid-template-rows: auto minmax(0, 1fr) auto;
        overflow: hidden;
        background-color: var(--color-body);
        left: 50%;
        top: 50%;
        z-index: 150;
        transform: translate(-50%, -50%);
        inline-size: 100%;
        // block-size: 100%;
        max-inline-size: var(--dialog-content-_width, calc(100% - 2rem));
        max-block-size: var(--dialog-content-_height, calc(100% - 2rem));

        // padding: calc(var(--spacing) * 5);
        border: 1px solid var(--color-iron-medium);
        border-radius: var(--typical-radius);
        outline: 0;
        transition-property: all; // i cant figure out how to make it work when scoping it to "inline-size". it just doesnt work
        transition-duration: 0.1s;
        transition-timing-function: ease-out;
        box-shadow:
            rgba(0, 0, 0, 0.16) 0px 10px 36px 0px,
            rgba(0, 0, 0, 0.06) 0px 0px 0px 1px;

        @media (min-width: 768px) {
            --dialog-content-_width: var(--dialog-content-width, 500px);
        }
    }

    :global(.dialog--content-container).fullScreenMobile {
        @media (max-width: 768px) {
            --dialog-content-_width: 100%;
            --dialog-content-_height: 100%;
            block-size: 100%;
        }
    }

    :global(.dialog--content-container).dialog-large {
        --dialog-content-width: 650px;
    }
</style>
