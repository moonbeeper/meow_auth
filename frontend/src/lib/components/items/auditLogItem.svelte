<script lang="ts">
    // also TODO: REMOVE ME !!!!
    // "Complex binding patterns require an initialization value" when lang="ts" is not set... OH CMON!
    import Tag from "$comps/tag.svelte";

    let {
        title = "space birb",
        tag = "hatch",
        who = "me",
        when = "right now"
    }: { title?: string; tag?: string; who?: string; when?: string } = $props();
</script>

<button class="panel">
    <div class="header">
        <div class="title">
            <span>{title}</span>
            <Tag>{tag}</Tag>
        </div>
        <div class="who-when">
            <span>{who}</span>
            <span>·</span>
            <span>{when}</span>
        </div>
    </div>
    <span>V</span>
</button>

<style lang="scss">
    .panel {
        // all: unset; // kills everything. great.
        --border-stuffies: 1px solid var(--color-iron-dark);
        display: flex;
        inline-size: 100%;
        padding-block: calc(var(--spacing) * 3);
        padding-inline: calc(var(--spacing) * 4);
        align-items: center;
        gap: calc(var(--spacing) * 3);
        text-align: start;
        cursor: pointer;
        border: none;
        border-inline: var(--border-stuffies);
        background: transparent;
        transition: background-color 0.1s ease-out;
        font: inherit;

        &:first-child {
            border-top-left-radius: var(--typical-radius);
            border-top-right-radius: var(--typical-radius);
            border-top: var(--border-stuffies);
        }

        &:last-child {
            border-bottom-left-radius: var(--typical-radius);
            border-bottom-right-radius: var(--typical-radius);
            border-bottom: var(--border-stuffies);
        }

        &:hover {
            background: color-mix(in oklab, var(--color-accent-lightest) 50%, transparent 50%);
        }
    }

    .header {
        display: flex;
        flex-direction: column;
        min-inline-size: 0;
        flex: 1;
        font-size: var(--text-small);
    }

    .header .title {
        display: flex;
        gap: calc(var(--spacing) * 2);
        align-items: center;
        font-weight: 500;
    }

    .header .who-when {
        display: flex;
        gap: calc(var(--spacing) * 1);
        font-size: var(--text-smaller);
    }
</style>
