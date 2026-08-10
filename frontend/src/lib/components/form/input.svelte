<script lang="ts">
    import type { HTMLInputAttributes } from "svelte/elements";

    type Props = {
        fontSize?: "normal" | "large";
    };

    type FormProps = HTMLInputAttributes & Props;

    let { fontSize = "normal", value = $bindable(), ...rest }: FormProps = $props();
    let fontSizeClass = $derived.by(() => {
        return "font-" + fontSize;
    });
</script>

<input class={["input", fontSizeClass]} placeholder="hello, hii" {...rest} bind:value />

<style lang="scss">
    .input {
        --focus-outline-offset: -1px;
        --input-transition-duration: 0.1s;
        --input-transition-ease: ease-out;
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
        inline-size: 100%;

        &:focus-visible {
            --input-border-color: var(--color-accent-light);
        }

        &::placeholder {
            transition-property: color;
            transition-duration: var(--input-transition-duration);
            transition-timing-function: var(--input-transition-ease);

            color: var(--input-placeholder-color, var(--color-iron-dark));
        }

        &[autocomplete="one-time-code"] {
            text-align: center;
            font-weight: 900;
            font-size: var(--text-larger);
            font-family: var(--font-mono);
            letter-spacing: 1ch; // new css stuff !!
            text-transform: uppercase;
            // same as letter spacing. It looked like if the text was uncentered AND IT WAS!!!
            // now with this padding, it makes so the text now looks centered.
            // This happens because mr letter spacing adds spacing between letter AND EVEN the last letter
            // making it look like "A A A A A ", see the space?. i intended to make it look like "A A A A A"
            // without the stupid last space in the end
            padding-inline-start: 1ch;
            padding-inline-end: 0;

            @media (max-width: 426px) {
                font-size: var(--text-large);
            }
            @media (max-width: 321px) {
                font-size: var(--text-medium);
            }
        }

        &[data-fs-invalid],
        &[aria-invalid="true"] {
            --input-border-color: var(--color-coral-medium);
            --focus-outline-color: var(--input-border-color);
            --input-text-color: var(--color-coral-darker);
            --input-placeholder-color: var(--input-text-color);

            @media (prefers-color-scheme: dark) {
                --input-border-color: var(--color-coral-light);
                --input-text-color: var(--color-coral-lighter);
            }
        }
    }

    .font-large {
        --input-font-size: var(--text-large);
    }
</style>
