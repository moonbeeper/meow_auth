<script lang="ts">
    import * as LogItem from "$comps/logItem/index";
    // "Complex binding patterns require an initialization value" when lang="ts" is not set... OH CMON!
    import Tag from "$comps/tag.svelte";

    let {
        title = "my cool appy",
        when = "Created 68 days ago"
    }: { title?: string; when?: string } = $props();
</script>

<LogItem.Root>
    <LogItem.Header {title} {when} />
    <LogItem.Actions>
        <span>T</span>
    </LogItem.Actions>
</LogItem.Root>
