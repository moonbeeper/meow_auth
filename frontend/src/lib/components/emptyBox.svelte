<script lang="ts">
    import type { Snippet } from "svelte";

    type Props = {
        children?: Snippet;
        actions?: Snippet;
        title?: string;
        subtitle?: string;
    };

    let {
        children,
        actions,
        title = "No items to display",
        subtitle = "Sometimes space cats... are space cats. BUt space birbs are even better! and prettier! and flufifer! and that's it. i love space birbs. (psst, i loooove space birbs and mr guys wiht a fancy visor on le head. [pssssst, i like not saying what i mean but i mean it.])"
    }: Props = $props();
</script>

<div class="empty-box">
    {#if children}
        <div class="content">
            {@render children()}
        </div>
    {/if}

    <div class="header">
        <h3 class="title">{title}</h3>
        <p class="subtitle">{subtitle}</p>
    </div>

    {#if actions}
        <div class="actions">
            {@render actions()}
        </div>
    {/if}
</div>

<style lang="scss">
    .empty-box {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        padding-block: calc(var(--spacing) * 4);
        padding-inline: calc(var(--spacing) * 6);
        border-radius: var(--typical-radius);
        border: 1.5px dashed var(--color-iron-dark);
        gap: calc(var(--spacing) * 4);
    }

    .actions {
        display: flex;
        align-items: center;
        justify-content: center;
        inline-size: 100%;
    }

    .header {
        justify-content: center;
        display: flex;
        flex-direction: column;
        gap: calc(var(--spacing) * 1);

        max-inline-size: 32ch;
    }

    .title {
        font-size: var(--text-normal);
        font-weight: 600;
    }

    .subtitle {
        font-size: var(--text-smaller);
        // font-weight: 500;
    }
</style>
