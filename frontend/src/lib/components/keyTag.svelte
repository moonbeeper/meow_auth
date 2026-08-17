<script lang="ts">
    import { getHumanDisplayKey, type KeysThatMatter } from "$lib/stupidKeymap";

    type Props = {
        /** The key that needs to be pressed to trigger the onKeyDown method */
        key: KeysThatMatter;
        /** A callback method to handle the keydown event
         *
         * The actual key filtering is already done by the component, but the event is passed to the callback
         * if it needed it for any reason.
         */
        onKeyDown?: (e: KeyboardEvent) => void;
    };

    let { key, onKeyDown }: Props = $props();

    function onkeydown(e: KeyboardEvent) {
        if (e.key != key) {
            return;
        }
        console.log(`key pressed! (${e.key}) calling callback`);
        onKeyDown?.(e);
    }

    let keyDisplay = $derived.by(() => {
        return getHumanDisplayKey(key);
    });
</script>

<svelte:window {onkeydown} />

<kbd class="kbd">{keyDisplay}</kbd>

<style lang="scss">
    .kbd {
        font-family: var(--font-mono);
        font-size: var(--text-smaller);
        font-weight: 600;
        opacity: 0.7;
        padding-inline: 0.3rem;
        text-transform: uppercase;
        text-decoration: none;
        vertical-align: middle;
        white-space: nowrap;
        border: 1px solid currentColor;
        border-radius: calc(var(--typical-radius) / 3);
        box-shadow: currentColor 0px 0.1em 0px;
        inline-size: fit-content;
    }
</style>
