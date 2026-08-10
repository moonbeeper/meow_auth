<script lang="ts">
    import { useFormField } from "formsnap";
    import type { HTMLAttributes } from "svelte/elements";
    import { fly } from "svelte/transition";

    import { getId } from "./idShenanigans";
    let { id = getId(), ...rest }: HTMLAttributes<HTMLElement> = $props();

    const field = useFormField({
        errorsId: () => id
    });
</script>

<!-- svelte is cool for having component comments (i have to use them haha) -->
<!--
@component
This is used to grab the form Control from FormSnap and display **the first error message** for that field.
-->

<!-- Shows the first error!! -->
{#if field.errors.length > 0}
    <div class="container" {id} {...rest} transition:fly={{ y: -5, duration: 200 }}>
        <p class="message">{field.errors[0]}</p>
    </div>
{/if}

<style lang="scss">
    .container {
        display: flex;
    }

    .message {
        color: var(--color-coral-medium);
        font-size: var(--text-small);
        text-align: center;
        font-weight: 500;

        @media (prefers-color-scheme: dark) {
            color: var(--color-coral-light);
        }
    }
</style>
