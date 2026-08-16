<script lang="ts">
    import type { Snippet } from "svelte";
    import type { HTMLInputAttributes } from "svelte/elements";

    import Input, { type FormProps } from "./input.svelte";

    type Props = {
        children: Snippet;
        inputProps?: FormProps;
    };

    type InputProxyProps = Props;

    let { children, inputProps }: InputProxyProps = $props();
</script>

<div class="input">
    <!-- <Input naked {...inputProps} /> -->
    {@render children()}
</div>

<style lang="scss">
    .input {
        --focus-outline-offset: -1px;
        --input-transition-duration: 0.1s;
        --input-transition-ease: ease-out;
        --input-padding-inline-end: calc(var(--spacing) * 3);
        border-radius: var(--input-radius, var(--typical-radius));
        border: 1px solid var(--input-border-color, var(--color-iron-medium));
        padding-inline-start: var(--input-padding-inline-start, calc(var(--spacing) * 3));
        padding-inline-end: var(--input-padding-inline-end, calc(var(--spacing) * 2));
        padding-block: var(--input-padding-block, calc(var(--spacing) * 3));
        accent-color: var(--color-accent-light);
        color: var(--input-text-color, var(--color-iron-darkest));
        font-size: var(--input-font-size, var(--text-normal));
        background: transparent;
        transition-property: background-color, border-color, color, outline;
        transition-duration: var(--input-transition-duration);
        transition-timing-function: var(--input-transition-ease);
        // omg THIS HECKING THIGN WAS MAKING THE phone VIEW GO WONK AND TRY TO FILL ALLL
        // the fricking screen instead of being constarint to the hecking padding it already had
        display: flex;
        inline-size: 100%;
        align-items: center;

        &:focus-within {
            --input-border-color: var(--color-accent-light);
            outline: var(--typical-outline-size) solid
                var(--focus-outline-color, var(--color-accent-light));
            outline-offset: -1px;
        }

        :global(.input) {
            --input-border-size: 0;
            --input-padding-inline-start: 0;
            --input-padding-inline-end: 0;
            --input-padding-block: 0;
            --input-border-color: transparent;
            --input-radius: 0;
            outline: 0 !important;
        }

        :global(a) {
            // without this, the line height makes the input be 48.84 tall instead of the correct 43.59
            // of a normal input that is not inside a proxy
            line-height: 1;
        }
    }
</style>
