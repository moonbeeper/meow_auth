<script lang="ts">
    import { fly, slide } from "svelte/transition";

    import Spinner from "./spinner.svelte";

    type Props = {
        /** What's loading?
         *
         * Example: "Audit log" or "Quirky space birb"
         */
        what?: string;
        hasError?: boolean;
        loaded?: boolean;
    };
    let { what, loaded = false, hasError: error = false }: Props = $props();

    let whatText = $derived.by(() => {
        return what ? what : "content";
    });

    let whatClass = $derived.by(() => {
        return error ? "error" : loaded ? "loaded" : "";
    });

    let dots = $state("");
    $effect(() => {
        let interval = setInterval(() => {
            dots = dots.length < 3 ? dots + "." : "";
        }, 500);

        return () => clearInterval(interval);
    });
</script>

<div class={["loading-box", whatClass]}>
    {#if error}
        <p>
            Failed loading {whatText} :(
        </p>
    {:else}
        <p>
            Loading {whatText}<span>{dots}</span>
        </p>
    {/if}
    {#if !(error || loaded)}
        <span transition:slide><Spinner /></span>
    {/if}
</div>

<style lang="scss">
    .loading-box {
        --loadbox-border-color: var(--color-amber-dark);
        padding-block: calc(var(--spacing) * 4);
        padding-inline: calc(var(--spacing) * 6);
        border-radius: var(--typical-radius);
        border: 2px dashed var(--loadbox-border-color);

        justify-content: center;
        display: flex;
        flex-direction: column;

        transition-property: border;
        transition-duration: 0.1s;
        transition-timing-function: ease-out;

        font-size: var(--text-normal);
        text-align: center;
        gap: calc(var(--spacing) * 2);
    }

    .error {
        --loadbox-border-color: var(--color-coral-dark);
    }

    .loaded {
        --loadbox-border-color: var(--color-iron-dark);
    }
</style>
