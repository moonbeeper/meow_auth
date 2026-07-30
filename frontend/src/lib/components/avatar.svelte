<script lang="ts">
    import { Avatar } from "bits-ui";

    let { small = false }: { small?: boolean } = $props();
</script>

<!-- SVELTE USES CLSX NOW?!?!?!?!??!?? -->
<Avatar.Root class={["avatar", { small }]}>
    <div class="inner">
        <Avatar.Image src="https://github.com/moonbeeper.png" alt="Moonbeeper's avatar" />
        <Avatar.Fallback>HB</Avatar.Fallback>
    </div>
</Avatar.Root>

<style lang="scss">
    :global(.avatar) {
        position: relative;
        // width: 96px;
        // height: 96px;
        inline-size: 6rem;
        block-size: 6rem;
        flex-shrink: 0;
        // aspect-ratio: 1;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 600;
        font-size: var(--text-small);
        // overflow: hidden;
        border: 1px solid var(--color-accent-light);
        outline: 1px solid var(--color-iron-inverted);
        outline-offset: -3px;

        // subtle as HECK but i like it
        box-shadow: rgba(0, 0, 0, 0.05) 0px 6px 24px 0px;
        // &::before {
        //     position: absolute;
        //     content: "";
        //     // width: 100%;
        //     // height: 100%;
        //     inset: 1px;
        //     border-radius: 50%;
        //     border: 1px solid var(--color-accent-light);
        // }
        //
    }

    :global(.avatar.small) {
        max-inline-size: 2.25rem;
        max-block-size: 2.25rem;
    }

    .inner {
        display: flex;
        width: 100%;
        height: 100%;
        overflow: hidden;
        justify-content: center;
        align-items: center;
        border-radius: 50%;
    }
</style>
